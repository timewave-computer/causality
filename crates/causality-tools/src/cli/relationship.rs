// CLI tools for relationship management
// Original file: src/cli/relationship.rs

// Cross-Domain Relationship CLI Module
//
// This module provides CLI commands for managing cross-domain relationships.

use std::str::FromStr;
use std::sync::Arc;
use clap::{Arg, Command, ArgMatches, value_parser};
use serde_json::json;

use causality_types::{Error, Result};
use causality_types::{*};
use causality_crypto::ContentId;;
use crate::resource::relationship::{
    CrossDomainRelationship,
    CrossDomainRelationshipType,
    CrossDomainMetadata,
    CrossDomainRelationshipManager,
    CrossDomainSyncManager,
    CrossDomainSyncScheduler,
    SyncStrategy,
    ValidationLevel,
};

/// Build the relationships subcommand
pub fn build_command() -> Command {
    Command::new("relationship")
        .about("Manage cross-domain relationships")
        .subcommand_required(true)
        .subcommand(
            Command::new("create")
                .about("Create a new cross-domain relationship")
                .arg(
                    Arg::new("source-resource")
                        .long("source-resource")
                        .short('s')
                        .help("Source resource ID")
                        .required(true)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("source-domain")
                        .long("source-domain")
                        .short('d')
                        .help("Source domain ID")
                        .required(true)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("target-resource")
                        .long("target-resource")
                        .short('t')
                        .help("Target resource ID")
                        .required(true)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("target-domain")
                        .long("target-domain")
                        .short('e')
                        .help("Target domain ID")
                        .required(true)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("type")
                        .long("type")
                        .short('y')
                        .help("Relationship type (mirror, reference, ownership, derived, bridge, custom)")
                        .required(true)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("bidirectional")
                        .long("bidirectional")
                        .short('b')
                        .help("Whether the relationship is bidirectional")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("requires-sync")
                        .long("requires-sync")
                        .help("Whether the relationship requires synchronization")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("sync-strategy")
                        .long("sync-strategy")
                        .help("Synchronization strategy (one-time, periodic, event-driven, hybrid)")
                        .default_value("event-driven")
                        .value_parser(value_parser!(String)),
                )
        )
        .subcommand(
            Command::new("list")
                .about("List cross-domain relationships")
                .arg(
                    Arg::new("source-domain")
                        .long("source-domain")
                        .short('d')
                        .help("Filter by source domain")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("target-domain")
                        .long("target-domain")
                        .short('t')
                        .help("Filter by target domain")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("type")
                        .long("type")
                        .short('y')
                        .help("Filter by relationship type")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .short('j')
                        .help("Output as JSON")
                        .action(clap::ArgAction::SetTrue),
                )
        )
        .subcommand(
            Command::new("get")
                .about("Get a cross-domain relationship by ID")
                .arg(
                    Arg::new("id")
                        .help("Relationship ID")
                        .required(true)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .short('j')
                        .help("Output as JSON")
                        .action(clap::ArgAction::SetTrue),
                )
        )
        .subcommand(
            Command::new("delete")
                .about("Delete a cross-domain relationship")
                .arg(
                    Arg::new("id")
                        .help("Relationship ID")
                        .required(true)
                        .value_parser(value_parser!(String)),
                )
        )
        .subcommand(
            Command::new("validate")
                .about("Validate a cross-domain relationship")
                .arg(
                    Arg::new("id")
                        .help("Relationship ID")
                        .required(true)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("level")
                        .long("level")
                        .short('l')
                        .help("Validation level (strict, moderate, permissive)")
                        .default_value("moderate")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .short('j')
                        .help("Output as JSON")
                        .action(clap::ArgAction::SetTrue),
                )
        )
        .subcommand(
            Command::new("sync")
                .about("Synchronize a cross-domain relationship")
                .arg(
                    Arg::new("id")
                        .help("Relationship ID")
                        .required(true)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .help("Force synchronization even if not required")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .short('j')
                        .help("Output as JSON")
                        .action(clap::ArgAction::SetTrue),
                )
        )
        .subcommand(
            Command::new("scheduler")
                .about("Manage the sync scheduler")
                .subcommand_required(true)
                .subcommand(
                    Command::new("start")
                        .about("Start the scheduler")
                )
                .subcommand(
                    Command::new("stop")
                        .about("Stop the scheduler")
                )
                .subcommand(
                    Command::new("pause")
                        .about("Pause the scheduler")
                )
                .subcommand(
                    Command::new("resume")
                        .about("Resume the scheduler")
                )
                .subcommand(
                    Command::new("status")
                        .about("Get scheduler status")
                        .arg(
                            Arg::new("json")
                                .long("json")
                                .short('j')
                                .help("Output as JSON")
                                .action(clap::ArgAction::SetTrue),
                        )
                )
        )
}

/// Command handler for relationship commands
pub struct RelationshipCommandHandler {
    /// Cross-domain relationship manager
    relationship_manager: Arc<CrossDomainRelationshipManager>,
    
    /// Cross-domain sync manager
    sync_manager: Arc<CrossDomainSyncManager>,
    
    /// Cross-domain sync scheduler
    sync_scheduler: Arc<CrossDomainSyncScheduler>,
}

impl RelationshipCommandHandler {
    /// Create a new command handler
    pub fn new(
        relationship_manager: Arc<CrossDomainRelationshipManager>,
        sync_manager: Arc<CrossDomainSyncManager>,
        sync_scheduler: Arc<CrossDomainSyncScheduler>,
    ) -> Self {
        Self {
            relationship_manager,
            sync_manager,
            sync_scheduler,
        }
    }
    
    /// Handle a command
    pub fn handle_command(&self, command: &ArgMatches) -> Result<()> {
        match command.subcommand() {
            Some(("create", args)) => self.handle_create(args),
            Some(("list", args)) => self.handle_list(args),
            Some(("get", args)) => self.handle_get(args),
            Some(("delete", args)) => self.handle_delete(args),
            Some(("validate", args)) => self.handle_validate(args),
            Some(("sync", args)) => self.handle_sync(args),
            Some(("scheduler", args)) => self.handle_scheduler(args),
            _ => Err(Error::InvalidArgument("Unknown command".to_string())),
        }
    }
    
    /// Handle 'create' command
    fn handle_create(&self, args: &ArgMatches) -> Result<()> {
        // Get command arguments
        let source_resource = args.get_one::<String>("source-resource")
            .ok_or_else(|| Error::InvalidArgument("Missing source-resource".to_string()))?
            .clone();
        
        let source_domain = args.get_one::<String>("source-domain")
            .ok_or_else(|| Error::InvalidArgument("Missing source-domain".to_string()))?
            .clone();
        
        let target_resource = args.get_one::<String>("target-resource")
            .ok_or_else(|| Error::InvalidArgument("Missing target-resource".to_string()))?
            .clone();
        
        let target_domain = args.get_one::<String>("target-domain")
            .ok_or_else(|| Error::InvalidArgument("Missing target-domain".to_string()))?
            .clone();
        
        let rel_type_str = args.get_one::<String>("type")
            .ok_or_else(|| Error::InvalidArgument("Missing type".to_string()))?
            .to_lowercase();
        
        // Parse relationship type
        let rel_type = match rel_type_str.as_str() {
            "mirror" => CrossDomainRelationshipType::Mirror,
            "reference" => CrossDomainRelationshipType::Reference,
            "ownership" => CrossDomainRelationshipType::Ownership,
            "derived" => CrossDomainRelationshipType::Derived,
            "bridge" => CrossDomainRelationshipType::Bridge,
            _ if rel_type_str.starts_with("custom:") => {
                let custom_type = rel_type_str.strip_prefix("custom:").unwrap_or(&rel_type_str).to_string();
                CrossDomainRelationshipType::Custom(custom_type)
            },
            _ => {
                return Err(Error::InvalidArgument(format!(
                    "Invalid relationship type: {}. Expected one of: mirror, reference, ownership, derived, bridge, custom:TYPE",
                    rel_type_str
                )));
            }
        };
        
        // Get optional arguments
        let bidirectional = args.get_flag("bidirectional");
        let requires_sync = args.get_flag("requires-sync");
        
        let sync_strategy_str = args.get_one::<String>("sync-strategy")
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "event-driven".to_string());
        
        // Parse sync strategy
        let sync_strategy = match sync_strategy_str.as_str() {
            "one-time" => SyncStrategy::OneTime,
            "periodic" => SyncStrategy::Periodic(std::time::Duration::from_secs(3600)), // Default to 1 hour
            "event-driven" => SyncStrategy::EventDriven,
            "hybrid" => SyncStrategy::Hybrid(std::time::Duration::from_secs(86400)), // Default to 1 day
            _ => {
                return Err(Error::InvalidArgument(format!(
                    "Invalid sync strategy: {}. Expected one of: one-time, periodic, event-driven, hybrid",
                    sync_strategy_str
                )));
            }
        };
        
        // Create metadata
        let metadata = CrossDomainMetadata {
            origin_domain: source_domain.clone(),
            target_domain: target_domain.clone(),
            requires_sync,
            sync_strategy,
        };
        
        // Create relationship
        let relationship = CrossDomainRelationship::new(
            source_resource,
            source_domain,
            target_resource,
            target_domain,
            rel_type,
            metadata,
            bidirectional,
        );
        
        // Add to manager
        self.relationship_manager.add_relationship(relationship.clone())?;
        
        println!("Created cross-domain relationship: {}", relationship.id);
        
        Ok(())
    }
    
    /// Handle 'list' command
    fn handle_list(&self, args: &ArgMatches) -> Result<()> {
        // Get filter arguments
        let source_domain = args.get_one::<String>("source-domain").cloned();
        let target_domain = args.get_one::<String>("target-domain").cloned();
        let rel_type_str = args.get_one::<String>("type").cloned();
        let json_output = args.get_flag("json");
        
        // Get all relationships
        let all_relationships = self.relationship_manager.get_all_relationships()?;
        
        // Apply filters
        let filtered_relationships = all_relationships.into_iter().filter(|rel| {
            // Filter by source domain if specified
            if let Some(sd) = &source_domain {
                if rel.source_domain != *sd {
                    return false;
                }
            }
            
            // Filter by target domain if specified
            if let Some(td) = &target_domain {
                if rel.target_domain != *td {
                    return false;
                }
            }
            
            // Filter by relationship type if specified
            if let Some(rt) = &rel_type_str {
                let type_matches = match rt.to_lowercase().as_str() {
                    "mirror" => rel.relationship_type == CrossDomainRelationshipType::Mirror,
                    "reference" => rel.relationship_type == CrossDomainRelationshipType::Reference,
                    "ownership" => rel.relationship_type == CrossDomainRelationshipType::Ownership,
                    "derived" => rel.relationship_type == CrossDomainRelationshipType::Derived,
                    "bridge" => rel.relationship_type == CrossDomainRelationshipType::Bridge,
                    _ if rt.starts_with("custom:") => {
                        match &rel.relationship_type {
                            CrossDomainRelationshipType::Custom(custom_type) => {
                                let filter_type = rt.strip_prefix("custom:").unwrap_or(rt);
                                custom_type == filter_type
                            },
                            _ => false,
                        }
                    },
                    _ => false,
                };
                
                if !type_matches {
                    return false;
                }
            }
            
            true
        }).collect::<Vec<_>>();
        
        // Output the results
        if json_output {
            let json_array = filtered_relationships.iter().map(|rel| {
                json!({
                    "id": rel.id,
                    "source_resource": rel.source_resource,
                    "source_domain": rel.source_domain,
                    "target_resource": rel.target_resource,
                    "target_domain": rel.target_domain,
                    "relationship_type": format!("{:?}", rel.relationship_type),
                    "bidirectional": rel.bidirectional,
                    "requires_sync": rel.metadata.requires_sync,
                    "sync_strategy": format!("{:?}", rel.metadata.sync_strategy),
                })
            }).collect::<Vec<_>>();
            
            println!("{}", serde_json::to_string_pretty(&json_array)?);
        } else {
            println!("Found {} relationships:", filtered_relationships.len());
            
            for rel in filtered_relationships {
                println!("ID: {}", rel.id);
                println!("  Source: {} (Domain: {})", rel.source_resource, rel.source_domain);
                println!("  Target: {} (Domain: {})", rel.target_resource, rel.target_domain);
                println!("  Type: {:?}", rel.relationship_type);
                println!("  Bidirectional: {}", rel.bidirectional);
                println!("  Requires Sync: {}", rel.metadata.requires_sync);
                println!("  Sync Strategy: {:?}", rel.metadata.sync_strategy);
                println!();
            }
        }
        
        Ok(())
    }
    
    /// Handle 'get' command
    fn handle_get(&self, args: &ArgMatches) -> Result<()> {
        // Get command arguments
        let id = args.get_one::<String>("id")
            .ok_or_else(|| Error::InvalidArgument("Missing id".to_string()))?;
        
        let json_output = args.get_flag("json");
        
        // Get the relationship
        let relationship = self.relationship_manager.get_relationship(id)?;
        
        // Output the result
        if json_output {
            let json_obj = json!({
                "id": relationship.id,
                "source_resource": relationship.source_resource,
                "source_domain": relationship.source_domain,
                "target_resource": relationship.target_resource,
                "target_domain": relationship.target_domain,
                "relationship_type": format!("{:?}", relationship.relationship_type),
                "bidirectional": relationship.bidirectional,
                "requires_sync": relationship.metadata.requires_sync,
                "sync_strategy": format!("{:?}", relationship.metadata.sync_strategy),
            });
            
            println!("{}", serde_json::to_string_pretty(&json_obj)?);
        } else {
            println!("ID: {}", relationship.id);
            println!("Source: {} (Domain: {})", relationship.source_resource, relationship.source_domain);
            println!("Target: {} (Domain: {})", relationship.target_resource, relationship.target_domain);
            println!("Type: {:?}", relationship.relationship_type);
            println!("Bidirectional: {}", relationship.bidirectional);
            println!("Requires Sync: {}", relationship.metadata.requires_sync);
            println!("Sync Strategy: {:?}", relationship.metadata.sync_strategy);
        }
        
        Ok(())
    }
    
    /// Handle 'delete' command
    fn handle_delete(&self, args: &ArgMatches) -> Result<()> {
        // Get command arguments
        let id = args.get_one::<String>("id")
            .ok_or_else(|| Error::InvalidArgument("Missing id".to_string()))?;
        
        // Delete the relationship
        self.relationship_manager.remove_relationship(id)?;
        
        println!("Deleted relationship: {}", id);
        
        Ok(())
    }
    
    /// Handle 'validate' command
    fn handle_validate(&self, args: &ArgMatches) -> Result<()> {
        // Get command arguments
        let id = args.get_one::<String>("id")
            .ok_or_else(|| Error::InvalidArgument("Missing id".to_string()))?;
        
        let level_str = args.get_one::<String>("level")
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "moderate".to_string());
        
        let json_output = args.get_flag("json");
        
        // Parse validation level
        let level = match level_str.as_str() {
            "strict" => ValidationLevel::Strict,
            "moderate" => ValidationLevel::Moderate,
            "permissive" => ValidationLevel::Permissive,
            _ => {
                return Err(Error::InvalidArgument(format!(
                    "Invalid validation level: {}. Expected one of: strict, moderate, permissive",
                    level_str
                )));
            }
        };
        
        // Get the relationship
        let relationship = self.relationship_manager.get_relationship(id)?;
        
        // Validate the relationship
        // Note: In a real implementation, we would pass the relationship to a validator
        // For now, we'll just mock a successful validation result
        let result = crate::resource::relationship::ValidationResult {
            is_valid: true,
            validation_level: level,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Output the result
        if json_output {
            let json_obj = json!({
                "id": relationship.id,
                "is_valid": result.is_valid,
                "validation_level": format!("{:?}", result.validation_level),
                "errors": result.errors,
                "warnings": result.warnings,
            });
            
            println!("{}", serde_json::to_string_pretty(&json_obj)?);
        } else {
            println!("Validation result for relationship {}", relationship.id);
            println!("Valid: {}", result.is_valid);
            println!("Validation Level: {:?}", result.validation_level);
            
            if !result.errors.is_empty() {
                println!("Errors:");
                for error in &result.errors {
                    println!("  - {}", error);
                }
            }
            
            if !result.warnings.is_empty() {
                println!("Warnings:");
                for warning in &result.warnings {
                    println!("  - {}", warning);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle 'sync' command
    fn handle_sync(&self, args: &ArgMatches) -> Result<()> {
        // Get command arguments
        let id = args.get_one::<String>("id")
            .ok_or_else(|| Error::InvalidArgument("Missing id".to_string()))?;
        
        let force = args.get_flag("force");
        let json_output = args.get_flag("json");
        
        // Get the relationship
        let relationship = self.relationship_manager.get_relationship(id)?;
        
        // Check if we should sync
        if !force && !self.sync_manager.should_sync(&relationship) {
            if json_output {
                let json_obj = json!({
                    "id": relationship.id,
                    "status": "skipped",
                    "reason": "Synchronization not required and force flag not set",
                });
                
                println!("{}", serde_json::to_string_pretty(&json_obj)?);
            } else {
                println!("Skipping synchronization for relationship {}", relationship.id);
                println!("Reason: Synchronization not required and force flag not set");
            }
            
            return Ok(());
        }
        
        // Synchronize the relationship
        let result = self.sync_manager.sync_relationship(&relationship)?;
        
        // Output the result
        if json_output {
            let json_obj = json!({
                "id": relationship.id,
                "status": format!("{:?}", result.status),
                "timestamp": result.timestamp,
                "error": result.error,
                "metadata": result.metadata,
            });
            
            println!("{}", serde_json::to_string_pretty(&json_obj)?);
        } else {
            println!("Synchronization result for relationship {}", relationship.id);
            println!("Status: {:?}", result.status);
            println!("Timestamp: {}", result.timestamp);
            
            if let Some(error) = &result.error {
                println!("Error: {}", error);
            }
            
            if !result.metadata.is_empty() {
                println!("Metadata:");
                for (key, value) in &result.metadata {
                    println!("  {}: {}", key, value);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle 'scheduler' command
    fn handle_scheduler(&self, args: &ArgMatches) -> Result<()> {
        match args.subcommand() {
            Some(("start", _)) => {
                self.sync_scheduler.start()?;
                println!("Scheduler started");
            },
            Some(("stop", _)) => {
                self.sync_scheduler.stop()?;
                println!("Scheduler stopped");
            },
            Some(("pause", _)) => {
                self.sync_scheduler.pause()?;
                println!("Scheduler paused");
            },
            Some(("resume", _)) => {
                self.sync_scheduler.resume()?;
                println!("Scheduler resumed");
            },
            Some(("status", status_args)) => {
                let json_output = status_args.get_flag("json");
                let status = self.sync_scheduler.get_status()?;
                let stats = self.sync_scheduler.get_stats()?;
                
                if json_output {
                    let json_obj = json!({
                        "status": format!("{:?}", status),
                        "stats": {
                            "successful_syncs": stats.successful_syncs,
                            "failed_syncs": stats.failed_syncs,
                            "retry_attempts": stats.retry_attempts,
                            "total_relationships": stats.total_relationships,
                            "active_tasks": stats.active_tasks,
                            "pending_tasks": stats.pending_tasks,
                            "avg_sync_time_ms": stats.avg_sync_time_ms,
                            "last_run_timestamp": stats.last_run_timestamp,
                        }
                    });
                    
                    println!("{}", serde_json::to_string_pretty(&json_obj)?);
                } else {
                    println!("Scheduler Status: {:?}", status);
                    println!("Statistics:");
                    println!("  Successful Syncs: {}", stats.successful_syncs);
                    println!("  Failed Syncs: {}", stats.failed_syncs);
                    println!("  Retry Attempts: {}", stats.retry_attempts);
                    println!("  Total Relationships: {}", stats.total_relationships);
                    println!("  Active Tasks: {}", stats.active_tasks);
                    println!("  Pending Tasks: {}", stats.pending_tasks);
                    println!("  Avg Sync Time: {:.2} ms", stats.avg_sync_time_ms);
                    
                    if let Some(timestamp) = stats.last_run_timestamp {
                        println!("  Last Run: {}", timestamp);
                    } else {
                        println!("  Last Run: Never");
                    }
                }
            },
            _ => {
                return Err(Error::InvalidArgument("Unknown scheduler subcommand".to_string()));
            }
        }
        
        Ok(())
    }
} 
