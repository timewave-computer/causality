//! Tests for witness generation using SSZ serialization

#[cfg(test)]
mod tests {
    use super::super::*;
    use causality_types::{
        core::id::DomainId,
        core::str::Str,
        expr::value::ValueExpr,
        resource::Resource,
        serialization::{Encode, Decode},
        core::{Timestamp, id::EntityId},
    };

    /// Test basic witness generation with SSZ serialization
    #[test]
    #[ignore] // TODO: Fix SSZ serialization for Resource types
    fn test_basic_witness_generation() {
        // Create a sample resource
        let resource = Resource {
            id: EntityId::new([0u8; 32]),
            name: Str::from("test_resource"),
            domain_id: DomainId::new([0u8; 32]),
            resource_type: Str::from("test_type"),
            quantity: 100,
            timestamp: Timestamp::now(),
        };

        // Create a sample value expression
        let value_expr = ValueExpr::String(Str::from("test value"));

        // Create a witness generator and add the resource and value
        let mut generator = WitnessGenerator::new();
        generator.add_resource(resource.clone());
        generator.add_value_expr(value_expr.clone());

        // Generate circuit inputs
        let inputs = generator.generate_circuit_inputs().unwrap();

        // Verify the inputs
        assert_eq!(inputs.len(), 2);
        
        // Check the resource input
        let resource_input = &inputs[0];
        assert_eq!(resource_input.metadata.input_type, ssz_input::SszInputType::Resource);
        let recovered_resource = resource_input.try_as_resource().unwrap();
        assert_eq!(recovered_resource.id, resource.id);
        
        // Check the value input
        let value_input = &inputs[1];
        assert_eq!(value_input.metadata.input_type, ssz_input::SszInputType::ValueExpr);
        let recovered_value = value_input.try_as_value_expr().unwrap();
        assert_eq!(recovered_value, value_expr);
    }

    /// Test generating a Merkle root from witness data
    #[test]
    fn test_merkle_root_generation() {
        // Create a witness generator with some test data
        let mut generator = WitnessGenerator::new();
        
        // Add a resource
        let resource = Resource {
            id: EntityId::new([0u8; 32]),
            name: Str::from("test_resource"),
            domain_id: DomainId::new([0u8; 32]),
            resource_type: Str::from("test_type"),
            quantity: 100,
            timestamp: Timestamp::now(),
        };
        generator.add_resource(resource);
        
        // Add a value expression
        let value_expr = ValueExpr::String(Str::from("test value"));
        generator.add_value_expr(value_expr);
        
        // Add some raw input
        let raw_input = vec![1, 2, 3, 4, 5];
        generator.add_raw_input(raw_input);
        
        // Generate a Merkle root
        let root = generator.generate_merkle_root().unwrap();
        
        // The root should not be all zeros
        assert_ne!(root, [0u8; 32]);
        
        // Generate circuit inputs
        let inputs = generator.generate_circuit_inputs().unwrap();
        assert_eq!(inputs.len(), 3);
    }

    /// Test the SSZ serialization roundtrip for circuit inputs
    #[test]
    fn test_ssz_serialization_roundtrip() {
        // Create a value expression
        let value_expr = ValueExpr::String(Str::from("test value"));
        
        // Serialize it using SSZ
        let serialized = value_expr.as_ssz_bytes();
        
        // Deserialize it using SSZ
        let deserialized = ValueExpr::from_ssz_bytes(&serialized).unwrap();
        
        // Check that we got the same value back
        assert_eq!(deserialized, value_expr);
    }
} 