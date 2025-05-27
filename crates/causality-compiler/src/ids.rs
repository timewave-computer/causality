//-----------------------------------------------------------------------------
// ID Generation and Utility Functions
//-----------------------------------------------------------------------------

pub use causality_types::primitive::ids::{AsId, CircuitId, ProgramId};
use sha2::{Digest, Sha256};
use uuid::Uuid;

//-----------------------------------------------------------------------------
// ID Generation Logic
//-----------------------------------------------------------------------------

/// Generates a unique CircuitId.
/// The `circuit_data_hash` is a pre-computed hash representing the circuit's unique properties.
/// The `domain_specific_seed` can be used to ensure ID uniqueness across different domains
/// or contexts if the same `circuit_data_hash` might occur.
pub fn generate_circuit_id(circuit_data_hash: &[u8; 32], _domain_specific_seed: &[u8]) -> CircuitId {
    CircuitId::new(*circuit_data_hash)
}

/// Generates a unique ProgramId.
/// `circuit_id_hashes` is a collection of hashes derived from the CircuitIds constituting the program.
pub fn generate_program_id(circuit_id_hashes: &[[u8; 32]]) -> ProgramId {
    let mut hasher = Sha256::new();
    for hash_val in circuit_id_hashes {
        hasher.update(hash_val);
    }
    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result[..]);
    ProgramId::new(bytes)
}

//-----------------------------------------------------------------------------
// Helper for creating IDs from UUIDs
//-----------------------------------------------------------------------------

/// Creates an ID from a UUID. The UUID bytes are directly used as the ID.
/// Ensures that the ID type `T` implements `AsId` for type safety.
pub fn id_from_uuid<T: AsId>(uuid: Uuid) -> T {
    let mut bytes = [0u8; 32];
    bytes[0..16].copy_from_slice(uuid.as_bytes());
    T::new(bytes)
}

//-----------------------------------------------------------------------------
// Helper for creating IDs from SHA256 Hashes
//-----------------------------------------------------------------------------

/// Creates an ID from a SHA256 hash of the given data.
/// Ensures that the ID type `T` implements `AsId` for type safety.
pub fn id_from_sha256<T: AsId>(data: &[u8]) -> T {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result[..]);
    T::new(bytes)
}


//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_circuit_id_basic() {
        let details = [1u8; 32];
        let seed = [0u8;0]; 
        let circuit_id = generate_circuit_id(&details, &seed);
        assert_eq!(circuit_id.inner(), details, "CircuitId should match input hash");
        println!("Generated Circuit ID: {}", circuit_id.to_hex());
    }

    #[test]
    fn test_generate_program_id_basic() {
        let circuit_hash1 = [2u8; 32];
        let circuit_hash2 = [3u8; 32];
        let program_id = generate_program_id(&[circuit_hash1, circuit_hash2]);
        println!("Generated Program ID: {}", program_id.to_hex());
        assert_ne!(program_id.inner(), circuit_hash1);
        assert_ne!(program_id.inner(), circuit_hash2);
    }
}
