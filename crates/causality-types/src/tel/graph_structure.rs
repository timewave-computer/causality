// Purpose: Defines the graph structure for the TEL (Temporal Effect Language).

use crate::{
    core::{Effect, Intent, Handler},
    core::str::Str,
    tel::graph::Edge,
};

/// Represents a complete Temporal Effect Language (TEL) graph,
/// which is the primary "program" executed by the TEL interpreter.
///
/// It contains collections of intents, effects, handlers, and the edges
/// that define their relationships.
#[derive(Debug, Clone, Default)]
pub struct EffectGraph {
    /// Optional identifier or name for this graph.
    pub id: Option<Str>,

    pub intents: Vec<Intent>,
    pub effects: Vec<Effect>,
    pub handlers: Vec<Handler>,
    pub edges: Vec<Edge>,
}

impl EffectGraph {
    pub fn new(id: Option<Str>) -> Self {
        Self {
            id,
            intents: Vec::new(),
            effects: Vec::new(),
            handlers: Vec::new(),
            edges: Vec::new(),
        }
    }
}
