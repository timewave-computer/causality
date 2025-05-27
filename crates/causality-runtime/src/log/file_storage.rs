// File-based log storage implementation
//
// This module provides a file-based storage implementation for the log system.

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write, Read, Seek, SeekFrom, BufRead};
use std::sync::{Arc, Mutex};
use anyhow::{Context, Result as AnyhowResult};

use crate::log::{LogEntry, entry::EntryType};
use causality_error::{EngineError, CausalityError, Result as CausalityResult};
use causality_types::TraceId;
use async_trait::async_trait;

/// File-based log storage
#[derive(Debug)]
pub struct FileLogStorage {
    file_path: PathBuf,
    // Use a mutex for simple thread-safe file access.
    file_mutex: Arc<Mutex<File>>,
}

impl FileLogStorage {
    /// Create a new file log storage
    pub fn new(path: impl AsRef<Path>) -> AnyhowResult<Self> {
        let path_buf = path.as_ref().to_path_buf();
        // Create parent directories if they don't exist
        if let Some(parent) = path_buf.parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {:?}", parent))?;
        }

        // Open file in append+read mode
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(&path_buf)
            .context(format!("Failed to open/create log file: {:?}", path_buf))?;

        Ok(Self {
            file_path: path_buf,
            file_mutex: Arc::new(Mutex::new(file)),
        })
    }

    // Helper for counting entries
    fn sync_entry_count(&self) -> AnyhowResult<usize> {
        let file = File::open(&self.file_path)
            .with_context(|| format!("FS Sync: Failed to open log file for reading: {:?}", self.file_path))?;
        let reader = BufReader::new(file);
        let count = reader.lines().count();
        Ok(count)
    }

    // Helper for reading entries
    fn sync_read(&self, offset: usize, limit: usize) -> AnyhowResult<Vec<LogEntry>> {
        let file = File::open(&self.file_path)
            .with_context(|| format!("FS Sync: Failed to open log file for reading: {:?}", self.file_path))?;
        let reader = BufReader::new(file);
        let entries: Vec<LogEntry> = reader.lines()
            .skip(offset)
            .take(limit)
            .filter_map(|line| {
                match line {
                    Ok(json_str) => {
                        match serde_json::from_str(&json_str) {
                            Ok(entry) => Some(entry),
                            Err(e) => {
                                eprintln!("FS Sync: Failed to deserialize log entry: {}", e);
                                None
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("FS Sync: Failed to read line: {}", e);
                        None
                    }
                }
            })
            .collect();

        Ok(entries)
    }

    // Helper for appending a single entry
    fn sync_append(&self, entry: &LogEntry) -> AnyhowResult<()> {
        let json = serde_json::to_string(entry)
            .with_context(|| format!("FS Sync: Failed to serialize entry ID {}", entry.id))?;

        let mut file = self.file_mutex.lock().unwrap();
        writeln!(file, "{}", json)
            .with_context(|| format!("FS Sync: Failed to write entry ID {} to file", entry.id))?;
        Ok(())
    }

    // Helper for appending multiple entries (batch)
    fn sync_append_batch(&self, entries: &[LogEntry]) -> AnyhowResult<()> {
        let mut file = self.file_mutex.lock().unwrap();
        let mut writer = BufWriter::new(&mut *file);

        for entry in entries {
            let json = serde_json::to_string(entry)
                .with_context(|| format!("FS Sync: Failed to serialize batch entry ID {}", entry.id))?;
            
            writeln!(writer, "{}", json)
                .with_context(|| format!("FS Sync: Failed to write batch entry ID {} to file", entry.id))?;
        }
        Ok(())
    }

    fn sync_get_entry_by_id(&self, id: &str) -> AnyhowResult<Option<LogEntry>> {
        let file = File::open(&self.file_path)
            .with_context(|| format!("FS Sync: Failed to open log file for reading: {:?}", self.file_path))?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            if let Ok(json_str) = line {
                if let Ok(entry) = serde_json::from_str::<LogEntry>(&json_str) {
                    if entry.id == id {
                        return Ok(Some(entry));
                    }
                }
            }
        }
        
        Ok(None)
    }

    fn sync_get_entries_by_trace(&self, trace_id: &str) -> AnyhowResult<Vec<LogEntry>> {
        let file = File::open(&self.file_path)
            .with_context(|| format!("FS Sync: Failed to open log file for reading: {:?}", self.file_path))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        
        for line in reader.lines() {
            if let Ok(json_str) = line {
                if let Ok(entry) = serde_json::from_str::<LogEntry>(&json_str) {
                    if let Some(entry_trace_id) = &entry.trace_id {
                        if entry_trace_id.to_string() == trace_id {
                            entries.push(entry);
                        }
                    }
                }
            }
        }
        
        Ok(entries)
    }
}

// Helper function to map Anyhow error to a boxed CausalityError
fn map_anyhow_error(e: anyhow::Error) -> Box<dyn CausalityError> {
    // Try to downcast to specific error types
    if let Some(io_err) = e.downcast_ref::<io::Error>() {
        return Box::new(EngineError::IoError(format!("IO Error: {}", io_err)));
    }
    
    if let Some(serde_err) = e.downcast_ref::<serde_json::Error>() {
        if serde_err.is_eof() || serde_err.is_io() {
            return Box::new(EngineError::IoError(format!("JSON IO Error: {}", serde_err)));
        } else if serde_err.is_syntax() {
            return Box::new(EngineError::DeserializationFailed(format!("JSON Syntax Error: {}", serde_err)));
        } else if serde_err.is_data() {
            return Box::new(EngineError::DeserializationFailed(format!("JSON Data Error: {}", serde_err)));
        }
    }
    
    // Fallback for other errors
    Box::new(EngineError::Other(format!("{:?}", e)))
}

// Implement LogStorage trait for FileLogStorage
#[async_trait]
impl crate::log::LogStorage for FileLogStorage {
    fn entry_count(&self) -> CausalityResult<usize> {
        self.sync_entry_count().map_err(map_anyhow_error)
    }

    fn read(&self, offset: usize, limit: usize) -> CausalityResult<Vec<LogEntry>> {
        self.sync_read(offset, limit).map_err(map_anyhow_error)
    }

    fn append(&self, entry: LogEntry) -> CausalityResult<()> {
        self.sync_append(&entry).map_err(map_anyhow_error)
    }
    
    fn get_entry_by_id(&self, id: &str) -> CausalityResult<Option<LogEntry>> {
        self.sync_get_entry_by_id(id).map_err(map_anyhow_error)
    }
    
    fn get_entries_by_trace(&self, trace_id: &str) -> CausalityResult<Vec<LogEntry>> {
        self.sync_get_entries_by_trace(trace_id).map_err(map_anyhow_error)
    }
    
    // Override the default implementation of append_batch for efficiency
    fn append_batch(&self, entries: Vec<LogEntry>) -> CausalityResult<()> {
        self.sync_append_batch(&entries).map_err(map_anyhow_error)
    }
    
    // We use the default implementations from the trait for:
    // read_time_range
    // find_entries_by_trace_id
    // find_entries_by_type
    // rotate
    // compact
    // close
    // All async methods
} 