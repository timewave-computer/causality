//! CLI Error Handling System
//!
//! This module implements error handling specifically for the CLI environment,
//! including colored terminal output, verbosity control, and appropriate exit codes.

//-----------------------------------------------------------------------------
// CLI Error Handling System
//-----------------------------------------------------------------------------

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use chrono::prelude::*;
use serde_json::{self, json, Value};

/// Shared error handler for command line operations
#[derive(Clone)]
pub struct CliErrorHandler {
    pub output_path: Option<PathBuf>,
    pub verbose: bool,
    pub json: bool,
}

impl CliErrorHandler {
    pub fn new(output_path: Option<PathBuf>, verbose: bool, json: bool) -> Self {
        Self {
            output_path,
            verbose,
            json,
        }
    }

    pub fn handle_error(&self, error: &anyhow::Error) -> Value {
        let error_message = error.to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let error_obj = json!({
            "error": error_message,
            "timestamp": now,
            "timestamp_human": Local::now().to_rfc3339(),
        });

        if self.verbose {
            eprintln!("Error: {}", error_message);
            for cause in error.chain().skip(1) {
                eprintln!("Caused by: {}", cause);
            }
        } else {
            eprintln!("Error: {}", error_message);
        }

        if self.json {
            if let Some(path) = &self.output_path {
                let _ = std::fs::write(path, error_obj.to_string());
            }
        }

        error_obj
    }

    #[allow(dead_code)]
    pub fn handle_success<T>(&self, data: T) -> Result<Value> 
    where T: serde::Serialize
    {
        let data_value = serde_json::to_value(data)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let result = json!({
            "success": true,
            "timestamp": now,
            "timestamp_human": Local::now().to_rfc3339(),
            "data": data_value,
        });

        if self.json {
            if let Some(path) = &self.output_path {
                std::fs::write(path, result.to_string())?;
            }
        }

        Ok(result)
    }

    // Helper method to create a standardized error
    pub fn create_error(&self, message: impl Into<String>, category: impl Into<String>) -> anyhow::Error {
        anyhow!("{}: {}", category.into(), message.into())
    }
}

/// CLI result type alias
pub type CliResult<T> = Result<T>;

/// Helper macro for creating CLI errors - simplified to use anyhow
#[macro_export]
macro_rules! cli_error {
    ($handler:expr, $category:expr, $message:expr) => {
        anyhow::anyhow!("{}: {}", $category, $message)
    };
    ($handler:expr, $category:expr, $message:expr, $($key:expr => $value:expr),*) => {{
        anyhow::anyhow!("{}: {}", $category, $message)
    }};
}
