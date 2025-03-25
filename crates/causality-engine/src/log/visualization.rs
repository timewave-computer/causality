// Log visualization tools
// Original file: src/log/visualization.rs

// Log Visualization Module
//
// This module provides tools for visualizing log entries, including 
// fact-effect causality, time-based filtering, and searching.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDateTime};

use causality_types::{Error, Result};
use crate::log::{LogEntry, EntryType, LogStorage};

/// Filter criteria for selecting log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationFilter {
    /// Start timestamp (inclusive)
    pub start_time: Option<u64>,
    /// End timestamp (inclusive)
    pub end_time: Option<u64>,
    /// Entry types to include
    pub entry_types: Option<Vec<EntryType>>,
    /// Domains to include
    pub domains: Option<Vec<String>>,
    /// Text to search for in entries
    pub search_text: Option<String>,
    /// Include specific entry IDs
    pub entry_ids: Option<Vec<String>>,
    /// Include entries with specific parent IDs
    pub parent_ids: Option<Vec<String>>,
}

impl Default for VisualizationFilter {
    fn default() -> Self {
        VisualizationFilter {
            start_time: None,
            end_time: None,
            entry_types: None,
            domains: None,
            search_text: None,
            entry_ids: None,
            parent_ids: None,
        }
    }
}

impl VisualizationFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set time range
    pub fn with_time_range(mut self, start: Option<u64>, end: Option<u64>) -> Self {
        self.start_time = start;
        self.end_time = end;
        self
    }
    
    /// Add entry types to filter
    pub fn with_entry_types(mut self, types: Vec<EntryType>) -> Self {
        self.entry_types = Some(types);
        self
    }
    
    /// Add domains to filter
    pub fn with_domains(mut self, domains: Vec<String>) -> Self {
        self.domains = Some(domains);
        self
    }
    
    /// Add search text
    pub fn with_search_text(mut self, text: &str) -> Self {
        self.search_text = Some(text.to_string());
        self
    }
    
    /// Add specific entry IDs
    pub fn with_entry_ids(mut self, ids: Vec<String>) -> Self {
        self.entry_ids = Some(ids);
        self
    }
    
    /// Add parent IDs
    pub fn with_parent_ids(mut self, ids: Vec<String>) -> Self {
        self.parent_ids = Some(ids);
        self
    }
    
    /// Check if an entry matches the filter criteria
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Check timestamp
        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }
        
        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }
        
        // Check entry type
        if let Some(types) = &self.entry_types {
            if !types.contains(&entry.entry_type) {
                return false;
            }
        }
        
        // Check domain
        if let Some(domains) = &self.domains {
            if !domains.contains(&entry.domain) {
                return false;
            }
        }
        
        // Check entry ID
        if let Some(ids) = &self.entry_ids {
            if !ids.contains(&entry.id) {
                return false;
            }
        }
        
        // Check parent ID
        if let Some(parent_ids) = &self.parent_ids {
            match &entry.parent_id {
                Some(parent_id) => {
                    if !parent_ids.contains(parent_id) {
                        return false;
                    }
                },
                None => {
                    if !parent_ids.is_empty() {
                        return false;
                    }
                }
            }
        }
        
        // Check search text if specified
        if let Some(search) = &self.search_text {
            let search_lower = search.to_lowercase();
            
            // Check in domain
            if entry.domain.to_lowercase().contains(&search_lower) {
                return true;
            }
            
            // Check in ID
            if entry.id.to_lowercase().contains(&search_lower) {
                return true;
            }
            
            // Check in parent ID if present
            if let Some(parent_id) = &entry.parent_id {
                if parent_id.to_lowercase().contains(&search_lower) {
                    return true;
                }
            }
            
            // Check in metadata
            for (key, value) in &entry.metadata {
                if key.to_lowercase().contains(&search_lower) || 
                   value.to_lowercase().contains(&search_lower) {
                    return true;
                }
            }
            
            // Check in data - this is approximate since we don't parse the data
            match &entry.data {
                // If JSON, convert to string and check
                causality_engine::EntryData::Json(json) => {
                    if let Ok(json_str) = serde_json::to_string(json) {
                        if json_str.to_lowercase().contains(&search_lower) {
                            return true;
                        }
                    }
                },
                // If binary, we can't really search effectively
                causality_engine::EntryData::Binary(_, content_type) => {
                    if content_type.to_lowercase().contains(&search_lower) {
                        return true;
                    }
                },
                // If text, we can search directly
                causality_engine::EntryData::Text(text) => {
                    if text.to_lowercase().contains(&search_lower) {
                        return true;
                    }
                },
            }
            
            // If we get here and search is enabled, this entry doesn't match
            return false;
        }
        
        // If we get here, all specified criteria match
        true
    }
}

/// A node in the causality graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityNode {
    /// Entry ID
    pub id: String,
    /// Entry type
    pub entry_type: EntryType,
    /// Domain
    pub domain: String,
    /// Timestamp
    pub timestamp: u64,
    /// Human-readable summary
    pub summary: String,
    /// Parent ID if any
    pub parent_id: Option<String>,
    /// Children IDs
    pub children: Vec<String>,
}

/// A complete causality graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityGraph {
    /// All nodes in the graph
    pub nodes: HashMap<String, CausalityNode>,
    /// Root nodes (no parent)
    pub roots: Vec<String>,
}

impl CausalityGraph {
    /// Create a new empty causality graph
    pub fn new() -> Self {
        CausalityGraph {
            nodes: HashMap::new(),
            roots: Vec::new(),
        }
    }
    
    /// Build a causality graph from a list of log entries
    pub fn from_entries(entries: &[LogEntry]) -> Self {
        let mut graph = CausalityGraph::new();
        
        // First pass: create nodes
        for entry in entries {
            let summary = match &entry.data {
                causality_engine::EntryData::Json(json) => {
                    format!("{}: {}", entry.entry_type.as_str(), json)
                },
                causality_engine::EntryData::Binary(_, content_type) => {
                    format!("{}: Binary data ({})", entry.entry_type.as_str(), content_type)
                },
                causality_engine::EntryData::Text(text) => {
                    let preview = if text.len() > 50 {
                        format!("{}...", &text[..47])
                    } else {
                        text.clone()
                    };
                    format!("{}: {}", entry.entry_type.as_str(), preview)
                },
            };
            
            let node = CausalityNode {
                id: entry.id.clone(),
                entry_type: entry.entry_type.clone(),
                domain: entry.domain.clone(),
                timestamp: entry.timestamp,
                summary,
                parent_id: entry.parent_id.clone(),
                children: Vec::new(),
            };
            
            graph.nodes.insert(entry.id.clone(), node);
        }
        
        // Second pass: build relationships
        for entry in entries {
            if let Some(parent_id) = &entry.parent_id {
                if let Some(parent_node) = graph.nodes.get_mut(parent_id) {
                    parent_node.children.push(entry.id.clone());
                }
            } else {
                graph.roots.push(entry.id.clone());
            }
        }
        
        graph
    }
    
    /// Get a visualization of the graph in ASCII
    pub fn visualize(&self) -> String {
        let mut result = String::new();
        
        // Process roots first
        for root_id in &self.roots {
            if let Some(root) = self.nodes.get(root_id) {
                self.visualize_node(&mut result, root, 0);
            }
        }
        
        result
    }
    
    /// Recursively visualize a node and its children
    fn visualize_node(&self, output: &mut String, node: &CausalityNode, depth: usize) {
        // Format timestamp
        let dt = NaiveDateTime::from_timestamp_opt((node.timestamp / 1000) as i64, 0)
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| format!("Invalid: {}", node.timestamp));
        
        // Indentation
        let indent = "  ".repeat(depth);
        
        // Node representation
        output.push_str(&format!("{}{} [{}] {} ({})\n", 
            indent,
            match node.entry_type {
                EntryType::Effect => "⚡",
                EntryType::Fact => "📋",
                EntryType::Event => "🔔",
            },
            dt,
            node.summary,
            node.domain
        ));
        
        // Process children
        for child_id in &node.children {
            if let Some(child) = self.nodes.get(child_id) {
                // Add connecting line
                output.push_str(&format!("{}  ↓\n", indent));
                
                // Recursively visualize child
                self.visualize_node(output, child, depth + 1);
            }
        }
    }
    
    /// Export the graph as a DOT file for visualization with GraphViz
    pub fn to_dot(&self) -> String {
        let mut result = String::new();
        
        // Start digraph
        result.push_str("digraph causality {\n");
        result.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"Arial\"];\n");
        
        // Add nodes
        for (id, node) in &self.nodes {
            let color = match node.entry_type {
                EntryType::Effect => "lightblue",
                EntryType::Fact => "lightgreen",
                EntryType::Event => "lightyellow",
            };
            
            let label = format!("{} ({}): {}", 
                node.entry_type.as_str(), 
                node.domain,
                node.summary.replace("\"", "\\\"") // Escape quotes
            );
            
            result.push_str(&format!("  \"{}\" [label=\"{}\", fillcolor=\"{}\"];\n", 
                id, label, color));
        }
        
        // Add edges
        for (id, node) in &self.nodes {
            for child_id in &node.children {
                result.push_str(&format!("  \"{}\" -> \"{}\";\n", id, child_id));
            }
        }
        
        // End digraph
        result.push_str("}\n");
        
        result
    }
    
    /// Find paths from any root to a specific node
    pub fn find_paths_to_node(&self, target_id: &str) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        
        for root_id in &self.roots {
            let mut path = Vec::new();
            if self.find_path_dfs(root_id, target_id, &mut path) {
                paths.push(path);
            }
        }
        
        paths
    }
    
    /// Helper for DFS path finding
    fn find_path_dfs(&self, current_id: &str, target_id: &str, path: &mut Vec<String>) -> bool {
        // Add current to path
        path.push(current_id.to_string());
        
        // Check if we found target
        if current_id == target_id {
            return true;
        }
        
        // Explore children
        if let Some(node) = self.nodes.get(current_id) {
            for child_id in &node.children {
                if self.find_path_dfs(child_id, target_id, path) {
                    return true;
                }
            }
        }
        
        // If we get here, we didn't find target in this branch
        path.pop();
        false
    }
}

/// Format for exporting log visualizations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizationFormat {
    /// Plain text
    Text,
    /// JSON 
    Json,
    /// DOT format for GraphViz
    Dot,
    /// HTML
    Html,
}

/// Log visualization service
pub struct LogVisualizer {
    /// Log storage
    storage: Arc<dyn LogStorage>,
}

impl LogVisualizer {
    /// Create a new log visualizer
    pub fn new(storage: Arc<dyn LogStorage>) -> Self {
        LogVisualizer { storage }
    }
    
    /// Get filtered entries
    pub async fn get_filtered_entries(&self, filter: &VisualizationFilter) -> Result<Vec<LogEntry>> {
        let mut entries = Vec::new();
        
        // Get time range from filter or use full range
        let start_pos = 0; // For simplicity, start from beginning
        let end_pos = match self.storage.get_entry_count() {
            Ok(count) => count,
            Err(e) => return Err(Error::Log(e)),
        };
        
        // Get all entries in range
        let all_entries = match self.storage.get_entries(start_pos, end_pos).await {
            Ok(e) => e,
            Err(e) => return Err(Error::Log(e)),
        };
        
        // Apply filter
        for entry in all_entries {
            if filter.matches(&entry) {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    /// Create a causality graph from filtered entries
    pub async fn create_causality_graph(&self, filter: &VisualizationFilter) -> Result<CausalityGraph> {
        let entries = self.get_filtered_entries(filter).await?;
        Ok(CausalityGraph::from_entries(&entries))
    }
    
    /// Visualize log entries
    pub async fn visualize(
        &self, 
        filter: &VisualizationFilter,
        format: VisualizationFormat
    ) -> Result<String> {
        let graph = self.create_causality_graph(filter).await?;
        
        match format {
            VisualizationFormat::Text => Ok(graph.visualize()),
            VisualizationFormat::Json => {
                serde_json::to_string_pretty(&graph)
                    .map_err(|e| Error::Internal(format!("JSON serialization error: {}", e)))
            },
            VisualizationFormat::Dot => Ok(graph.to_dot()),
            VisualizationFormat::Html => {
                // Create a basic HTML visualization
                let mut html = String::new();
                html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
                html.push_str("<title>Log Visualization</title>\n");
                html.push_str("<style>\n");
                html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
                html.push_str(".entry { margin: 10px 0; padding: 10px; border-radius: 5px; }\n");
                html.push_str(".effect { background-color: #d0e0ff; }\n");
                html.push_str(".fact { background-color: #d0ffd0; }\n");
                html.push_str(".event { background-color: #ffffd0; }\n");
                html.push_str(".entry-header { font-weight: bold; }\n");
                html.push_str(".entry-timestamp { color: #666; }\n");
                html.push_str(".entry-domain { color: #333; font-style: italic; }\n");
                html.push_str(".entry-children { margin-left: 30px; }\n");
                html.push_str("</style>\n");
                html.push_str("</head>\n<body>\n");
                
                html.push_str("<h1>Log Visualization</h1>\n");
                
                // Process roots
                for root_id in &graph.roots {
                    if let Some(node) = graph.nodes.get(root_id) {
                        self.html_visualize_node(&mut html, &graph, node);
                    }
                }
                
                html.push_str("</body>\n</html>");
                
                Ok(html)
            }
        }
    }
    
    /// Helper for HTML visualization
    fn html_visualize_node(&self, html: &mut String, graph: &CausalityGraph, node: &CausalityNode) {
        // Determine CSS class
        let css_class = match node.entry_type {
            EntryType::Effect => "effect",
            EntryType::Fact => "fact",
            EntryType::Event => "event",
        };
        
        // Format timestamp
        let dt = NaiveDateTime::from_timestamp_opt((node.timestamp / 1000) as i64, 0)
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| format!("Invalid: {}", node.timestamp));
        
        // Node representation
        html.push_str(&format!("<div class=\"entry {}\" id=\"{}\">\n", css_class, node.id));
        html.push_str(&format!("  <div class=\"entry-header\">{}</div>\n", node.summary));
        html.push_str(&format!("  <div class=\"entry-timestamp\">{}</div>\n", dt));
        html.push_str(&format!("  <div class=\"entry-domain\">{}</div>\n", node.domain));
        
        // Process children
        if !node.children.is_empty() {
            html.push_str("  <div class=\"entry-children\">\n");
            
            for child_id in &node.children {
                if let Some(child) = graph.nodes.get(child_id) {
                    self.html_visualize_node(html, graph, child);
                }
            }
            
            html.push_str("  </div>\n");
        }
        
        html.push_str("</div>\n");
    }
    
    /// Find causality relationships for a specific entry
    pub async fn find_causality(&self, entry_id: &str) -> Result<CausalityGraph> {
        // Create a filter for this entry and its related entries
        let mut filter = VisualizationFilter::new();
        
        // First, get this specific entry
        let mut entry_ids = HashSet::new();
        entry_ids.insert(entry_id.to_string());
        
        // We'll add related entries in iterations
        let mut expanded = true;
        while expanded {
            expanded = false;
            
            // Get all entries with the current set of IDs
            let filter = VisualizationFilter::new()
                .with_entry_ids(entry_ids.iter().cloned().collect());
                
            let entries = self.get_filtered_entries(&filter).await?;
            
            // Collect parent IDs and child IDs
            let mut new_ids = HashSet::new();
            
            for entry in &entries {
                // Add parent if present
                if let Some(parent_id) = &entry.parent_id {
                    if !entry_ids.contains(parent_id) {
                        new_ids.insert(parent_id.clone());
                        expanded = true;
                    }
                }
                
                // Add children (we need to search for entries with this as parent)
                let child_filter = VisualizationFilter::new()
                    .with_parent_ids(vec![entry.id.clone()]);
                    
                let children = self.get_filtered_entries(&child_filter).await?;
                
                for child in &children {
                    if !entry_ids.contains(&child.id) {
                        new_ids.insert(child.id.clone());
                        expanded = true;
                    }
                }
            }
            
            // Add new IDs to our set
            entry_ids.extend(new_ids);
        }
        
        // Create final filter with all related entries
        let filter = VisualizationFilter::new()
            .with_entry_ids(entry_ids.iter().cloned().collect());
            
        let entries = self.get_filtered_entries(&filter).await?;
        
        Ok(CausalityGraph::from_entries(&entries))
    }
}

/// Implementation of EntryType as_str
impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::Effect => "Effect",
            EntryType::Fact => "Fact",
            EntryType::Event => "Event",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{MemoryLogStorage, EntryData};
    
    #[tokio::test]
    async fn test_filtering() {
        // Create a storage with test entries
        let storage = Arc::new(MemoryLogStorage::new());
        
        // Add some test entries
        let entries = vec![
            LogEntry {
                id: "1".to_string(),
                timestamp: 100,
                domain: "domain1".to_string(),
                entry_type: EntryType::Effect,
                data: EntryData::Text("Test effect 1".to_string()),
                metadata: HashMap::new(),
                hash: None,
                parent_id: None,
            },
            LogEntry {
                id: "2".to_string(),
                timestamp: 200,
                domain: "domain2".to_string(),
                entry_type: EntryType::Fact,
                data: EntryData::Text("Test fact 1".to_string()),
                metadata: HashMap::new(),
                hash: None,
                parent_id: Some("1".to_string()),
            },
            LogEntry {
                id: "3".to_string(),
                timestamp: 300,
                domain: "domain1".to_string(),
                entry_type: EntryType::Event,
                data: EntryData::Text("Test event 1".to_string()),
                metadata: HashMap::new(),
                hash: None,
                parent_id: Some("2".to_string()),
            },
        ];
        
        storage.add_entries(&entries).await.unwrap();
        
        // Create visualizer
        let visualizer = LogVisualizer::new(storage);
        
        // Test time filter
        let time_filter = VisualizationFilter::new()
            .with_time_range(Some(150), Some(250));
            
        let filtered = visualizer.get_filtered_entries(&time_filter).await.unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "2");
        
        // Test type filter
        let type_filter = VisualizationFilter::new()
            .with_entry_types(vec![EntryType::Effect, EntryType::Event]);
            
        let filtered = visualizer.get_filtered_entries(&type_filter).await.unwrap();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "1");
        assert_eq!(filtered[1].id, "3");
        
        // Test domain filter
        let domain_filter = VisualizationFilter::new()
            .with_domains(vec!["domain1".to_string()]);
            
        let filtered = visualizer.get_filtered_entries(&domain_filter).await.unwrap();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "1");
        assert_eq!(filtered[1].id, "3");
        
        // Test search filter
        let search_filter = VisualizationFilter::new()
            .with_search_text("fact");
            
        let filtered = visualizer.get_filtered_entries(&search_filter).await.unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "2");
        
        // Test parent ID filter
        let parent_filter = VisualizationFilter::new()
            .with_parent_ids(vec!["1".to_string()]);
            
        let filtered = visualizer.get_filtered_entries(&parent_filter).await.unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "2");
    }
    
    #[tokio::test]
    async fn test_causality_graph() {
        // Create a storage with test entries
        let storage = Arc::new(MemoryLogStorage::new());
        
        // Add some test entries with causal relationships
        let entries = vec![
            LogEntry {
                id: "1".to_string(),
                timestamp: 100,
                domain: "domain1".to_string(),
                entry_type: EntryType::Effect,
                data: EntryData::Text("Root effect".to_string()),
                metadata: HashMap::new(),
                hash: None,
                parent_id: None,
            },
            LogEntry {
                id: "2".to_string(),
                timestamp: 200,
                domain: "domain1".to_string(),
                entry_type: EntryType::Fact,
                data: EntryData::Text("Child fact".to_string()),
                metadata: HashMap::new(),
                hash: None,
                parent_id: Some("1".to_string()),
            },
            LogEntry {
                id: "3".to_string(),
                timestamp: 300,
                domain: "domain2".to_string(),
                entry_type: EntryType::Effect,
                data: EntryData::Text("Grandchild effect".to_string()),
                metadata: HashMap::new(),
                hash: None,
                parent_id: Some("2".to_string()),
            },
            LogEntry {
                id: "4".to_string(),
                timestamp: 400,
                domain: "domain2".to_string(),
                entry_type: EntryType::Event,
                data: EntryData::Text("Another child event".to_string()),
                metadata: HashMap::new(),
                hash: None,
                parent_id: Some("1".to_string()),
            },
        ];
        
        storage.add_entries(&entries).await.unwrap();
        
        // Create visualizer
        let visualizer = LogVisualizer::new(storage);
        
        // Get all entries
        let filter = VisualizationFilter::new();
        
        // Create causality graph
        let graph = visualizer.create_causality_graph(&filter).await.unwrap();
        
        // Verify graph structure
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.roots.len(), 1);
        assert_eq!(graph.roots[0], "1");
        
        let root = graph.nodes.get("1").unwrap();
        assert_eq!(root.children.len(), 2);
        assert!(root.children.contains(&"2".to_string()));
        assert!(root.children.contains(&"4".to_string()));
        
        let child = graph.nodes.get("2").unwrap();
        assert_eq!(child.children.len(), 1);
        assert_eq!(child.children[0], "3");
        
        let grandchild = graph.nodes.get("3").unwrap();
        assert_eq!(grandchild.children.len(), 0);
        
        // Verify visualization formats
        let dot = visualizer.visualize(&filter, VisualizationFormat::Dot).await.unwrap();
        assert!(dot.contains("digraph causality"));
        assert!(dot.contains("\"1\" -> \"2\""));
        assert!(dot.contains("\"2\" -> \"3\""));
        assert!(dot.contains("\"1\" -> \"4\""));
        
        let text = visualizer.visualize(&filter, VisualizationFormat::Text).await.unwrap();
        assert!(text.contains("Root effect"));
        assert!(text.contains("Child fact"));
        assert!(text.contains("Grandchild effect"));
    }
} 