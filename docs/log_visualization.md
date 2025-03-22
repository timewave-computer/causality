# Log Visualization in Causality

This document describes the log visualization tools available in the Causality framework and how to use them effectively.

## Overview

The Causality framework provides a set of powerful tools for visualizing and exploring log entries. These tools enable users to:

- Filter log entries by time, type, domain, and text content
- Visualize causal relationships between log entries
- Export visualizations in various formats
- Search and analyze log data efficiently

## Core Components

### LogVisualizer

The `LogVisualizer` is the main entry point for visualization operations. It provides methods for filtering entries, generating causality graphs, and creating visualizations in different formats.

```rust
// Create a new log visualizer with a log storage
let storage = Arc::new(MemoryLogStorage::new());
let visualizer = LogVisualizer::new(storage.clone());

// Get filtered entries
let entries = visualizer.get_filtered_entries(&filter).await?;

// Create a causality graph
let graph = visualizer.create_causality_graph(&filter).await?;

// Generate a visualization
let content = visualizer.visualize(&filter, VisualizationFormat::Dot).await?;
```

### VisualizationFilter

The `VisualizationFilter` allows you to define criteria for filtering log entries:

```rust
// Create a new filter
let filter = VisualizationFilter::new()
    .with_time_range(start_time, end_time)         // Filter by time range
    .with_entry_types(vec![EntryType::Effect])     // Filter by entry type
    .with_domains(vec!["banking".to_string()])     // Filter by domain
    .with_search_text("deposit")                   // Filter by text content
    .with_entry_id("entry-123")                    // Filter by specific entry ID
    .with_parent_id("parent-456");                 // Filter by parent ID
```

### CausalityGraph

The `CausalityGraph` represents the causal relationships between log entries. It can be created from filtered log entries and provides methods for visualization and analysis:

```rust
// Create a causality graph from filtered entries
let graph = visualizer.create_causality_graph(&filter).await?;

// Visualize the graph in ASCII format
println!("{}", graph.visualize());

// Export the graph as a DOT file for GraphViz
let dot_content = graph.to_dot("Log Causality");
std::fs::write("causality.dot", dot_content)?;

// Find the path to a specific node
let path = graph.find_path_to("entry-123");
```

## Visualization Formats

The log visualizer supports multiple output formats:

- **Text**: A simple text-based representation of log entries
- **JSON**: A structured JSON format for programmatic processing
- **DOT**: GraphViz DOT format for rendering complex graphs
- **HTML**: Interactive HTML visualization with expandable details

To generate a visualization in a specific format:

```rust
let content = visualizer.visualize(&filter, VisualizationFormat::Html).await?;
```

## Command Line Interface

The Causality framework includes a CLI tool for interactive log visualization. The CLI provides a user-friendly interface for:

- Browsing and filtering log entries
- Viewing causality graphs
- Exporting visualizations in different formats
- Searching and analyzing log data

To run the CLI tool:

```rust
// Create a log storage
let storage = Arc::new(MemoryLogStorage::new());

// Create and run the CLI
let mut cli = LogViewerCli::new(storage);
cli.run().await?;
```

## Visualization Best Practices

1. **Start with broad filters**: Begin with minimal filtering and gradually narrow down to find specific patterns.

2. **Use time-based filtering**: For large logs, always start by filtering to a specific time range.

3. **Focus on domains**: Filtering by domain can help isolate specific subsystems.

4. **Export complex graphs**: For large causality graphs, export to DOT format and use GraphViz for visualization.

5. **Search for specific terms**: Use text search to find entries related to specific operations or errors.

6. **Analyze root causes**: Use the causality graph to trace effects back to their originating causes.

## Advanced Usage

### Custom Visualizations

You can extend the visualization system with custom formats by implementing the appropriate methods:

```rust
pub enum VisualizationFormat {
    Text,
    Json,
    Dot,
    Html,
    // Add your custom format here
}

// Then extend the visualizer to handle your format
```

### Integration with External Tools

The DOT format output can be integrated with GraphViz for advanced graph visualization:

```bash
# Generate a PNG from DOT file
dot -Tpng causality.dot -o causality.png

# Generate an SVG for web integration
dot -Tsvg causality.dot -o causality.svg
```

## Example: Analyzing Transaction Flow

Here's a practical example of using log visualization to analyze a transaction flow:

1. Filter logs to the "banking" domain
2. Search for a specific transaction ID
3. Generate a causality graph to see all effects and facts
4. Export to DOT format for detailed analysis
5. Trace the path from an error event back to the originating action

This process can help identify issues in complex workflows and understand the causal relationships between different system components.

## Conclusion

The log visualization tools in the Causality framework provide powerful capabilities for exploring, analyzing, and understanding log data. By using these tools effectively, developers and operators can gain insights into system behavior, diagnose issues, and understand complex causal relationships. 