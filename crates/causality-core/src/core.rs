use std::collections::BTreeMap;
// use std::sync::Arc; // Unused

use causality_types::{
// ... existing code ...
    fn resolve_symbol(&self, name: &Str) -> Option<ExprResult>;

    /// Set a symbol in the current scope.
    /// This is a convenience method for direct manipulation, typically used by host functions or setup.
    fn set_symbol(&mut self, _name: Str, value: ExprResult); // _name prefixed

    /// Get a resource by its ID.
// ... existing code ...

#[derive(Default)]
pub struct DefaultExprContext {
    pub symbols: BTreeMap<Str, ExprResult>,
    pub resources: BTreeMap<ResourceId, Resource>,
    // Removed Arc imports as no longer used here directly
}

impl DefaultExprContext {
    pub fn new() -> Self {
        DefaultExprContext::default()
    }

    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.insert(resource.id.clone(), resource);
    }
}

impl AsExprContext for DefaultExprContext {
    fn resolve_symbol(&self, name: &Str) -> Option<ExprResult> {
        self.symbols.get(name).cloned()
    }

    fn set_symbol(&mut self, _name: Str, value: ExprResult) { // _name prefixed
        self.symbols.insert(_name, value); // Use _name if it were not ignored, but it is.
    }

    fn get_resource(&self, id: &ResourceId) -> Option<Resource> {
// ... existing code ...
    }
} 