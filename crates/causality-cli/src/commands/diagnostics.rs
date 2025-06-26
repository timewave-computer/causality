//! Type checker diagnostics and development tools
//! 
//! This module provides comprehensive diagnostics for type checking,
//! linear resource analysis, and compilation issues to aid development.

use anyhow::Result;
use causality_compiler::{compile, error::CompileError};
use causality_core::{
    lambda::{Term, TermKind},
    machine::{Instruction, RegisterId},
};
use std::collections::{BTreeMap, BTreeSet};

/// Diagnostic information for type checking and resource analysis
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub source: String,
    pub type_errors: Vec<TypeError>,
    pub linearity_warnings: Vec<LinearityWarning>,
    pub resource_usage: ResourceUsageAnalysis,
    pub compilation_summary: CompilationSummary,
}

/// Type error with location and suggested fixes
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub location: Option<(usize, usize)>, // line, column
    pub expected_type: Option<String>,
    pub actual_type: Option<String>,
    pub suggestion: Option<String>,
}

/// Linear resource usage warning
#[derive(Debug, Clone)]
pub struct LinearityWarning {
    pub message: String,
    pub resource: String,
    pub warning_type: LinearityWarningType,
    pub location: Option<(usize, usize)>,
    pub suggestion: String,
}

/// Types of linearity warnings
#[derive(Debug, Clone)]
pub enum LinearityWarningType {
    UnusedResource,
    MultipleConsumption,
    ResourceLeak,
    PotentialUseAfterConsume,
}

/// Resource usage analysis
#[derive(Debug, Clone)]
pub struct ResourceUsageAnalysis {
    pub total_allocations: usize,
    pub total_consumptions: usize,
    pub live_resources: Vec<String>,
    pub resource_lifetime_graph: Vec<ResourceLifetime>,
}

/// Resource lifetime tracking
#[derive(Debug, Clone)]
pub struct ResourceLifetime {
    pub resource_id: String,
    pub allocated_at: usize, // instruction index
    pub consumed_at: Option<usize>,
    pub status: ResourceStatus,
}

/// Resource status tracking
#[derive(Debug, Clone)]
pub enum ResourceStatus {
    Allocated,
    Consumed,
    Leaked,
}

/// Compilation summary with performance metrics
#[derive(Debug, Clone)]
pub struct CompilationSummary {
    pub total_instructions: usize,
    pub register_count: usize,
    pub estimated_gas_cost: u64,
    pub compilation_stages: Vec<StageMetrics>,
}

/// Performance metrics for each compilation stage
#[derive(Debug, Clone)]
pub struct StageMetrics {
    pub stage_name: String,
    pub duration_ms: u64,
    pub memory_usage: usize,
}

/// Run comprehensive diagnostics on source code
pub fn run_diagnostics(source: &str) -> Result<DiagnosticReport> {
    let mut report = DiagnosticReport {
        source: source.to_string(),
        type_errors: Vec::new(),
        linearity_warnings: Vec::new(),
        resource_usage: ResourceUsageAnalysis {
            total_allocations: 0,
            total_consumptions: 0,
            live_resources: Vec::new(),
            resource_lifetime_graph: Vec::new(),
        },
        compilation_summary: CompilationSummary {
            total_instructions: 0,
            register_count: 0,
            estimated_gas_cost: 0,
            compilation_stages: Vec::new(),
        },
    };

    // Attempt compilation and collect diagnostics
    match compile(source) {
        Ok(artifact) => {
            // Successful compilation - analyze for warnings and optimizations
            analyze_successful_compilation(&mut report, &artifact);
        }
        Err(error) => {
            // Compilation failed - analyze errors and provide suggestions
            analyze_compilation_error(&mut report, &error);
        }
    }

    // Additional static analysis
    analyze_resource_usage(&mut report)?;
    estimate_performance(&mut report)?;

    Ok(report)
}

/// Analyze a successful compilation for potential issues
fn analyze_successful_compilation(
    report: &mut DiagnosticReport,
    artifact: &causality_compiler::pipeline::CompiledArtifact,
) {
    // Analyze the compiled instructions
    analyze_instructions(report, &artifact.instructions);
    
    // Analyze the lambda term for linearity
    analyze_term_linearity(report, &artifact.term);
    
    // Update compilation summary
    report.compilation_summary.total_instructions = artifact.instructions.len();
    report.compilation_summary.register_count = count_registers(&artifact.instructions);
    report.compilation_summary.estimated_gas_cost = estimate_gas_cost(&artifact.instructions);
}

/// Analyze compilation errors and provide helpful diagnostics
fn analyze_compilation_error(report: &mut DiagnosticReport, error: &CompileError) {
    let (message, location, suggestion, expected_type, actual_type) = match error {
        CompileError::ParseError { message, location } => {
            (
                format!("Parse error: {}", message),
                location.clone().map(|l| (l.line, l.column)),
                Some("Check for balanced parentheses and valid syntax".to_string()),
                None,
                None,
            )
        }
        CompileError::UnknownSymbol { symbol, location } => {
            (
                format!("Unknown symbol: {}", symbol),
                location.clone().map(|l| (l.line, l.column)),
                Some(format!("Did you mean to define '{}' or is it misspelled?", symbol)),
                None,
                None,
            )
        }
        CompileError::InvalidArity { expected, found, location } => {
            (
                format!("Function called with {} arguments, expected {}", found, expected),
                location.clone().map(|l| (l.line, l.column)),
                Some("Check the function signature and argument count".to_string()),
                None,
                None,
            )
        }
        CompileError::CompilationError { message, location } => {
            (
                format!("Compilation error: {}", message),
                location.clone().map(|l| (l.line, l.column)),
                Some("Review the code structure and types".to_string()),
                None,
                None,
            )
        }
        CompileError::Layer1Error { message, location } => {
            (
                format!("Lambda calculus error: {}", message),
                location.clone().map(|l| (l.line, l.column)),
                Some("Check variable bindings and function applications".to_string()),
                None,
                None,
            )
        }
        CompileError::TypeError { message, expected, found, location } => {
            let type_info = match (expected, found) {
                (Some(exp), Some(fnd)) => format!(" (expected {}, found {})", exp, fnd),
                (Some(exp), None) => format!(" (expected {})", exp),
                (None, Some(fnd)) => format!(" (found {})", fnd),
                (None, None) => String::new(),
            };
            (
                format!("Type error: {}{}", message, type_info),
                location.clone().map(|l| (l.line, l.column)),
                Some("Check type annotations and variable usage".to_string()),
                expected.clone(),
                found.clone(),
            )
        }
        CompileError::Layer2Error { message, location } => {
            (
                format!("Effect algebra error: {}", message),
                location.clone().map(|l| (l.line, l.column)),
                Some("Check effect handling and resource management".to_string()),
                None,
                None,
            )
        }
    };

    report.type_errors.push(TypeError {
        message,
        location,
        expected_type,
        actual_type,
        suggestion,
    });
}

/// Analyze instructions for resource usage patterns
fn analyze_instructions(report: &mut DiagnosticReport, instructions: &[Instruction]) {
    let mut resource_map: BTreeMap<RegisterId, ResourceLifetime> = BTreeMap::new();
    let mut allocation_count = 0;
    let mut consumption_count = 0;

    for (i, instruction) in instructions.iter().enumerate() {
        match instruction {
            Instruction::Alloc { out_reg, .. } => {
                allocation_count += 1;
                resource_map.insert(
                    *out_reg,
                    ResourceLifetime {
                        resource_id: format!("r{}", out_reg.0),
                        allocated_at: i,
                        consumed_at: None,
                        status: ResourceStatus::Allocated,
                    },
                );
            }
            Instruction::Consume { resource_reg, .. } => {
                consumption_count += 1;
                if let Some(lifetime) = resource_map.get_mut(resource_reg) {
                    lifetime.consumed_at = Some(i);
                    lifetime.status = ResourceStatus::Consumed;
                }
            }
            _ => {}
        }
    }

    // Check for resource leaks
    for (_reg_id, lifetime) in &resource_map {
        if lifetime.consumed_at.is_none() {
            report.linearity_warnings.push(LinearityWarning {
                message: format!("Resource {} allocated but never consumed", lifetime.resource_id),
                resource: lifetime.resource_id.clone(),
                warning_type: LinearityWarningType::ResourceLeak,
                location: None,
                suggestion: "Ensure all allocated resources are consumed or explicitly handled".to_string(),
            });
        }
    }

    report.resource_usage.total_allocations = allocation_count;
    report.resource_usage.total_consumptions = consumption_count;
    report.resource_usage.resource_lifetime_graph = resource_map.into_values().collect();
}

/// Analyze lambda term for linearity properties
fn analyze_term_linearity(_report: &mut DiagnosticReport, term: &Term) {
    // Simple linearity analysis - could be expanded significantly
    let mut used_vars = BTreeSet::new();
    collect_used_variables(term, &mut used_vars);
    
    // This is a simplified analysis - in practice would need much more sophisticated
    // variable tracking and scope analysis
}

/// Collect all variables used in a term
fn collect_used_variables(term: &Term, used_vars: &mut BTreeSet<String>) {
    match &term.kind {
        TermKind::Var(name) => {
            used_vars.insert(name.clone());
        }
        TermKind::Apply { func, arg } => {
            collect_used_variables(func, used_vars);
            collect_used_variables(arg, used_vars);
        }
        TermKind::Lambda { body, .. } => {
            collect_used_variables(body, used_vars);
        }
        TermKind::Let { value, body, .. } => {
            collect_used_variables(value, used_vars);
            collect_used_variables(body, used_vars);
        }
        TermKind::Alloc { value } => {
            collect_used_variables(value, used_vars);
        }
        TermKind::Consume { resource } => {
            collect_used_variables(resource, used_vars);
        }
        _ => {}
    }
}

/// Analyze resource usage patterns
fn analyze_resource_usage(report: &mut DiagnosticReport) -> Result<()> {
    // Additional static analysis could go here
    // For now, just populate basic stats from what we already collected
    
    let live_count = report.resource_usage.resource_lifetime_graph
        .iter()
        .filter(|r| matches!(r.status, ResourceStatus::Allocated))
        .count();
    
    if live_count > 0 {
        report.linearity_warnings.push(LinearityWarning {
            message: format!("{} resources may be leaked", live_count),
            resource: "multiple".to_string(),
            warning_type: LinearityWarningType::ResourceLeak,
            location: None,
            suggestion: "Review resource consumption patterns".to_string(),
        });
    }

    Ok(())
}

/// Estimate performance characteristics
fn estimate_performance(report: &mut DiagnosticReport) -> Result<()> {
    // Add performance estimation logic
    let instruction_count = report.compilation_summary.total_instructions;
    
    report.compilation_summary.compilation_stages.push(StageMetrics {
        stage_name: "Analysis".to_string(),
        duration_ms: 1, // Mock timing
        memory_usage: instruction_count * 64, // Rough estimate
    });

    Ok(())
}

/// Count unique registers used in instructions
fn count_registers(instructions: &[Instruction]) -> usize {
    let mut registers = BTreeSet::new();
    
    for instruction in instructions {
        match instruction {
            Instruction::Move { src, dst } => {
                registers.insert(*src);
                registers.insert(*dst);
            }
            Instruction::Alloc { type_reg, val_reg, out_reg } => {
                registers.insert(*type_reg);
                registers.insert(*val_reg);
                registers.insert(*out_reg);
            }
            Instruction::Consume { resource_reg, out_reg } => {
                registers.insert(*resource_reg);
                registers.insert(*out_reg);
            }
            Instruction::Witness { out_reg } => {
                registers.insert(*out_reg);
            }
            Instruction::Apply { fn_reg, arg_reg, out_reg } => {
                registers.insert(*fn_reg);
                registers.insert(*arg_reg);
                registers.insert(*out_reg);
            }
            _ => {}
        }
    }
    
    registers.len()
}

/// Estimate gas cost for instructions
fn estimate_gas_cost(instructions: &[Instruction]) -> u64 {
    let mut total_cost = 0u64;
    
    for instruction in instructions {
        total_cost += match instruction {
            Instruction::Move { .. } => 1,
            Instruction::Alloc { .. } => 10,
            Instruction::Consume { .. } => 5,
            Instruction::Witness { .. } => 3,
            Instruction::Apply { .. } => 20,
            _ => 5,
        };
    }
    
    total_cost
}

/// Display formatted diagnostic report
impl std::fmt::Display for DiagnosticReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Causality Diagnostics Report ===")?;
        writeln!(f, "Source: {}", self.source)?;
        writeln!(f)?;

        if !self.type_errors.is_empty() {
            writeln!(f, "🚨 Type Errors:")?;
            for error in &self.type_errors {
                writeln!(f, "  • {}", error.message)?;
                if let Some((line, col)) = error.location {
                    writeln!(f, "    at line {}, column {}", line, col)?;
                }
                if let Some(suggestion) = &error.suggestion {
                    writeln!(f, "    💡 {}", suggestion)?;
                }
                writeln!(f)?;
            }
        }

        if !self.linearity_warnings.is_empty() {
            writeln!(f, "⚠️  Linearity Warnings:")?;
            for warning in &self.linearity_warnings {
                writeln!(f, "  • {}", warning.message)?;
                writeln!(f, "    💡 {}", warning.suggestion)?;
                writeln!(f)?;
            }
        }

        writeln!(f, "Resource Usage:")?;
        writeln!(f, "  Allocations: {}", self.resource_usage.total_allocations)?;
        writeln!(f, "  Consumptions: {}", self.resource_usage.total_consumptions)?;
        writeln!(f, "  Live resources: {}", self.resource_usage.live_resources.len())?;
        writeln!(f)?;

        writeln!(f, "⚡ Performance Summary:")?;
        writeln!(f, "  Instructions: {}", self.compilation_summary.total_instructions)?;
        writeln!(f, "  Registers: {}", self.compilation_summary.register_count)?;
        writeln!(f, "  Estimated gas: {}", self.compilation_summary.estimated_gas_cost)?;

        Ok(())
    }
} 