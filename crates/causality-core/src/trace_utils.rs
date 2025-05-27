// Purpose: Utility functions for working with the Trace type from causality-types.

// use ssz::sszSerialize; // Unused
// use causality_types::primitive::ids::AsId; // Unused
// use causality_types::primitive::ids::EffectId; // Moved to test module
use causality_types::primitive::ids::{AsId, EffectId, ResourceId};
use causality_types::state::ResourceState;
use causality_types::trace::ZkExecutionTrace;
use causality_types::serialization::Encode;
use sha2::{Digest, Sha256};

/// Computes and sets the trace hash for a ZkExecutionTrace.
/// This function is moved from the ZkExecutionTrace impl in causality-types.
pub fn zk_execution_trace_compute_hash(trace: &mut ZkExecutionTrace) -> [u8; 32] {
    let mut hasher = Sha256::new();

    // Hash all input resources
    for resource_state in &trace.input_resources {
        let bytes = resource_state.as_ssz_bytes();
        hasher.update(&bytes);
    }

    // Hash all output resources
    for resource_state in &trace.output_resources {
        let bytes = resource_state.as_ssz_bytes();
        hasher.update(&bytes);
    }

    // Hash all effect IDs
    for effect_id in &trace.effect_ids {
        hasher.update(effect_id.inner());
    }

    // Hash metadata
    hasher.update(trace.metadata.execution_id.as_bytes());
    hasher.update(trace.metadata.timestamp.to_le_bytes());

    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);

    trace.metadata.trace_hash = Some(hash);
    hash
}

/// Creates a unique trace ID by hashing the state transition.
pub fn create_trace_id(
    resource_id: &ResourceId,
    resource_state: &ResourceState,
    effect_id: &EffectId,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(resource_id.inner());

    // Serialize the resource state using SSZ
    let bytes = resource_state.as_ssz_bytes();
    hasher.update(&bytes);

    // Alternative approach if SSZ isn't available for ResourceState
    // hasher.update(&format!("{:?}", resource_state).as_bytes());

    hasher.update(effect_id.inner());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::primitive::ids::EffectId; // Import EffectId for tests
    use causality_types::{
        state::ResourceState,
        trace::{ZkExecutionMetadata, ZkExecutionTrace},
    };

    #[test]
    fn test_zk_trace_hash_computation() {
        let mut trace = ZkExecutionTrace {
            input_resources: vec![ResourceState::Available],
            output_resources: vec![ResourceState::Consumed],
            effect_ids: vec![EffectId::new([3u8; 32])],
            metadata: ZkExecutionMetadata {
                execution_id: "test_exec_123".to_string(),
                timestamp: 1234567890,
                trace_hash: None,
            },
        };

        let hash1 = zk_execution_trace_compute_hash(&mut trace);
        assert!(trace.metadata.trace_hash.is_some());
        assert_eq!(trace.metadata.trace_hash.unwrap(), hash1);

        // Ensure it's deterministic
        let mut trace2 = ZkExecutionTrace {
            input_resources: vec![ResourceState::Available],
            output_resources: vec![ResourceState::Consumed],
            effect_ids: vec![EffectId::new([3u8; 32])],
            metadata: ZkExecutionMetadata {
                execution_id: "test_exec_123".to_string(),
                timestamp: 1234567890,
                trace_hash: None,
            },
        };
        let hash2 = zk_execution_trace_compute_hash(&mut trace2);
        assert_eq!(hash1, hash2);

        // Change data and ensure hash changes
        trace2.metadata.timestamp = 11111;
        let hash3 = zk_execution_trace_compute_hash(&mut trace2);
        assert_ne!(hash1, hash3);
    }
}
