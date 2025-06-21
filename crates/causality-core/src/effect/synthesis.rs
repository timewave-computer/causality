//! Flow synthesis engine for automated effect sequence generation from intents
//!
//! This module implements basic flow synthesis that converts declarative intents
//! into executable effect sequences.

use crate::{
    effect::{
        intent::{Intent, ResourceBinding, LocationRequirements},
        transform_constraint::{TransformConstraint, TransformConstraintError},
    },
    lambda::{
        base::{TypeInner, Value, Location, SessionType, BaseType},
        Term, TermKind, Literal, Symbol,
    },
    machine::{
        instruction::{Instruction, RegisterId},
        value::MachineValue,
    },
    effect::capability::Capability,
    system::{
        content_addressing::{EntityId, Timestamp, Str},
        causality::CausalProof,
    },
};
use std::collections::BTreeMap;
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
    pub domain: Location,
    
    /// Available resources in the system
    pub available_resources: BTreeMap<String, ResourceInfo>,
    
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
    pub templates: BTreeMap<String, EffectTemplate>,
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
    pub fn new(domain: Location) -> Self {
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
                    continuation: Box::new(self.compile_session_protocol(&choices[0].1)?),
                });
                Ok(select_effect)
            }
            
            SessionType::ExternalChoice(branches) => {
                if branches.is_empty() {
                    return Err(SynthesisError::UnsupportedIntent("Empty external choice".to_string()));
                }
                
                // Create case branches
                let mut case_branches = Vec::new();
                for (i, (label, session_type)) in branches.iter().enumerate() {
                    case_branches.push(SessionBranch {
                        label: label.clone(),
                        body: self.compile_session_protocol(session_type)?,
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
                self.create_resource_term(binding),
            ],
        }))
    }
    
    /// Create effect to produce a resource
    fn create_produce_effect(&self, binding: &ResourceBinding) -> Result<EffectExpr, SynthesisError> {
        let mut args = vec![
            self.create_resource_term(binding),
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
    
    /// Create a term for a resource binding
    fn create_resource_term(&self, binding: &ResourceBinding) -> Term {
        match binding.resource.access_pattern {
            crate::effect::intent::AccessPattern::ReadOnly => {
                Term::literal(Literal::Symbol(Symbol::new(&binding.resource.resource_type.to_string()))),
            }
            _ => {
                // For other access patterns, create appropriate terms
                Term::literal(Literal::Symbol(Symbol::new(&binding.resource.resource_type.to_string()))),
            }
        }
        
        // For now, create a simple term based on the resource type
        Term::literal(Literal::Symbol(Symbol::new(&binding.resource.resource_type.to_string()))),
        
        // Add quantity if present
        if binding.required {
            Term::literal(Literal::Symbol(Symbol::new("required")))
        } else {
            Term::literal(Literal::Symbol(Symbol::new("optional")))
        }
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

    /// Validate that an intent is well-formed and supported
    fn validate_intent(&self, intent: &Intent) -> Result<(), SynthesisError> {
        // Check if intent has valid constraints
        if intent.constraints.is_empty() {
            return Err(SynthesisError::UnsupportedIntent(
                "Intent must have at least one constraint".to_string()
            ));
        }
        
        // Check location requirements are reasonable
        if !intent.location_requirements.allowed_locations.is_empty() {
            return Err(SynthesisError::UnsupportedIntent(
                "Complex location requirements not yet supported".to_string()
            ));
        }
        
        // Check if we have required protocols
        if intent.location_requirements.required_protocols.len() > 5 {
            return Err(SynthesisError::UnsupportedIntent(
                "Too many required protocols".to_string()
            ));
        }
        
        // Check constraints for compatibility
        let strategy = self.select_strategy(&intent.constraints)?;
        
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
    pub fn new(domain: Location) -> Self {
        Self {
            domain,
            available_resources: BTreeMap::new(),
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
        let mut templates = BTreeMap::new();
        
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
        lambda::base::{Value, Location},
    };

    #[test]
    fn test_flow_synthesizer_creation() {
        let location = Location::Remote("test_domain".to_string());
        let synthesizer = FlowSynthesizer::new(location);
        
        assert!(!synthesizer.effect_library.templates.is_empty());
        assert!(!synthesizer.constraint_solver.strategies.is_empty());
    }

    #[test]
    fn test_simple_transfer_synthesis() {
        let location = Location::Remote("test_domain".to_string());
        let synthesizer = FlowSynthesizer::new(location.clone());
        
        let intent = Intent::new(
            location,
            vec![
                ResourceBinding::new("source_account", "Account").with_quantity(100),
            ],
            Constraint::produces_quantity("dest_account", "Account", 100),
        );
        
        let result = synthesizer.synthesize(&intent);
        assert!(result.is_ok());
        
        let effects = result.unwrap();
        assert!(!effects.is_empty());
    }

    #[test]
    fn test_flow_validation() {
        let location = Location::Remote("test_domain".to_string());
        let synthesizer = FlowSynthesizer::new(location.clone());
        
        let intent = Intent::new(
            location,
            vec![ResourceBinding::new("input", "Token")],
            Constraint::produces("output", "Token"),
        );
        
        let effects = vec![
            EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "load_resource".to_string(),
                args: vec![Term::literal(Literal::Symbol(Symbol::new("input")))],
            }),
            EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "produce_resource".to_string(),
                args: vec![Term::literal(Literal::Symbol(Symbol::new("output")))],
            }),
        ];
        
        let result = synthesizer.validate_flow(&effects, &intent);
        assert!(result.is_ok());
    }

    #[test]
    fn test_effect_library_default_templates() {
        let library = EffectLibrary::default();
        
        assert!(library.templates.contains_key("transfer"));
        assert!(library.templates.contains_key("transform"));
        assert!(library.templates.contains_key("mint"));
        assert!(library.templates.contains_key("burn"));
        assert!(library.templates.contains_key("swap"));
        
        let transfer_template = &library.templates["transfer"];
        assert_eq!(transfer_template.name, "transfer");
        assert!(!transfer_template.inputs.is_empty());
        assert!(!transfer_template.outputs.is_empty());
    }

    #[test]
    fn test_strategy_selection() {
        let location = Location::Remote("test_domain".to_string());
        let synthesizer = FlowSynthesizer::new(location);
        
        // Test conservation constraint -> Transfer strategy
        let conservation_constraint = Constraint::conservation(
            vec!["input".to_string()],
            vec!["output".to_string()],
        );
        let strategy = synthesizer.select_strategy(&conservation_constraint);
        assert!(strategy.is_ok());
        assert!(matches!(strategy.unwrap(), SynthesisStrategy::Transfer));
        
        // Test existence constraint -> Transform strategy
        let existence_constraint = Constraint::produces("output", "Token");
        let strategy = synthesizer.select_strategy(&existence_constraint);
        assert!(strategy.is_ok());
        assert!(matches!(strategy.unwrap(), SynthesisStrategy::Transform));
    }

    #[test]
    fn test_output_binding_extraction() {
        let location = Location::Remote("test_domain".to_string());
        let synthesizer = FlowSynthesizer::new(location);
        
        let constraint = Constraint::And(vec![
            Constraint::produces("token_out", "Token"),
            Constraint::produces_quantity("fees", "Token", 5),
        ]);
        
        let outputs = synthesizer.extract_output_bindings(&constraint);
        assert_eq!(outputs.len(), 2);
        assert!(outputs.iter().any(|b| b.name == "token_out"));
        assert!(outputs.iter().any(|b| b.name == "fees"));
    }

    #[test]
    fn test_expanded_effect_library() {
        let library = EffectLibrary::default();
        
        // Test that we have both basic and advanced templates
        assert!(library.templates.contains_key("transfer"));
        assert!(library.templates.contains_key("swap"));
        assert!(library.templates.contains_key("add_liquidity"));
        assert!(library.templates.contains_key("stake"));
        assert!(library.templates.contains_key("lend"));
        assert!(library.templates.contains_key("borrow"));
        
        // Test swap template specifically
        let swap_template = &library.templates["swap"];
        assert_eq!(swap_template.inputs.len(), 2); // TokenA + LiquidityPool
        assert_eq!(swap_template.outputs.len(), 2); // TokenB + Updated Pool
        assert_eq!(swap_template.cost, 300);
        
        // Test liquidity template
        let liquidity_template = &library.templates["add_liquidity"];
        assert_eq!(liquidity_template.inputs.len(), 2); // TokenA + TokenB
        assert_eq!(liquidity_template.outputs.len(), 2); // LP tokens + Updated Pool
    }

    #[test]
    fn test_defi_focused_library() {
        let library = EffectLibrary::defi_focused();
        
        // Should contain DeFi-specific templates
        assert!(library.templates.contains_key("swap"));
        assert!(library.templates.contains_key("add_liquidity"));
        assert!(library.templates.contains_key("remove_liquidity"));
        assert!(library.templates.contains_key("stake"));
        assert!(library.templates.contains_key("unstake"));
        assert!(library.templates.contains_key("lend"));
        assert!(library.templates.contains_key("borrow"));
        assert!(library.templates.contains_key("repay"));
        
        // Check that templates have appropriate DeFi characteristics
        let swap_template = &library.templates["swap"];
        assert!(swap_template.cost > 200); // DeFi operations should be more expensive
    }

    #[test]
    fn test_template_matching() {
        let library = EffectLibrary::default();
        
        let location = Location::Remote("test_domain".to_string());
        let intent = Intent::new(
            location,
            vec![
                ResourceBinding::new("token_a", "TokenA").with_quantity(100),
                ResourceBinding::new("pool", "LiquidityPool"),
            ],
            Constraint::produces("token_b", "TokenB"),
        );
        
        let matching_templates = library.find_matching_templates(&intent);
        
        // Should match swap template due to TokenA input and TokenB output
        assert!(!matching_templates.is_empty());
        let has_swap = matching_templates.iter().any(|t| t.name == "swap");
        assert!(has_swap, "Swap template should match this intent");
    }

    #[test]
    fn test_swap_template_matching() {
        let library = EffectLibrary::default();
        
        // Test intent that should match swap template
        let location = Location::Remote("defi_domain".to_string());
        let swap_intent = Intent::new(
            location,
            vec![
                ResourceBinding::new("input_tokens", "TokenA").with_quantity(100),
                ResourceBinding::new("dex_pool", "LiquidityPool"),
            ],
            Constraint::And(vec![
                Constraint::produces_quantity("output_tokens", "TokenB", 90),
                Constraint::produces("updated_pool", "LiquidityPool"),
            ])
        );
        
        let matching = library.find_matching_templates(&swap_intent);
        assert!(!matching.is_empty());
        
        let swap_template = matching.iter().find(|t| t.name == "swap");
        assert!(swap_template.is_some(), "Should find swap template");
        
        // Test intent that should NOT match swap (missing pool)
        let non_swap_intent = Intent::new(
            Location::Remote("test_domain".to_string()),
            vec![
                ResourceBinding::new("simple_token", "Token").with_quantity(50),
            ],
            Constraint::produces("other_token", "Token"),
        );
        
        let non_matching = library.find_matching_templates(&non_swap_intent);
        let has_swap = non_matching.iter().any(|t| t.name == "swap");
        assert!(!has_swap, "Should not match swap template without pool input");
    }

    #[test]
    fn test_complex_constraint_output_extraction() {
        let location = Location::Remote("test_domain".to_string());
        let synthesizer = FlowSynthesizer::new(location);
        
        let complex_constraint = Constraint::And(vec![
            Constraint::Or(vec![
                Constraint::produces("option_a", "TokenA"),
                Constraint::produces("option_b", "TokenB"),
            ]),
            Constraint::And(vec![
                Constraint::produces_quantity("fee", "Token", 10),
                Constraint::Not(Box::new(Constraint::produces("invalid", "Invalid"))),
            ]),
            Constraint::ExistsAll(vec![
                ResourceBinding::new("multi_1", "Token"),
                ResourceBinding::new("multi_2", "Token"),
            ]),
        ]);
        
        let outputs = synthesizer.extract_output_bindings(&complex_constraint);
        
        // Should extract: option_a, option_b, fee, multi_1, multi_2
        // Note: invalid should NOT be extracted due to Not wrapper
        assert!(outputs.len() >= 5);
        
        let output_names: Vec<&str> = outputs.iter().map(|b| b.name.as_str()).collect();
        assert!(output_names.contains(&"option_a"));
        assert!(output_names.contains(&"option_b"));
        assert!(output_names.contains(&"fee"));
        assert!(output_names.contains(&"multi_1"));
        assert!(output_names.contains(&"multi_2"));
        assert!(!output_names.contains(&"invalid"));
        
        // Check that quantities are preserved
        let fee_binding = outputs.iter().find(|b| b.name == "fee").unwrap();
        assert_eq!(fee_binding.quantity, Some(10));
    }

    #[test]
    fn test_session_synthesis() {
        let location = Location::Remote("test_domain".to_string());
        let synthesizer = FlowSynthesizer::new(location.clone());
        
        let session_requirement = SessionRequirement::new(
            "PaymentProtocol",
            "client",
            SessionType::Send(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(SessionType::Receive(
                    Box::new(TypeInner::Base(BaseType::Bool)),
                    Box::new(SessionType::End)
                ))
            )
        );
        
        let intent = Intent::new(
            location,
            vec![ResourceBinding::new("payment_data", "PaymentRequest")],
            Constraint::session_compliant("PaymentProtocol", "client"),
        ).with_session_requirement(session_requirement);
        
        let result = synthesizer.synthesize_session_effects(&intent);
        assert!(result.is_ok());
        
        let effects = result.unwrap();
        assert!(!effects.is_empty());
        
        // Should contain session-related effects
        let has_session_effects = effects.iter().any(|effect| {
            matches!(effect.kind, 
                EffectExprKind::SessionSend { .. } | 
                EffectExprKind::SessionReceive { .. } |
                EffectExprKind::WithSession { .. }
            )
        });
        assert!(has_session_effects);
    }

    #[test]
    fn test_session_protocol_compilation() {
        let location = Location::Remote("test_domain".to_string());
        let synthesizer = FlowSynthesizer::new(location);
        
        let send_protocol = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let recv_protocol = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let choice_protocol = SessionType::InternalChoice(vec![
            ("choice_0".to_string(), SessionType::End),
            ("choice_1".to_string(), SessionType::Send(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(SessionType::End)
            )),
        ]);
        
        let send_req = SessionRequirement::new("test", "sender", send_protocol);
        let recv_req = SessionRequirement::new("test", "receiver", recv_protocol);
        let choice_req = SessionRequirement::new("test", "chooser", choice_protocol);
        
        // Test compilation of each protocol type
        let send_result = synthesizer.compile_session_to_effects(&send_req);
        assert!(send_result.is_ok());
        
        let recv_result = synthesizer.compile_session_to_effects(&recv_req);
        assert!(recv_result.is_ok());
        
        let choice_result = synthesizer.compile_session_to_effects(&choice_req);
        assert!(choice_result.is_ok());
    }

    #[test]
    fn test_session_compilation_error_handling() {
        let location = Location::Remote("test_domain".to_string());
        let synthesizer = FlowSynthesizer::new(location);
        
        // Test empty internal choice (should error)
        let empty_choice = SessionType::InternalChoice(vec![]);
        let empty_req = SessionRequirement::new("test", "role", empty_choice);
        
        let result = synthesizer.compile_session_to_effects(&empty_req);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SynthesisError::UnsupportedIntent(_)));
    }
} 