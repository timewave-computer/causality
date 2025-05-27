#[cfg(test)]
mod tests {
    use crate::core::resource::Resource;
    use crate::primitive::ids::{EntityId, DomainId, AsId};
    use crate::primitive::string::Str;
    use crate::core::time::Timestamp;
    

    fn create_test_resource() -> Resource {
        Resource {
            id: EntityId::new([1u8; 32]),
            name: Str::from("test_resource"),
            domain_id: DomainId::new([2u8; 32]),
            resource_type: Str::from("token"),
            quantity: 100,
            timestamp: Timestamp {
                domain_id: DomainId::new([2u8; 32]),
                logical: 0,
                wall: crate::core::time::WallClock(1000000),
            },
        }
    }

    #[test]
    fn test_resource_creation() {
        let resource = create_test_resource();
        
        assert!(!resource.name.is_empty());
        assert!(!resource.domain_id.is_null());
        assert!(!resource.resource_type.is_empty());
    }

    #[test]
    fn test_resource_with_different_types() {
        let token_resource = Resource {
            id: EntityId::new([1u8; 32]),
            name: Str::from("token_resource"),
            domain_id: DomainId::new([2u8; 32]),
            resource_type: Str::from("token"),
            quantity: 100,
            timestamp: Timestamp {
                domain_id: DomainId::new([2u8; 32]),
                logical: 0,
                wall: crate::core::time::WallClock(1000000),
            },
        };
        
        let compute_resource = Resource {
            id: EntityId::new([3u8; 32]),
            name: Str::from("compute_resource"),
            domain_id: DomainId::new([2u8; 32]),
            resource_type: Str::from("compute"),
            quantity: 50,
            timestamp: Timestamp {
                domain_id: DomainId::new([2u8; 32]),
                logical: 0,
                wall: crate::core::time::WallClock(1000000),
            },
        };
        
        assert_eq!(token_resource.resource_type, Str::from("token"));
        assert_eq!(compute_resource.resource_type, Str::from("compute"));
        assert_ne!(token_resource.name, compute_resource.name);
    }

    #[test]
    fn test_resource_equality() {
        let resource1 = create_test_resource();
        let resource2 = create_test_resource();
        
        // Resources with same data should be equal
        assert_eq!(resource1, resource2);
    }

    #[test]
    fn test_resource_debug_display() {
        let resource = create_test_resource();
        let debug_str = format!("{:?}", resource);
        assert!(debug_str.contains("Resource"));
        assert!(debug_str.contains("100"));  // Check for the quantity value instead
    }
} 