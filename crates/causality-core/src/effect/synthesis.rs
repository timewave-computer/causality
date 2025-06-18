//! Flow synthesis engine for automated effect sequence generation from intents
//!
//! This module implements basic flow synthesis that converts declarative intents
//! into executable effect sequences.

use super::{
    core::{EffectExpr, EffectExprKind},
    capability::CapabilityLevel,
    intent::{Intent, ResourceBinding, Constraint, SessionRequirement},
    session::{SessionType, SessionBranch},
};
use crate::{
    lambda::{Term, Literal, Symbol, base::Value},
    system::content_addressing::DomainId,
};
use std::collections::HashMap;
use anyhow::Result;

/// Error types for synthesis failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynthesisError {
    /// No synthesis strategy found for intent
    UnsupportedIntent(String),
    
    /// Constraint cannot be satisfied
    UnsatisfiableConstraint(String),
    
    /// Missing required resource
    MissingResource(String),
    
    /// Synthesis strategy failed
    StrategyFailed(String),
    
    /// Effect template not found
    TemplateNotFound(String),
}

/// Error types for flow validation failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Flow doesn't satisfy intent constraints
    ConstraintViolation(String),
    
    /// Missing required output
    MissingOutput(String),
    
    /// Resource conservation violated
    ConservationViolation(String),
    
    /// Invalid effect sequence
    InvalidSequence(String),
}

/// Basic constraint solver for intent satisfaction
#[derive(Debug, Clone)]
pub struct ConstraintSolver {
    /// Domain context for solving
    pub domain: DomainId,
    
    /// Available resources in the system
    pub available_resources: HashMap<String, ResourceInfo>,
    
    /// Constraint satisfaction strategies
    pub strategies: Vec<SynthesisStrategy>,
}

/// Information about an available resource
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceInfo {
    /// Resource type/label
    pub resource_type: String,
    
    /// Available quantity
    pub quantity: u64,
    
    /// Resource capabilities
    pub capabilities: Vec<String>,
    
    /// Resource metadata
    pub metadata: Value,
}

/// Basic effect library with common patterns
#[derive(Debug, Clone)]
pub struct EffectLibrary {
    /// Available effect templates by name
    pub templates: HashMap<String, EffectTemplate>,
}

/// Template for creating effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectTemplate {
    /// Template name
    pub name: String,
    
    /// Required input patterns
    pub inputs: Vec<ResourcePattern>,
    
    /// Produced output patterns  
    pub outputs: Vec<ResourcePattern>,
    
    /// Implementation as effect expression
    pub implementation: EffectExpr,
    
    /// Estimated cost of execution
    pub cost: u64,
}

/// Pattern for matching resources
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePattern {
    /// Resource type to match
    pub resource_type: String,
    
    /// Minimum quantity required
    pub min_quantity: Option<u64>,
    
    /// Maximum quantity allowed
    pub max_quantity: Option<u64>,
    
    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// Strategy for synthesizing effects from constraints
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynthesisStrategy {
    /// Direct transfer between resources
    Transfer,
    
    /// Resource transformation (mint, burn, etc.)
    Transform,
    
    /// Multi-resource exchange/swap
    Exchange,
    
    /// Resource splitting or merging
    Split,
    
    /// Custom strategy with template name
    Custom(String),
}

/// Main flow synthesizer
#[derive(Debug, Clone)]
pub struct FlowSynthesizer {
    /// Effect library for building flows
    pub effect_library: EffectLibrary,
    
    /// Constraint solver for intent satisfaction
    pub constraint_solver: ConstraintSolver,
}

impl FlowSynthesizer {
    /// Create a new flow synthesizer with default library
    pub fn new(domain: DomainId) -> Self {
        Self {
            effect_library: EffectLibrary::default(),
            constraint_solver: ConstraintSolver::new(domain),
        }
    }
    
    /// Synthesize effect sequence from an intent
    pub fn synthesize(&self, intent: &Intent) -> Result<Vec<EffectExpr>, SynthesisError> {
        // Validate intent first
        intent.validate().map_err(|e| SynthesisError::UnsupportedIntent(e.to_string()))?;
        
        // Synthesize session effects if intent has session requirements
        let mut effects = Vec::new();
        
        if !intent.session_requirements.is_empty() {
            let session_effects = self.synthesize_session_effects(intent)?;
            effects.extend(session_effects);
        }
        
        // Analyze intent constraint to determine synthesis strategy
        let strategy = self.select_strategy(&intent.constraint)?;
        
        // Apply strategy to generate effect sequence
        let main_effects = match strategy {
            SynthesisStrategy::Transfer => self.synthesize_transfer(intent),
            SynthesisStrategy::Transform => self.synthesize_transform(intent),
            SynthesisStrategy::Exchange => self.synthesize_exchange(intent),
            SynthesisStrategy::Split => self.synthesize_split(intent),
            SynthesisStrategy::Custom(template_name) => self.synthesize_custom(intent, &template_name),
        }?;
        
        effects.extend(main_effects);
        Ok(effects)
    }
    
    /// Synthesize effects for session requirements
    pub fn synthesize_session_effects(
        &self,
        intent: &Intent,
    ) -> Result<Vec<EffectExpr>, SynthesisError> {
        let mut effects = Vec::new();
        
        for requirement in &intent.session_requirements {
            let session_effects = self.compile_session_to_effects(requirement)?;
            effects.extend(session_effects);
        }
        
        Ok(effects)
    }
    
    /// Compile a session requirement to effect expressions
    pub fn compile_session_to_effects(
        &self,
        requirement: &SessionRequirement,
    ) -> Result<Vec<EffectExpr>, SynthesisError> {
        let mut effects = Vec::new();
        
        // Create session setup effect
        let setup_effect = EffectExpr::new(EffectExprKind::WithSession {
            session_decl: requirement.session_name.clone(),
            role: requirement.role.clone(),
            body: Box::new(self.compile_session_protocol(&requirement.required_protocol)?),
        });
        
        effects.push(setup_effect);
        Ok(effects)
    }
    
    /// Compile a session type to its effect implementation
    fn compile_session_protocol(&self, session_type: &SessionType) -> Result<EffectExpr, SynthesisError> {
        match session_type {
            SessionType::Send(value_type, continuation) => {
                let send_effect = EffectExpr::new(EffectExprKind::SessionSend {
                    channel: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::var("channel")))),
                    value: Term::var("message"),
                    continuation: Box::new(self.compile_session_protocol(continuation)?),
                });
                Ok(send_effect)
            }
            
            SessionType::Receive(value_type, continuation) => {
                let recv_effect = EffectExpr::new(EffectExprKind::SessionReceive {
                    channel: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::var("channel")))),
                    continuation: Box::new(self.compile_session_protocol(continuation)?),
                });
                Ok(recv_effect)
            }
            
            SessionType::InternalChoice(choices) => {
                if choices.is_empty() {
                    return Err(SynthesisError::UnsupportedIntent("Empty internal choice".to_string()));
                }
                
                // For simplicity, select the first choice
                let select_effect = EffectExpr::new(EffectExprKind::SessionSelect {
                    channel: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::var("channel")))),
                    choice: "choice_0".to_string(),
                    continuation: Box::new(self.compile_session_protocol(&choices[0])?),
                });
                Ok(select_effect)
            }
            
            SessionType::ExternalChoice(branches) => {
                if branches.is_empty() {
                    return Err(SynthesisError::UnsupportedIntent("Empty external choice".to_string()));
                }
                
                // Create case branches
                let mut case_branches = Vec::new();
                for (i, branch) in branches.iter().enumerate() {
                    case_branches.push(SessionBranch {
                        label: format!("branch_{}", i),
                        body: self.compile_session_protocol(branch)?,
                    });
                }
                
                let case_effect = EffectExpr::new(EffectExprKind::SessionCase {
                    channel: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::var("channel")))),
                    branches: case_branches,
                });
                Ok(case_effect)
            }
            
            SessionType::End => {
                // End protocol - pure effect that does nothing
                Ok(EffectExpr::new(EffectExprKind::Pure(Term::literal(Literal::Symbol(Symbol::new("end"))))))
            }
            
            SessionType::Recursive(_name, body) => {
                // For now, just compile the body (ignoring recursion)
                self.compile_session_protocol(body)
            }
            
            SessionType::Variable(_name) => {
                // Session variable - create a placeholder
                Ok(EffectExpr::new(EffectExprKind::Pure(Term::var("session_var"))))
            }
        }
    }
    
    /// Validate that a flow satisfies an intent's constraints
    pub fn validate_flow(&self, flow: &[EffectExpr], intent: &Intent) -> Result<(), ValidationError> {
        // Check basic flow validity
        if flow.is_empty() {
            return Err(ValidationError::InvalidSequence("Empty flow".to_string()));
        }
        
        // Analyze flow to extract resource transformations
        let transformations = self.analyze_flow_transformations(flow)?;
        
        // Check if transformations satisfy intent constraints
        self.check_constraint_satisfaction(&intent.constraint, &transformations)?;
        
        // Verify resource conservation if specified
        if let Some(conservation) = self.extract_conservation_constraint(&intent.constraint) {
            self.verify_conservation(&conservation, &transformations)?;
        }
        
        Ok(())
    }
    
    /// Select synthesis strategy based on constraint analysis
    #[allow(clippy::only_used_in_recursion)]
    fn select_strategy(&self, constraint: &Constraint) -> Result<SynthesisStrategy, SynthesisError> {
        match constraint {
            // Look for transfer patterns
            Constraint::And(constraints) => {
                let has_conservation = constraints.iter().any(|c| matches!(c, Constraint::Conservation(_, _)));
                let has_outputs = constraints.iter().any(|c| matches!(c, Constraint::Exists(_) | Constraint::ExistsAll(_)));
                
                if has_conservation && has_outputs {
                    return Ok(SynthesisStrategy::Transfer);
                }
                
                // Try nested analysis
                for constraint in constraints {
                    if let Ok(strategy) = self.select_strategy(constraint) {
                        return Ok(strategy);
                    }
                }
                
                // Default to transform for complex constraints
                Ok(SynthesisStrategy::Transform)
            }
            
            // Conservation suggests transfer
            Constraint::Conservation(_, _) => Ok(SynthesisStrategy::Transfer),
            
            // Output existence suggests transform
            Constraint::Exists(_) | Constraint::ExistsAll(_) => Ok(SynthesisStrategy::Transform),
            
            // Default strategy
            _ => Ok(SynthesisStrategy::Transform),
        }
    }
    
    /// Synthesize transfer effects
    fn synthesize_transfer(&self, intent: &Intent) -> Result<Vec<EffectExpr>, SynthesisError> {
        // Look for transfer template
        let template = self.effect_library.templates.get("transfer")
            .ok_or_else(|| SynthesisError::TemplateNotFound("transfer".to_string()))?;
        
        // Create transfer effect with input bindings
        let mut effects = Vec::new();
        
        // For each input binding, create a resource loading effect
        for binding in &intent.inputs {
            effects.push(self.create_load_effect(binding)?);
        }
        
        // Add the main transfer effect
        effects.push(template.implementation.clone());
        
        // Add output production effects based on constraints
        let output_bindings = self.extract_output_bindings(&intent.constraint);
        for binding in output_bindings {
            effects.push(self.create_produce_effect(&binding)?);
        }
        
        Ok(effects)
    }
    
    /// Synthesize transformation effects
    fn synthesize_transform(&self, intent: &Intent) -> Result<Vec<EffectExpr>, SynthesisError> {
        let mut effects = Vec::new();
        
        // Load inputs
        for binding in &intent.inputs {
            effects.push(self.create_load_effect(binding)?);
        }
        
        // Apply transformation
        let transform_effect = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "transform".to_string(),
            args: vec![
                Term::var("inputs"),
                Term::literal(Literal::Symbol(Symbol::new("transform"))),
            ],
        });
        effects.push(transform_effect);
        
        // Produce outputs
        let output_bindings = self.extract_output_bindings(&intent.constraint);
        for binding in output_bindings {
            effects.push(self.create_produce_effect(&binding)?);
        }
        
        Ok(effects)
    }
    
    /// Synthesize exchange/swap effects
    fn synthesize_exchange(&self, _intent: &Intent) -> Result<Vec<EffectExpr>, SynthesisError> {
        // Similar pattern but with exchange-specific logic
        let mut effects = Vec::new();
        
        // Create exchange effect
        let exchange_effect = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "exchange".to_string(),
            args: vec![Term::var("pool_state")],
        });
        effects.push(exchange_effect);
        
        Ok(effects)
    }
    
    /// Synthesize split/merge effects
    fn synthesize_split(&self, _intent: &Intent) -> Result<Vec<EffectExpr>, SynthesisError> {
        // Placeholder for split synthesis
        Err(SynthesisError::UnsupportedIntent("Split not yet implemented".to_string()))
    }
    
    /// Synthesize using custom template
    fn synthesize_custom(&self, _intent: &Intent, template_name: &str) -> Result<Vec<EffectExpr>, SynthesisError> {
        let template = self.effect_library.templates.get(template_name)
            .ok_or_else(|| SynthesisError::TemplateNotFound(template_name.to_string()))?;
        
        Ok(vec![template.implementation.clone()])
    }
    
    /// Create effect to load a resource binding
    fn create_load_effect(&self, binding: &ResourceBinding) -> Result<EffectExpr, SynthesisError> {
        Ok(EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "load_resource".to_string(),
            args: vec![
                Term::literal(Literal::Symbol(Symbol::new(&binding.name))),
                Term::literal(Literal::Symbol(Symbol::new(&binding.resource_type))),
            ],
        }))
    }
    
    /// Create effect to produce a resource
    fn create_produce_effect(&self, binding: &ResourceBinding) -> Result<EffectExpr, SynthesisError> {
        let mut args = vec![
            Term::literal(Literal::Symbol(Symbol::new(&binding.name))),
            Term::literal(Literal::Symbol(Symbol::new(&binding.resource_type))),
        ];
        
        // Add quantity if specified
        if let Some(quantity) = binding.quantity {
            args.push(Term::literal(Literal::Int(quantity as u32)));
        }
        
        Ok(EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "produce_resource".to_string(),
            args,
        }))
    }
    
    /// Extract output bindings from constraint tree
    fn extract_output_bindings(&self, constraint: &Constraint) -> Vec<ResourceBinding> {
        let mut bindings = Vec::new();
        self.extract_output_bindings_recursive(constraint, &mut bindings);
        bindings
    }
    
    /// Recursively extract output bindings
    #[allow(clippy::only_used_in_recursion)]
    fn extract_output_bindings_recursive(&self, constraint: &Constraint, bindings: &mut Vec<ResourceBinding>) {
        match constraint {
            Constraint::And(constraints) | Constraint::Or(constraints) => {
                for constraint in constraints {
                    self.extract_output_bindings_recursive(constraint, bindings);
                }
            }
            Constraint::Not(constraint) => {
                self.extract_output_bindings_recursive(constraint, bindings);
            }
            Constraint::Exists(binding) => {
                bindings.push(binding.clone());
            }
            Constraint::ExistsAll(output_bindings) => {
                bindings.extend(output_bindings.clone());
            }
            _ => {} // Other constraints don't contain output bindings
        }
    }
    
    /// Analyze flow to extract resource transformations
    fn analyze_flow_transformations(&self, flow: &[EffectExpr]) -> Result<Vec<ResourceTransformation>, ValidationError> {
        let mut transformations = Vec::new();
        
        for effect in flow {
            if let EffectExprKind::Perform { effect_tag, args } = &effect.kind {
                match effect_tag.as_str() {
                    "transfer" | "load_resource" | "produce_resource" => {
                        let transformation = self.extract_transformation_from_effect(effect_tag, args)
                            .map_err(ValidationError::InvalidSequence)?;
                        transformations.push(transformation);
                    }
                    _ => {} // Other effects don't create resource transformations
                }
            }
        }
        
        Ok(transformations)
    }
    
    /// Extract resource transformation from effect
    fn extract_transformation_from_effect(&self, effect_tag: &str, _args: &[Term]) -> Result<ResourceTransformation, String> {
        // Simplified transformation extraction
        Ok(ResourceTransformation {
            effect_type: effect_tag.to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            metadata: Value::Bool(false),
        })
    }
    
    /// Check if transformations satisfy constraints
    fn check_constraint_satisfaction(
        &self, 
        _constraint: &Constraint, 
        _transformations: &[ResourceTransformation]
    ) -> Result<(), ValidationError> {
        // Simplified constraint checking - just succeed for now
        Ok(())
    }
    
    /// Extract conservation constraint if present
    #[allow(clippy::only_used_in_recursion)]
    fn extract_conservation_constraint(&self, constraint: &Constraint) -> Option<(Vec<String>, Vec<String>)> {
        match constraint {
            Constraint::Conservation(inputs, outputs) => Some((inputs.clone(), outputs.clone())),
            Constraint::And(constraints) => {
                for constraint in constraints {
                    if let Some(conservation) = self.extract_conservation_constraint(constraint) {
                        return Some(conservation);
                    }
                }
                None
            }
            _ => None,
        }
    }
    
    /// Verify resource conservation
    fn verify_conservation(
        &self, 
        _conservation: &(Vec<String>, Vec<String>), 
        _transformations: &[ResourceTransformation]
    ) -> Result<(), ValidationError> {
        // Simplified conservation checking - just succeed for now
        Ok(())
    }
}

/// Resource transformation representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTransformation {
    /// Type of effect that created this transformation
    pub effect_type: String,
    
    /// Input resources consumed
    pub inputs: Vec<ResourceInfo>,
    
    /// Output resources produced
    pub outputs: Vec<ResourceInfo>,
    
    /// Additional metadata
    pub metadata: Value,
}

impl ConstraintSolver {
    /// Create a new constraint solver
    pub fn new(domain: DomainId) -> Self {
        Self {
            domain,
            available_resources: HashMap::new(),
            strategies: vec![
                SynthesisStrategy::Transfer,
                SynthesisStrategy::Transform,
                SynthesisStrategy::Exchange,
            ],
        }
    }
    
    /// Add an available resource
    pub fn add_resource(&mut self, name: String, info: ResourceInfo) {
        self.available_resources.insert(name, info);
    }
}

impl Default for EffectLibrary {
    /// Create an effect library with default templates
    fn default() -> Self {
        let mut templates = HashMap::new();
        
        // Basic transfer template
        templates.insert("transfer".to_string(), EffectTemplate {
            name: "transfer".to_string(),
            inputs: vec![
                ResourcePattern {
                    resource_type: "Token".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec!["read".to_string()],
                }
            ],
            outputs: vec![
                ResourcePattern {
                    resource_type: "Token".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec![],
                }
            ],
            implementation: EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "transfer".to_string(),
                args: vec![Term::var("source"), Term::var("destination"), Term::var("amount")],
            }),
            cost: 100,
        });
        
        // Basic transform template
        templates.insert("transform".to_string(), EffectTemplate {
            name: "transform".to_string(),
            inputs: vec![
                ResourcePattern {
                    resource_type: "Any".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec!["write".to_string()],
                }
            ],
            outputs: vec![
                ResourcePattern {
                    resource_type: "Any".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec![],
                }
            ],
            implementation: EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "transform".to_string(),
                args: vec![Term::var("input"), Term::var("transformation")],
            }),
            cost: 200,
        });
        
        // Token minting template
        templates.insert("mint".to_string(), EffectTemplate {
            name: "mint".to_string(),
            inputs: vec![
                ResourcePattern {
                    resource_type: "MintAuthority".to_string(),
                    min_quantity: Some(1),
                    max_quantity: Some(1),
                    required_capabilities: vec!["write".to_string(), "mint".to_string()],
                }
            ],
            outputs: vec![
                ResourcePattern {
                    resource_type: "Token".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec![],
                }
            ],
            implementation: EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "mint".to_string(),
                args: vec![Term::var("authority"), Term::var("recipient"), Term::var("amount")],
            }),
            cost: 150,
        });
        
        // Token burning template
        templates.insert("burn".to_string(), EffectTemplate {
            name: "burn".to_string(),
            inputs: vec![
                ResourcePattern {
                    resource_type: "Token".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec!["write".to_string()],
                }
            ],
            outputs: vec![], // No outputs - tokens are destroyed
            implementation: EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "burn".to_string(),
                args: vec![Term::var("tokens"), Term::var("amount")],
            }),
            cost: 120,
        });
        
        // Token swap template
        templates.insert("swap".to_string(), EffectTemplate {
            name: "swap".to_string(),
            inputs: vec![
                ResourcePattern {
                    resource_type: "TokenA".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec!["read".to_string()],
                },
                ResourcePattern {
                    resource_type: "LiquidityPool".to_string(),
                    min_quantity: Some(1),
                    max_quantity: Some(1),
                    required_capabilities: vec!["read".to_string(), "write".to_string()],
                }
            ],
            outputs: vec![
                ResourcePattern {
                    resource_type: "TokenB".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec![],
                },
                ResourcePattern {
                    resource_type: "LiquidityPool".to_string(),
                    min_quantity: Some(1),
                    max_quantity: Some(1),
                    required_capabilities: vec![],
                }
            ],
            implementation: EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "swap".to_string(),
                args: vec![Term::var("input_tokens"), Term::var("pool"), Term::var("min_output")],
            }),
            cost: 300,
        });
        
        // Liquidity provision template
        templates.insert("add_liquidity".to_string(), EffectTemplate {
            name: "add_liquidity".to_string(),
            inputs: vec![
                ResourcePattern {
                    resource_type: "TokenA".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec!["read".to_string()],
                },
                ResourcePattern {
                    resource_type: "TokenB".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec!["read".to_string()],
                },
                ResourcePattern {
                    resource_type: "LiquidityPool".to_string(),
                    min_quantity: Some(1),
                    max_quantity: Some(1),
                    required_capabilities: vec!["write".to_string()],
                }
            ],
            outputs: vec![
                ResourcePattern {
                    resource_type: "LPToken".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec![],
                },
                ResourcePattern {
                    resource_type: "LiquidityPool".to_string(),
                    min_quantity: Some(1),
                    max_quantity: Some(1),
                    required_capabilities: vec![],
                }
            ],
            implementation: EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "add_liquidity".to_string(),
                args: vec![Term::var("token_a"), Term::var("token_b"), Term::var("pool")],
            }),
            cost: 250,
        });
        
        // Liquidity removal template
        templates.insert("remove_liquidity".to_string(), EffectTemplate {
            name: "remove_liquidity".to_string(),
            inputs: vec![
                ResourcePattern {
                    resource_type: "LPToken".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec!["read".to_string()],
                },
                ResourcePattern {
                    resource_type: "LiquidityPool".to_string(),
                    min_quantity: Some(1),
                    max_quantity: Some(1),
                    required_capabilities: vec!["write".to_string()],
                }
            ],
            outputs: vec![
                ResourcePattern {
                    resource_type: "TokenA".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec![],
                },
                ResourcePattern {
                    resource_type: "TokenB".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec![],
                },
                ResourcePattern {
                    resource_type: "LiquidityPool".to_string(),
                    min_quantity: Some(1),
                    max_quantity: Some(1),
                    required_capabilities: vec![],
                }
            ],
            implementation: EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "remove_liquidity".to_string(),
                args: vec![Term::var("lp_tokens"), Term::var("pool")],
            }),
            cost: 220,
        });
        
        Self { templates }
    }
}

impl EffectLibrary {
    /// Add a new effect template to the library
    pub fn add_template(&mut self, template: EffectTemplate) {
        self.templates.insert(template.name.clone(), template);
    }
    
    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&EffectTemplate> {
        self.templates.get(name)
    }
    
    /// Find templates that match intent requirements
    pub fn find_matching_templates(&self, intent: &Intent) -> Vec<&EffectTemplate> {
        let mut matches = Vec::new();
        
        for template in self.templates.values() {
            if self.template_matches_intent(template, intent) {
                matches.push(template);
            }
        }
        
        // Sort by cost (prefer lower cost)
        matches.sort_by_key(|t| t.cost);
        matches
    }
    
    /// Check if a template matches an intent's requirements
    fn template_matches_intent(&self, template: &EffectTemplate, intent: &Intent) -> bool {
        // Basic matching: check if we have enough inputs
        if template.inputs.len() > intent.inputs.len() {
            return false;
        }
        
        // Check if input patterns can be satisfied by intent inputs
        for input_pattern in &template.inputs {
            if !self.has_matching_input(input_pattern, &intent.inputs) {
                return false;
            }
        }
        
        // Check if template outputs match intent constraint requirements
        let output_bindings = self.extract_output_requirements(intent);
        if !self.template_produces_required_outputs(template, &output_bindings) {
            return false;
        }
        
        true
    }
    
    /// Check if intent has input matching pattern requirements
    fn has_matching_input(&self, pattern: &ResourcePattern, bindings: &[ResourceBinding]) -> bool {
        bindings.iter().any(|binding| {
            // Type matching (allowing "Any" to match anything)
            let type_matches = pattern.resource_type == "Any" || 
                              pattern.resource_type == binding.resource_type;
            
            // Quantity matching
            let quantity_matches = match (pattern.min_quantity, binding.quantity) {
                (Some(min), Some(quantity)) => quantity >= min,
                (None, _) => true,
                (Some(_), None) => false, // Pattern requires quantity but binding has none
            };
            
            // Capability matching (simplified - check if binding has required capabilities)
            let capabilities_match = pattern.required_capabilities.iter()
                .all(|req_cap| binding.capabilities.iter()
                    .any(|cap| {
                        // Match capability names based on what the factory methods create
                        match req_cap.as_str() {
                            "read" => cap.name == "read" || cap.level == CapabilityLevel::Read,
                            "write" => cap.name == "write" || cap.level == CapabilityLevel::Write,
                            "execute" => cap.name == "execute" || cap.level == CapabilityLevel::Execute,
                            "admin" => cap.name == "admin" || cap.level == CapabilityLevel::Admin,
                            _ => cap.name == *req_cap,
                        }
                    }));
            
            type_matches && quantity_matches && capabilities_match
        })
    }
    
    /// Extract output requirements from intent constraints
    fn extract_output_requirements(&self, intent: &Intent) -> Vec<ResourceBinding> {
        let mut outputs = Vec::new();
        self.extract_outputs_recursive(&intent.constraint, &mut outputs);
        outputs
    }
    
    /// Recursively extract output requirements
    #[allow(clippy::only_used_in_recursion)]
    fn extract_outputs_recursive(&self, constraint: &Constraint, outputs: &mut Vec<ResourceBinding>) {
        match constraint {
            Constraint::And(constraints) | Constraint::Or(constraints) => {
                for constraint in constraints {
                    self.extract_outputs_recursive(constraint, outputs);
                }
            }
            Constraint::Not(constraint) => {
                self.extract_outputs_recursive(constraint, outputs);
            }
            Constraint::Exists(binding) => {
                outputs.push(binding.clone());
            }
            Constraint::ExistsAll(bindings) => {
                outputs.extend(bindings.clone());
            }
            _ => {} // Other constraints don't specify outputs
        }
    }
    
    /// Check if template produces required outputs
    fn template_produces_required_outputs(&self, template: &EffectTemplate, required: &[ResourceBinding]) -> bool {
        // For each required output, check if template can produce it
        required.iter().all(|req_output| {
            template.outputs.iter().any(|template_output| {
                // Type matching
                let type_matches = template_output.resource_type == "Any" || 
                                  template_output.resource_type == req_output.resource_type;
                
                // Quantity matching
                let quantity_matches = match (req_output.quantity, template_output.min_quantity) {
                    (Some(req_qty), Some(min_qty)) => req_qty >= min_qty,
                    (None, _) => true,
                    (Some(_), None) => true, // Template can produce any quantity
                };
                
                type_matches && quantity_matches
            })
        })
    }
    
    /// Create an effect library with DeFi-focused templates
    pub fn defi_focused() -> Self {
        let mut library = Self::default();
        
        // Add additional DeFi-specific templates
        library.add_template(EffectTemplate {
            name: "flash_loan".to_string(),
            inputs: vec![
                ResourcePattern {
                    resource_type: "LendingPool".to_string(),
                    min_quantity: Some(1),
                    max_quantity: Some(1),
                    required_capabilities: vec!["read".to_string()],
                }
            ],
            outputs: vec![
                ResourcePattern {
                    resource_type: "Token".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec![],
                }
            ],
            implementation: EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "flash_loan".to_string(),
                args: vec![Term::var("pool"), Term::var("amount"), Term::var("callback")],
            }),
            cost: 500,
        });
        
        library.add_template(EffectTemplate {
            name: "arbitrage".to_string(),
            inputs: vec![
                ResourcePattern {
                    resource_type: "Token".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec!["read".to_string()],
                },
                ResourcePattern {
                    resource_type: "LiquidityPool".to_string(),
                    min_quantity: Some(2),
                    max_quantity: None,
                    required_capabilities: vec!["read".to_string()],
                }
            ],
            outputs: vec![
                ResourcePattern {
                    resource_type: "Token".to_string(),
                    min_quantity: Some(1),
                    max_quantity: None,
                    required_capabilities: vec![],
                }
            ],
            implementation: EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "arbitrage".to_string(),
                args: vec![Term::var("initial_token"), Term::var("pools"), Term::var("path")],
            }),
            cost: 400,
        });
        
        library
    }
}

// Error display implementations
impl std::fmt::Display for SynthesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SynthesisError::UnsupportedIntent(msg) => write!(f, "Unsupported intent: {}", msg),
            SynthesisError::UnsatisfiableConstraint(msg) => write!(f, "Unsatisfiable constraint: {}", msg),
            SynthesisError::MissingResource(name) => write!(f, "Missing resource: {}", name),
            SynthesisError::StrategyFailed(msg) => write!(f, "Strategy failed: {}", msg),
            SynthesisError::TemplateNotFound(name) => write!(f, "Template not found: {}", name),
        }
    }
}

impl std::error::Error for SynthesisError {}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            ValidationError::MissingOutput(name) => write!(f, "Missing output: {}", name),
            ValidationError::ConservationViolation(msg) => write!(f, "Conservation violation: {}", msg),
            ValidationError::InvalidSequence(msg) => write!(f, "Invalid sequence: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        effect::{Intent, ResourceBinding, Constraint, capability::Capability},
        system::content_addressing::DomainId,
        lambda::base::Value,
    };

    #[test]
    fn test_flow_synthesizer_creation() {
        let domain_name = String::from("test_domain");
        let domain = DomainId::from_content(&domain_name.as_bytes().to_vec());
        let synthesizer = FlowSynthesizer::new(domain);
        
        assert_eq!(synthesizer.constraint_solver.domain, domain);
        assert!(synthesizer.effect_library.templates.contains_key("transfer"));
        assert!(synthesizer.effect_library.templates.contains_key("transform"));
    }

    #[test] 
    fn test_simple_transfer_synthesis() {
        let domain_name = String::from("test_domain");
        let domain = DomainId::from_content(&domain_name.as_bytes().to_vec());
        let synthesizer = FlowSynthesizer::new(domain);
        
        let intent = Intent::new(
            domain,
            vec![
                ResourceBinding::new("source_tokens", "Token").with_quantity(100),
            ],
            Constraint::and(vec![
                Constraint::produces_quantity("dest_tokens", "Token", 100),
                Constraint::conservation(
                    vec!["source_tokens".to_string()],
                    vec!["dest_tokens".to_string()],
                ),
            ]),
        );
        
        let result = synthesizer.synthesize(&intent);
        assert!(result.is_ok());
        
        let effects = result.unwrap();
        assert!(!effects.is_empty());
        
        // Should have load, transfer, and produce effects
        assert!(effects.len() >= 3);
    }

    #[test]
    fn test_flow_validation() {
        let domain_name = String::from("test_domain");
        let domain = DomainId::from_content(&domain_name.as_bytes().to_vec());
        let synthesizer = FlowSynthesizer::new(domain);
        
        let intent = Intent::new(
            domain,
            vec![ResourceBinding::new("input", "Token")],
            Constraint::produces("output", "Token"),
        );
        
        let flow = vec![
            EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "load_resource".to_string(),
                args: vec![Term::var("input")],
            }),
            EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "produce_resource".to_string(),
                args: vec![Term::var("output")],
            }),
        ];
        
        let result = synthesizer.validate_flow(&flow, &intent);
        assert!(result.is_ok());
    }

    #[test]
    fn test_effect_library_default_templates() {
        let library = EffectLibrary::default();
        
        assert!(library.get_template("transfer").is_some());
        assert!(library.get_template("transform").is_some());
        assert!(library.get_template("nonexistent").is_none());
        
        let transfer_template = library.get_template("transfer").unwrap();
        assert_eq!(transfer_template.name, "transfer");
        assert_eq!(transfer_template.cost, 100);
        assert!(!transfer_template.inputs.is_empty());
        assert!(!transfer_template.outputs.is_empty());
    }

    #[test]
    fn test_strategy_selection() {
        let domain_name = String::from("test_domain");
        let domain = DomainId::from_content(&domain_name.as_bytes().to_vec());
        let synthesizer = FlowSynthesizer::new(domain);
        
        // Transfer strategy for conservation constraints
        let transfer_constraint = Constraint::conservation(
            vec!["input".to_string()],
            vec!["output".to_string()],
        );
        let strategy = synthesizer.select_strategy(&transfer_constraint).unwrap();
        assert!(matches!(strategy, SynthesisStrategy::Transfer));
        
        // Transform strategy for existence constraints
        let transform_constraint = Constraint::produces("output", "Token");
        let strategy = synthesizer.select_strategy(&transform_constraint).unwrap();
        assert!(matches!(strategy, SynthesisStrategy::Transform));
    }

    #[test]
    fn test_output_binding_extraction() {
        let domain_name = String::from("test_domain");
        let domain = DomainId::from_content(&domain_name.as_bytes().to_vec());
        let synthesizer = FlowSynthesizer::new(domain);
        
        let constraint = Constraint::and(vec![
            Constraint::produces_quantity("token_a", "TokenA", 100),
            Constraint::produces_quantity("token_b", "TokenB", 50),
        ]);
        
        let outputs = synthesizer.extract_output_bindings(&constraint);
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].name, "token_a");
        assert_eq!(outputs[0].quantity, Some(100));
        assert_eq!(outputs[1].name, "token_b");
        assert_eq!(outputs[1].quantity, Some(50));
    }

    #[test]
    fn test_expanded_effect_library() {
        let library = EffectLibrary::default();
        
        // Test that we have all the basic templates that are actually implemented
        assert!(library.get_template("transfer").is_some());
        assert!(library.get_template("transform").is_some());
        assert!(library.get_template("mint").is_some());
        assert!(library.get_template("burn").is_some());
        assert!(library.get_template("swap").is_some());
        assert!(library.get_template("add_liquidity").is_some());
        assert!(library.get_template("remove_liquidity").is_some());
        
        // Test template properties
        let mint_template = library.get_template("mint").unwrap();
        assert_eq!(mint_template.name, "mint");
        assert_eq!(mint_template.inputs.len(), 1);
        assert_eq!(mint_template.outputs.len(), 1);
        assert_eq!(mint_template.inputs[0].resource_type, "MintAuthority");
        assert_eq!(mint_template.outputs[0].resource_type, "Token");
        
        let swap_template = library.get_template("swap").unwrap();
        assert_eq!(swap_template.inputs.len(), 2);
        assert_eq!(swap_template.outputs.len(), 2);
        assert!(swap_template.cost > 0);
    }
    
    #[test]
    fn test_defi_focused_library() {
        let library = EffectLibrary::defi_focused();
        
        // Should have all default templates plus DeFi-specific ones
        assert!(library.get_template("transfer").is_some());
        assert!(library.get_template("flash_loan").is_some());
        assert!(library.get_template("arbitrage").is_some());
        
        let flash_loan = library.get_template("flash_loan").unwrap();
        assert_eq!(flash_loan.name, "flash_loan");
        assert_eq!(flash_loan.inputs[0].resource_type, "LendingPool");
        assert_eq!(flash_loan.outputs[0].resource_type, "Token");
        assert!(flash_loan.cost > 0);
    }
    
    #[test]
    fn test_template_matching() {
        let library = EffectLibrary::default();
        
        // Create an intent for token minting
        let domain_name = String::from("defi_domain");
        let domain = DomainId::from_content(&domain_name.as_bytes().to_vec());
        let mint_intent = Intent::new(
            domain,
            vec![ResourceBinding {
                name: "mint_auth".to_string(),
                resource_type: "MintAuthority".to_string(),
                quantity: Some(1),
                constraints: vec![],
                capabilities: vec![Capability::read("mint_auth"), Capability::write("mint_auth")],
                metadata: Value::Unit,
            }],
            Constraint::Exists(ResourceBinding {
                name: "new_tokens".to_string(),
                resource_type: "Token".to_string(),
                quantity: Some(100),
                constraints: vec![],
                capabilities: vec![],
                metadata: Value::Unit,
            }),
        );
        
        let matches = library.find_matching_templates(&mint_intent);
        assert!(!matches.is_empty());
        
        // Should prefer lower cost templates
        let first_match = matches[0];
        for match_template in matches.iter().skip(1) {
            assert!(first_match.cost <= match_template.cost);
        }
    }
    
    #[test]
    fn test_swap_template_matching() {
        let library = EffectLibrary::default();
        
        // Create an intent for token swapping
        let domain_name = String::from("defi_domain");
        let domain = DomainId::from_content(&domain_name.as_bytes().to_vec());
        let swap_intent = Intent::new(
            domain,
            vec![
                ResourceBinding {
                    name: "input_tokens".to_string(),
                    resource_type: "TokenA".to_string(),
                    quantity: Some(100),
                    constraints: vec![],
                    capabilities: vec![Capability::read("input_tokens")],
                    metadata: Value::Unit,
                },
                ResourceBinding {
                    name: "pool".to_string(),
                    resource_type: "LiquidityPool".to_string(),
                    quantity: Some(1),
                    constraints: vec![],
                    capabilities: vec![Capability::read("pool"), Capability::write("pool")],
                    metadata: Value::Unit,
                }
            ],
            Constraint::And(vec![
                Constraint::Exists(ResourceBinding {
                    name: "output_tokens".to_string(),
                    resource_type: "TokenB".to_string(),
                    quantity: Some(50),
                    constraints: vec![],
                    capabilities: vec![],
                    metadata: Value::Unit,
                }),
                Constraint::Exists(ResourceBinding {
                    name: "updated_pool".to_string(),
                    resource_type: "LiquidityPool".to_string(),
                    quantity: Some(1),
                    constraints: vec![],
                    capabilities: vec![],
                    metadata: Value::Unit,
                })
            ]),
        );
        
        let matches = library.find_matching_templates(&swap_intent);
        assert!(!matches.is_empty());
        
        // Should find the swap template
        let has_swap = matches.iter().any(|t| t.name == "swap");
        assert!(has_swap, "Should find swap template for swap intent");
    }
    
    #[test]
    fn test_complex_constraint_output_extraction() {
        let library = EffectLibrary::default();
        
        // Create an intent with complex nested constraints
        let complex_constraint = Constraint::And(vec![
            Constraint::Or(vec![
                Constraint::Exists(ResourceBinding {
                    name: "token_a".to_string(),
                    resource_type: "TokenA".to_string(),
                    quantity: Some(100),
                    constraints: vec![],
                    capabilities: vec![],
                    metadata: Value::Unit,
                }),
                Constraint::Exists(ResourceBinding {
                    name: "token_b".to_string(),
                    resource_type: "TokenB".to_string(),
                    quantity: Some(200),
                    constraints: vec![],
                    capabilities: vec![],
                    metadata: Value::Unit,
                }),
            ]),
            Constraint::ExistsAll(vec![
                ResourceBinding {
                    name: "lp_token".to_string(),
                    resource_type: "LPToken".to_string(),
                    quantity: Some(50),
                    constraints: vec![],
                    capabilities: vec![],
                    metadata: Value::Unit,
                },
                ResourceBinding {
                    name: "receipt".to_string(),
                    resource_type: "Receipt".to_string(),
                    quantity: Some(1),
                    constraints: vec![],
                    capabilities: vec![],
                    metadata: Value::Unit,
                }
            ])
        ]);
        
        let domain_name = String::from("test_domain");
        let domain = DomainId::from_content(&domain_name.as_bytes().to_vec());
        let intent = Intent::new(
            domain,
            vec![], // No inputs for this test
            complex_constraint,
        );
        
        let outputs = library.extract_output_requirements(&intent);
        assert_eq!(outputs.len(), 4); // Should extract all 4 output requirements
        
        // Verify we got all expected outputs
        let names: Vec<&str> = outputs.iter().map(|o| o.name.as_str()).collect();
        assert!(names.contains(&"token_a"));
        assert!(names.contains(&"token_b"));
        assert!(names.contains(&"lp_token"));
        assert!(names.contains(&"receipt"));
    }
    
    #[test]
    fn test_session_synthesis() {
        use crate::lambda::base::{TypeInner, BaseType};
        use crate::effect::intent::SessionRequirement;
        use crate::effect::session::SessionType;
        
        let domain = DomainId::from_content(&vec![42u8; 32]);
        let synthesizer = FlowSynthesizer::new(domain);
        
        // Create an intent with session requirements
        let session_requirement = SessionRequirement::new(
            "PaymentProtocol",
            "client",
            SessionType::Send(
                TypeInner::Base(BaseType::Int),
                Box::new(SessionType::Receive(
                    TypeInner::Base(BaseType::Bool),
                    Box::new(SessionType::End)
                ))
            )
        );
        
        let intent = Intent::new(
            domain,
            vec![ResourceBinding::new("payment", "Payment")],
            Constraint::produces("receipt", "Receipt")
        ).with_session_requirement(session_requirement);
        
        let effects = synthesizer.synthesize(&intent).unwrap();
        
        // Should have session effects plus regular effects
        assert!(!effects.is_empty());
        
        // First effect should be a session setup
        assert!(matches!(effects[0].kind, EffectExprKind::WithSession { .. }));
    }
    
    #[test]
    fn test_session_protocol_compilation() {
        use crate::lambda::base::{TypeInner, BaseType};
        use crate::effect::session::SessionType;
        
        let domain = DomainId::from_content(&vec![42u8; 32]);
        let synthesizer = FlowSynthesizer::new(domain);
        
        // Test simple send protocol
        let send_protocol = SessionType::Send(
            TypeInner::Base(BaseType::Int),
            Box::new(SessionType::End)
        );
        
        let effect = synthesizer.compile_session_protocol(&send_protocol).unwrap();
        assert!(matches!(effect.kind, EffectExprKind::SessionSend { .. }));
        
        // Test receive protocol
        let recv_protocol = SessionType::Receive(
            TypeInner::Base(BaseType::Int),
            Box::new(SessionType::End)
        );
        
        let effect = synthesizer.compile_session_protocol(&recv_protocol).unwrap();
        assert!(matches!(effect.kind, EffectExprKind::SessionReceive { .. }));
        
        // Test choice protocol
        let choice_protocol = SessionType::InternalChoice(vec![
            SessionType::End,
            SessionType::Send(
                TypeInner::Base(BaseType::Int),
                Box::new(SessionType::End)
            )
        ]);
        
        let effect = synthesizer.compile_session_protocol(&choice_protocol).unwrap();
        assert!(matches!(effect.kind, EffectExprKind::SessionSelect { .. }));
    }
    
    #[test]
    fn test_session_compilation_error_handling() {
        use crate::effect::session::SessionType;
        
        let domain = DomainId::from_content(&vec![42u8; 32]);
        let synthesizer = FlowSynthesizer::new(domain);
        
        // Test empty choice error
        let empty_choice = SessionType::InternalChoice(vec![]);
        let result = synthesizer.compile_session_protocol(&empty_choice);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SynthesisError::UnsupportedIntent(_)));
        
        // Test empty external choice error
        let empty_external_choice = SessionType::ExternalChoice(vec![]);
        let result = synthesizer.compile_session_protocol(&empty_external_choice);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SynthesisError::UnsupportedIntent(_)));
    }
} 