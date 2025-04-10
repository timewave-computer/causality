//! Export module for the Temporal Effect Graph (TEG) API
//! 
//! This module provides export capabilities for TEGs, including:
//! 1. Export to visualization formats (DOT, Mermaid)
//! 2. Export to serialization formats (JSON, YAML)
//! 3. Export to interchange formats for external tools

use std::collections::HashMap;
use std::fmt::Write;
use anyhow::{Result, anyhow};

use crate::{
    TemporalEffectGraph, EffectId, ResourceId, DomainId,
    effect_node::ParameterValue,
};

/// Export format for TEG visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// DOT format for Graphviz
    DOT,
    /// Mermaid diagram format 
    Mermaid,
    /// JSON format
    JSON,
    /// YAML format
    YAML,
    /// Custom format
    Custom,
}

/// Options for TEG exports
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Include resource nodes in export
    pub include_resources: bool,
    /// Include parameter details
    pub include_parameters: bool,
    /// Include domain information
    pub include_domains: bool,
    /// Include capability information
    pub include_capabilities: bool,
    /// Include metadata
    pub include_metadata: bool,
    /// Filter to specific domains
    pub domain_filter: Option<Vec<DomainId>>,
    /// Simplify export by removing certain details
    pub simplify: bool,
    /// Custom options for specific export formats
    pub custom_options: HashMap<String, String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_resources: true,
            include_parameters: true,
            include_domains: true,
            include_capabilities: false,
            include_metadata: false,
            domain_filter: None,
            simplify: false,
            custom_options: HashMap::new(),
        }
    }
}

/// The TEG Exporter provides export capabilities for TEGs
#[derive(Debug)]
pub struct TEGExporter<'a> {
    /// Reference to the TEG being exported
    teg: &'a TemporalEffectGraph,
}

impl<'a> TEGExporter<'a> {
    /// Create a new TEG exporter
    pub fn new(teg: &'a TemporalEffectGraph) -> Self {
        Self { teg }
    }
    
    /// Export TEG to specified format
    pub fn export(&self, format: ExportFormat, options: &ExportOptions) -> Result<String> {
        match format {
            ExportFormat::DOT => self.export_dot(options),
            ExportFormat::Mermaid => self.export_mermaid(options),
            ExportFormat::JSON => self.export_json(options),
            ExportFormat::YAML => self.export_yaml(options),
            ExportFormat::Custom => {
                let format_name = options.custom_options.get("format_name")
                    .ok_or_else(|| anyhow!("Custom format requires 'format_name' option"))?;
                
                self.export_custom(format_name, options)
            }
        }
    }
    
    /// Export TEG to DOT format for Graphviz
    pub fn export_dot(&self, options: &ExportOptions) -> Result<String> {
        let mut output = String::new();
        
        // Start digraph
        writeln!(output, "digraph TemporalEffectGraph {{")?;
        writeln!(output, "  rankdir=LR;")?;
        writeln!(output, "  node [shape=box, style=filled, fillcolor=lightblue];")?;
        
        // Export effect nodes
        for (id, effect) in &self.teg.effect_nodes {
            // Apply domain filter if present
            if let Some(domain_filter) = &options.domain_filter {
                if !domain_filter.contains(&effect.domain_id) {
                    continue;
                }
            }
            
            // Create node label
            let mut label = format!("{} ({})", id, effect.effect_type);
            
            // Add domain info if requested
            if options.include_domains {
                label.push_str(&format!("\\nDomain: {}", effect.domain_id));
            }
            
            // Add parameter info if requested
            if options.include_parameters && !effect.parameters.is_empty() {
                label.push_str("\\nParams: ");
                let param_str: Vec<String> = effect.parameters.iter()
                    .map(|(k, v)| format!("{}={}", k, self.format_parameter_value(v, options.simplify)))
                    .collect();
                label.push_str(&param_str.join(", "));
            }
            
            // Add capability info if requested
            if options.include_capabilities && !effect.required_capabilities.is_empty() {
                label.push_str("\\nCapabilities: ");
                let cap_str = effect.required_capabilities.join(", ");
                label.push_str(&cap_str);
            }
            
            // Write node
            writeln!(output, "  \"{}\" [label=\"{}\"];", id, label)?;
        }
        
        // Export resource nodes if requested
        if options.include_resources {
            writeln!(output, "  node [shape=ellipse, style=filled, fillcolor=lightgreen];")?;
            
            for (id, resource) in &self.teg.resource_nodes {
                // Apply domain filter if present
                if let Some(domain_filter) = &options.domain_filter {
                    if !domain_filter.contains(&resource.domain_id) {
                        continue;
                    }
                }
                
                // Create node label
                let mut label = format!("{} ({})", id, resource.resource_type);
                
                // Add domain info if requested
                if options.include_domains {
                    label.push_str(&format!("\\nDomain: {}", resource.domain_id));
                }
                
                // Write node
                writeln!(output, "  \"{}\" [label=\"{}\"];", id, label)?;
            }
        }
        
        // Export effect dependencies
        for (to_id, dependencies) in &self.teg.effect_dependencies {
            // Apply domain filter if needed
            if let Some(domain_filter) = &options.domain_filter {
                if let Some(to_effect) = self.teg.effect_nodes.get(to_id) {
                    if !domain_filter.contains(&to_effect.domain_id) {
                        continue;
                    }
                }
            }
            
            for from_id in dependencies {
                // Apply domain filter if needed
                if let Some(domain_filter) = &options.domain_filter {
                    if let Some(from_effect) = self.teg.effect_nodes.get(from_id) {
                        if !domain_filter.contains(&from_effect.domain_id) {
                            continue;
                        }
                    }
                }
                
                writeln!(output, "  \"{}\" -> \"{}\" [style=dashed];", from_id, to_id)?;
            }
        }
        
        // Export effect continuations
        for (from_id, continuations) in &self.teg.effect_continuations {
            // Apply domain filter if needed
            if let Some(domain_filter) = &options.domain_filter {
                if let Some(from_effect) = self.teg.effect_nodes.get(from_id) {
                    if !domain_filter.contains(&from_effect.domain_id) {
                        continue;
                    }
                }
            }
            
            for (to_id, condition) in continuations {
                // Apply domain filter if needed
                if let Some(domain_filter) = &options.domain_filter {
                    if let Some(to_effect) = self.teg.effect_nodes.get(to_id) {
                        if !domain_filter.contains(&to_effect.domain_id) {
                            continue;
                        }
                    }
                }
                
                let label = if let Some(cond) = condition {
                    format!("{}", cond)
                } else {
                    "".to_string()
                };
                
                writeln!(output, "  \"{}\" -> \"{}\" [label=\"{}\"];", from_id, to_id, label)?;
            }
        }
        
        // Export resource access if requested
        if options.include_resources {
            writeln!(output, "  edge [color=green, style=dotted];")?;
            
            for (effect_id, effect) in &self.teg.effect_nodes {
                // Apply domain filter if needed
                if let Some(domain_filter) = &options.domain_filter {
                    if !domain_filter.contains(&effect.domain_id) {
                        continue;
                    }
                }
                
                for resource_id in &effect.resources_accessed {
                    // Apply domain filter if needed
                    if let Some(domain_filter) = &options.domain_filter {
                        if let Some(resource) = self.teg.resource_nodes.get(resource_id) {
                            if !domain_filter.contains(&resource.domain_id) {
                                continue;
                            }
                        }
                    }
                    
                    writeln!(output, "  \"{}\" -> \"{}\" [dir=both];", effect_id, resource_id)?;
                }
            }
        }
        
        // Close digraph
        writeln!(output, "}}")?;
        
        Ok(output)
    }
    
    /// Export TEG to Mermaid diagram format
    pub fn export_mermaid(&self, options: &ExportOptions) -> Result<String> {
        let mut output = String::new();
        
        // Start flowchart
        writeln!(output, "```mermaid")?;
        writeln!(output, "graph LR")?;
        
        // Export effect nodes
        for (id, effect) in &self.teg.effect_nodes {
            // Apply domain filter if present
            if let Some(domain_filter) = &options.domain_filter {
                if !domain_filter.contains(&effect.domain_id) {
                    continue;
                }
            }
            
            // Create node ID (sanitize for Mermaid)
            let node_id = format!("effect_{}", id.replace(|c: char| !c.is_alphanumeric(), "_"));
            
            // Create node label
            let mut label = format!("{} ({})", id, effect.effect_type);
            
            // Add domain info if requested
            if options.include_domains {
                label.push_str(&format!("<br>Domain: {}", effect.domain_id));
            }
            
            // Add parameter info if requested
            if options.include_parameters && !effect.parameters.is_empty() {
                label.push_str("<br>Params: ");
                let param_str: Vec<String> = effect.parameters.iter()
                    .map(|(k, v)| format!("{}={}", k, self.format_parameter_value(v, options.simplify)))
                    .collect();
                label.push_str(&param_str.join(", "));
            }
            
            // Write node
            writeln!(output, "    {}[\"{}\"]", node_id, label)?;
        }
        
        // Export resource nodes if requested
        if options.include_resources {
            for (id, resource) in &self.teg.resource_nodes {
                // Apply domain filter if present
                if let Some(domain_filter) = &options.domain_filter {
                    if !domain_filter.contains(&resource.domain_id) {
                        continue;
                    }
                }
                
                // Create node ID (sanitize for Mermaid)
                let node_id = format!("resource_{}", id.replace(|c: char| !c.is_alphanumeric(), "_"));
                
                // Create node label
                let mut label = format!("{} ({})", id, resource.resource_type);
                
                // Add domain info if requested
                if options.include_domains {
                    label.push_str(&format!("<br>Domain: {}", resource.domain_id));
                }
                
                // Write node
                writeln!(output, "    {}((\"{}\"))", node_id, label)?;
            }
        }
        
        // Export effect dependencies
        for (to_id, dependencies) in &self.teg.effect_dependencies {
            // Apply domain filter if needed
            if let Some(domain_filter) = &options.domain_filter {
                if let Some(to_effect) = self.teg.effect_nodes.get(to_id) {
                    if !domain_filter.contains(&to_effect.domain_id) {
                        continue;
                    }
                }
            }
            
            let to_node_id = format!("effect_{}", to_id.replace(|c: char| !c.is_alphanumeric(), "_"));
            
            for from_id in dependencies {
                // Apply domain filter if needed
                if let Some(domain_filter) = &options.domain_filter {
                    if let Some(from_effect) = self.teg.effect_nodes.get(from_id) {
                        if !domain_filter.contains(&from_effect.domain_id) {
                            continue;
                        }
                    }
                }
                
                let from_node_id = format!("effect_{}", from_id.replace(|c: char| !c.is_alphanumeric(), "_"));
                
                writeln!(output, "    {} -.-> {}", from_node_id, to_node_id)?;
            }
        }
        
        // Export effect continuations
        for (from_id, continuations) in &self.teg.effect_continuations {
            // Apply domain filter if needed
            if let Some(domain_filter) = &options.domain_filter {
                if let Some(from_effect) = self.teg.effect_nodes.get(from_id) {
                    if !domain_filter.contains(&from_effect.domain_id) {
                        continue;
                    }
                }
            }
            
            let from_node_id = format!("effect_{}", from_id.replace(|c: char| !c.is_alphanumeric(), "_"));
            
            for (to_id, condition) in continuations {
                // Apply domain filter if needed
                if let Some(domain_filter) = &options.domain_filter {
                    if let Some(to_effect) = self.teg.effect_nodes.get(to_id) {
                        if !domain_filter.contains(&to_effect.domain_id) {
                            continue;
                        }
                    }
                }
                
                let to_node_id = format!("effect_{}", to_id.replace(|c: char| !c.is_alphanumeric(), "_"));
                
                let label = if let Some(cond) = condition {
                    format!("{}", cond)
                } else {
                    "".to_string()
                };
                
                if label.is_empty() {
                    writeln!(output, "    {} --> {}", from_node_id, to_node_id)?;
                } else {
                    writeln!(output, "    {} -- \"{}\" --> {}", from_node_id, label, to_node_id)?;
                }
            }
        }
        
        // Export resource access if requested
        if options.include_resources {
            for (effect_id, effect) in &self.teg.effect_nodes {
                // Apply domain filter if needed
                if let Some(domain_filter) = &options.domain_filter {
                    if !domain_filter.contains(&effect.domain_id) {
                        continue;
                    }
                }
                
                let effect_node_id = format!("effect_{}", effect_id.replace(|c: char| !c.is_alphanumeric(), "_"));
                
                for resource_id in &effect.resources_accessed {
                    // Apply domain filter if needed
                    if let Some(domain_filter) = &options.domain_filter {
                        if let Some(resource) = self.teg.resource_nodes.get(resource_id) {
                            if !domain_filter.contains(&resource.domain_id) {
                                continue;
                            }
                        }
                    }
                    
                    let resource_node_id = format!("resource_{}", resource_id.replace(|c: char| !c.is_alphanumeric(), "_"));
                    
                    writeln!(output, "    {} -.- {}", effect_node_id, resource_node_id)?;
                }
            }
        }
        
        // Close flowchart
        writeln!(output, "```")?;
        
        Ok(output)
    }
    
    /// Export TEG to JSON format
    pub fn export_json(&self, options: &ExportOptions) -> Result<String> {
        let serializable = self.create_serializable_teg(options)?;
        Ok(serde_json::to_string_pretty(&serializable)?)
    }
    
    /// Export TEG to YAML format
    pub fn export_yaml(&self, options: &ExportOptions) -> Result<String> {
        let serializable = self.create_serializable_teg(options)?;
        Ok(serde_yaml::to_string(&serializable)?)
    }
    
    /// Export TEG to a custom format
    pub fn export_custom(&self, format_name: &str, options: &ExportOptions) -> Result<String> {
        match format_name {
            "cytoscape" => self.export_cytoscape(options),
            "d3" => self.export_d3(options),
            _ => Err(anyhow!("Unsupported custom format: {}", format_name)),
        }
    }
    
    /// Export TEG to Cytoscape.js format
    fn export_cytoscape(&self, options: &ExportOptions) -> Result<String> {
        let mut elements = serde_json::json!({
            "nodes": [],
            "edges": []
        });
        
        let nodes = elements["nodes"].as_array_mut().unwrap();
        let edges = elements["edges"].as_array_mut().unwrap();
        
        // Add effect nodes
        for (id, effect) in &self.teg.effect_nodes {
            // Apply domain filter if present
            if let Some(domain_filter) = &options.domain_filter {
                if !domain_filter.contains(&effect.domain_id) {
                    continue;
                }
            }
            
            let mut data = serde_json::json!({
                "id": id,
                "label": format!("{} ({})", id, effect.effect_type),
                "type": "effect",
                "effectType": effect.effect_type,
                "domain": effect.domain_id
            });
            
            // Add parameters if requested
            if options.include_parameters {
                let params = serde_json::json!(effect.parameters);
                data["parameters"] = params;
            }
            
            nodes.push(serde_json::json!({
                "data": data,
                "classes": "effect"
            }));
        }
        
        // Add resource nodes if requested
        if options.include_resources {
            for (id, resource) in &self.teg.resource_nodes {
                // Apply domain filter if present
                if let Some(domain_filter) = &options.domain_filter {
                    if !domain_filter.contains(&resource.domain_id) {
                        continue;
                    }
                }
                
                nodes.push(serde_json::json!({
                    "data": {
                        "id": id,
                        "label": format!("{} ({})", id, resource.resource_type),
                        "type": "resource",
                        "resourceType": resource.resource_type,
                        "domain": resource.domain_id
                    },
                    "classes": "resource"
                }));
            }
        }
        
        // Add dependency edges
        for (to_id, dependencies) in &self.teg.effect_dependencies {
            // Apply domain filter if needed
            if let Some(domain_filter) = &options.domain_filter {
                if let Some(to_effect) = self.teg.effect_nodes.get(to_id) {
                    if !domain_filter.contains(&to_effect.domain_id) {
                        continue;
                    }
                }
            }
            
            for from_id in dependencies {
                // Apply domain filter if needed
                if let Some(domain_filter) = &options.domain_filter {
                    if let Some(from_effect) = self.teg.effect_nodes.get(from_id) {
                        if !domain_filter.contains(&from_effect.domain_id) {
                            continue;
                        }
                    }
                }
                
                edges.push(serde_json::json!({
                    "data": {
                        "id": format!("dep_{}_{}", from_id, to_id),
                        "source": from_id,
                        "target": to_id,
                        "type": "dependency"
                    },
                    "classes": "dependency"
                }));
            }
        }
        
        // Add continuation edges
        for (from_id, continuations) in &self.teg.effect_continuations {
            // Apply domain filter if needed
            if let Some(domain_filter) = &options.domain_filter {
                if let Some(from_effect) = self.teg.effect_nodes.get(from_id) {
                    if !domain_filter.contains(&from_effect.domain_id) {
                        continue;
                    }
                }
            }
            
            for (to_id, condition) in continuations {
                // Apply domain filter if needed
                if let Some(domain_filter) = &options.domain_filter {
                    if let Some(to_effect) = self.teg.effect_nodes.get(to_id) {
                        if !domain_filter.contains(&to_effect.domain_id) {
                            continue;
                        }
                    }
                }
                
                let mut edge_data = serde_json::json!({
                    "id": format!("cont_{}_{}", from_id, to_id),
                    "source": from_id,
                    "target": to_id,
                    "type": "continuation"
                });
                
                if let Some(cond) = condition {
                    edge_data["condition"] = serde_json::json!(cond.to_string());
                }
                
                edges.push(serde_json::json!({
                    "data": edge_data,
                    "classes": "continuation"
                }));
            }
        }
        
        // Add resource access edges if requested
        if options.include_resources {
            for (effect_id, effect) in &self.teg.effect_nodes {
                // Apply domain filter if needed
                if let Some(domain_filter) = &options.domain_filter {
                    if !domain_filter.contains(&effect.domain_id) {
                        continue;
                    }
                }
                
                for resource_id in &effect.resources_accessed {
                    // Apply domain filter if needed
                    if let Some(domain_filter) = &options.domain_filter {
                        if let Some(resource) = self.teg.resource_nodes.get(resource_id) {
                            if !domain_filter.contains(&resource.domain_id) {
                                continue;
                            }
                        }
                    }
                    
                    edges.push(serde_json::json!({
                        "data": {
                            "id": format!("access_{0}_{1}", effect_id, resource_id),
                            "source": effect_id,
                            "target": resource_id,
                            "type": "resource_access"
                        },
                        "classes": "resource_access"
                    }));
                }
            }
        }
        
        Ok(serde_json::to_string_pretty(&elements)?)
    }
    
    /// Export TEG to D3.js format
    fn export_d3(&self, options: &ExportOptions) -> Result<String> {
        let mut result = serde_json::json!({
            "nodes": [],
            "links": []
        });
        
        let nodes = result["nodes"].as_array_mut().unwrap();
        let links = result["links"].as_array_mut().unwrap();
        
        // Similar implementation to Cytoscape but with D3's expected format
        // ...
        
        Ok(serde_json::to_string_pretty(&result)?)
    }
    
    /// Create a serializable representation of the TEG based on export options
    fn create_serializable_teg(&self, options: &ExportOptions) -> Result<serde_json::Value> {
        let mut result = serde_json::json!({
            "effects": {},
            "resources": {},
            "dependencies": {},
            "continuations": {},
            "resource_relationships": {}
        });
        
        // Add effect nodes
        let effects = result["effects"].as_object_mut().unwrap();
        
        for (id, effect) in &self.teg.effect_nodes {
            // Apply domain filter if present
            if let Some(domain_filter) = &options.domain_filter {
                if !domain_filter.contains(&effect.domain_id) {
                    continue;
                }
            }
            
            let mut effect_json = serde_json::json!({
                "id": id,
                "effect_type": effect.effect_type,
                "domain": effect.domain_id
            });
            
            // Add parameters if requested
            if options.include_parameters {
                effect_json["parameters"] = self.serialize_parameters(&effect.parameters, options.simplify)?;
            }
            
            // Add capabilities if requested
            if options.include_capabilities {
                effect_json["required_capabilities"] = serde_json::json!(effect.required_capabilities);
            }
            
            // Add metadata if requested
            if options.include_metadata {
                // Add placeholder for metadata
                effect_json["metadata"] = serde_json::json!({});
            }
            
            effects[id] = effect_json;
        }
        
        // Add resource nodes if requested
        if options.include_resources {
            let resources = result["resources"].as_object_mut().unwrap();
            
            for (id, resource) in &self.teg.resource_nodes {
                // Apply domain filter if present
                if let Some(domain_filter) = &options.domain_filter {
                    if !domain_filter.contains(&resource.domain_id) {
                        continue;
                    }
                }
                
                let mut resource_json = serde_json::json!({
                    "id": id,
                    "resource_type": resource.resource_type,
                    "domain": resource.domain_id
                });
                
                // Add metadata if requested
                if options.include_metadata {
                    resource_json["metadata"] = serde_json::json!(resource.metadata);
                }
                
                resources[id] = resource_json;
            }
        }
        
        // Add dependencies
        let dependencies = result["dependencies"].as_object_mut().unwrap();
        
        for (to_id, deps) in &self.teg.effect_dependencies {
            // Apply domain filter if needed
            if let Some(domain_filter) = &options.domain_filter {
                if let Some(to_effect) = self.teg.effect_nodes.get(to_id) {
                    if !domain_filter.contains(&to_effect.domain_id) {
                        continue;
                    }
                }
            }
            
            // Filter dependencies based on domain filter
            let filtered_deps = if let Some(domain_filter) = &options.domain_filter {
                deps.iter()
                    .filter(|dep_id| {
                        if let Some(dep_effect) = self.teg.effect_nodes.get(*dep_id) {
                            domain_filter.contains(&dep_effect.domain_id)
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                deps.clone()
            };
            
            if !filtered_deps.is_empty() {
                dependencies[to_id] = serde_json::json!(filtered_deps);
            }
        }
        
        // Add continuations
        let continuations = result["continuations"].as_object_mut().unwrap();
        
        for (from_id, conts) in &self.teg.effect_continuations {
            // Apply domain filter if needed
            if let Some(domain_filter) = &options.domain_filter {
                if let Some(from_effect) = self.teg.effect_nodes.get(from_id) {
                    if !domain_filter.contains(&from_effect.domain_id) {
                        continue;
                    }
                }
            }
            
            // Filter continuations based on domain filter
            let filtered_conts = if let Some(domain_filter) = &options.domain_filter {
                conts.iter()
                    .filter(|(to_id, _)| {
                        if let Some(to_effect) = self.teg.effect_nodes.get(to_id) {
                            domain_filter.contains(&to_effect.domain_id)
                        } else {
                            false
                        }
                    })
                    .map(|(to_id, condition)| {
                        (to_id.clone(), condition.clone().map(|c| c.to_string()))
                    })
                    .collect::<Vec<_>>()
            } else {
                conts.iter()
                    .map(|(to_id, condition)| {
                        (to_id.clone(), condition.clone().map(|c| c.to_string()))
                    })
                    .collect::<Vec<_>>()
            };
            
            if !filtered_conts.is_empty() {
                continuations[from_id] = serde_json::json!(filtered_conts);
            }
        }
        
        // Add resource relationships if requested
        if options.include_resources {
            let relationships = result["resource_relationships"].as_object_mut().unwrap();
            
            for (from_id, rels) in &self.teg.resource_relationships {
                // Apply domain filter if needed
                if let Some(domain_filter) = &options.domain_filter {
                    if let Some(from_resource) = self.teg.resource_nodes.get(from_id) {
                        if !domain_filter.contains(&from_resource.domain_id) {
                            continue;
                        }
                    }
                }
                
                // Filter relationships based on domain filter
                let filtered_rels = if let Some(domain_filter) = &options.domain_filter {
                    rels.iter()
                        .filter(|(to_id, _)| {
                            if let Some(to_resource) = self.teg.resource_nodes.get(to_id) {
                                domain_filter.contains(&to_resource.domain_id)
                            } else {
                                false
                            }
                        })
                        .map(|(to_id, rel_type)| {
                            (to_id.clone(), format!("{:?}", rel_type))
                        })
                        .collect::<Vec<_>>()
                } else {
                    rels.iter()
                        .map(|(to_id, rel_type)| {
                            (to_id.clone(), format!("{:?}", rel_type))
                        })
                        .collect::<Vec<_>>()
                };
                
                if !filtered_rels.is_empty() {
                    relationships[from_id] = serde_json::json!(filtered_rels);
                }
            }
        }
        
        Ok(result)
    }
    
    /// Format a parameter value for display
    fn format_parameter_value(&self, value: &ParameterValue, simplify: bool) -> String {
        match value {
            ParameterValue::String(s) => {
                if simplify && s.len() > 10 {
                    format!("\"{}...\"", &s[0..7])
                } else {
                    format!("\"{}\"", s)
                }
            }
            ParameterValue::Integer(i) => i.to_string(),
            ParameterValue::Float(f) => f.to_string(),
            ParameterValue::Boolean(b) => b.to_string(),
            ParameterValue::Bytes(b) => {
                if simplify {
                    format!("bytes[{}]", b.len())
                } else {
                    format!("{:?}", b)
                }
            }
            ParameterValue::Array(a) => {
                if simplify {
                    format!("array[{}]", a.len())
                } else {
                    let items: Vec<String> = a.iter()
                        .map(|v| self.format_parameter_value(v, simplify))
                        .collect();
                    format!("[{}]", items.join(", "))
                }
            }
            ParameterValue::Object(o) => {
                if simplify {
                    format!("object{{{}}}", o.len())
                } else {
                    let fields: Vec<String> = o.iter()
                        .map(|(k, v)| format!("{}:{}", k, self.format_parameter_value(v, simplify)))
                        .collect();
                    format!("{{{}}}", fields.join(", "))
                }
            }
            ParameterValue::Null => "null".to_string(),
        }
    }
    
    /// Serialize parameters to JSON
    fn serialize_parameters(&self, params: &HashMap<String, ParameterValue>, simplify: bool) -> Result<serde_json::Value> {
        if simplify {
            // Create simplified version with just keys and simplified values
            let mut result = serde_json::Map::new();
            for (key, value) in params {
                result.insert(key.clone(), serde_json::Value::String(self.format_parameter_value(value, true)));
            }
            Ok(serde_json::Value::Object(result))
        } else {
            // Use full serialization
            Ok(serde_json::to_value(params)?)
        }
    }
}

/// Create a new TEG exporter
pub fn create_exporter(teg: &TemporalEffectGraph) -> TEGExporter {
    TEGExporter::new(teg)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dot_export() {
        let teg = TemporalEffectGraph::new();
        let exporter = TEGExporter::new(&teg);
        let options = ExportOptions::default();
        
        let dot = exporter.export_dot(&options).unwrap();
        assert!(dot.starts_with("digraph TemporalEffectGraph {"));
        assert!(dot.ends_with("}\n"));
    }
    
    #[test]
    fn test_mermaid_export() {
        let teg = TemporalEffectGraph::new();
        let exporter = TEGExporter::new(&teg);
        let options = ExportOptions::default();
        
        let mermaid = exporter.export_mermaid(&options).unwrap();
        assert!(mermaid.starts_with("```mermaid"));
        assert!(mermaid.ends_with("```\n"));
    }
} 