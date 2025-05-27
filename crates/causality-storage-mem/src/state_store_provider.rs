pub struct InMemoryStateStoreProvider;

impl StateStoreProvider for InMemoryStateStoreProvider {
    fn create_store(&self) -> Arc<dyn ResourceStateStore> {
        // TODO: Need to import the moved InMemoryStateStore here
        // Arc::new(InMemoryStateStore::new())
        unimplemented!("InMemoryStateStore needs to be imported/created here");
    }
}

/// Default state store provider (currently in-memory)
pub struct DefaultStateStoreProvider;

impl StateStoreProvider for DefaultStateStoreProvider {
    fn create_store(&self) -> Arc<dyn ResourceStateStore> {
        // TODO: Need to import the moved InMemoryStateStore here
        // Arc::new(InMemoryStateStore::new())
        unimplemented!("InMemoryStateStore needs to be imported/created here");
    }
}

/// Check if a state transition is valid
/// TODO: Define this logic more formally
fn is_valid_state_transition(from: &ResourceState, to: &ResourceState) -> bool {
    // Use fully qualified enum variants
    match (from, to) {
