//! Tests for the rich capability system

use causality_core::effect::capability::{
    Capability, CapabilityLevel, CapabilitySet,
};
use ssz::{Encode, Decode};

#[test]
fn test_capability_levels() {
    // Test all capability level constructors
    let read_cap = Capability::read("file");
    let write_cap = Capability::write("file");
    let execute_cap = Capability::execute("script");
    let admin_cap = Capability::admin("system");
    
    assert_eq!(read_cap.level, CapabilityLevel::Read);
    assert_eq!(write_cap.level, CapabilityLevel::Write);
    assert_eq!(execute_cap.level, CapabilityLevel::Execute);
    assert_eq!(admin_cap.level, CapabilityLevel::Admin);
}

#[test]
fn test_capability_implication_hierarchy() {
    let file_admin = Capability::admin("file");
    let file_write = Capability::write("file");
    let file_execute = Capability::execute("file");
    let file_read = Capability::read("file");
    
    // Admin implies everything
    assert!(file_admin.implies(&file_write));
    assert!(file_admin.implies(&file_execute));
    assert!(file_admin.implies(&file_read));
    assert!(file_admin.implies(&file_admin));
    
    // Write implies read but not execute or admin
    assert!(file_write.implies(&file_read));
    assert!(!file_write.implies(&file_execute));
    assert!(!file_write.implies(&file_admin));
    
    // Execute implies read but not write or admin
    assert!(file_execute.implies(&file_read));
    assert!(!file_execute.implies(&file_write));
    assert!(!file_execute.implies(&file_admin));
    
    // Read only implies itself
    assert!(file_read.implies(&file_read));
    assert!(!file_read.implies(&file_write));
    assert!(!file_read.implies(&file_execute));
    assert!(!file_read.implies(&file_admin));
}

#[test]
fn test_capability_resource_isolation() {
    let file_admin = Capability::admin("file");
    let db_read = Capability::read("database");
    
    // Different resources don't imply each other, even with higher privileges
    assert!(!file_admin.implies(&db_read));
    assert!(!db_read.implies(&file_admin));
}

#[test]
fn test_capability_set_operations() {
    let mut cap_set = CapabilitySet::new();
    
    // Add capabilities
    cap_set.add(Capability::write("config"));
    cap_set.add(Capability::execute("script"));
    cap_set.add(Capability::admin("logs"));
    
    // Test implied capabilities
    assert!(cap_set.has_capability(&Capability::read("config"))); // Write implies read
    assert!(cap_set.has_capability(&Capability::write("config")));
    
    assert!(cap_set.has_capability(&Capability::read("script"))); // Execute implies read
    assert!(cap_set.has_capability(&Capability::execute("script")));
    
    assert!(cap_set.has_capability(&Capability::read("logs"))); // Admin implies read
    assert!(cap_set.has_capability(&Capability::write("logs"))); // Admin implies write
    assert!(cap_set.has_capability(&Capability::execute("logs"))); // Admin implies execute
    assert!(cap_set.has_capability(&Capability::admin("logs")));
    
    // Test missing capabilities
    assert!(!cap_set.has_capability(&Capability::admin("config"))); // Only have write
    assert!(!cap_set.has_capability(&Capability::write("script"))); // Only have execute
    assert!(!cap_set.has_capability(&Capability::read("other"))); // Different resource
}

#[test]
fn test_capability_set_multiple_requirements() {
    let cap_set = CapabilitySet::from_capabilities(vec![
        Capability::admin("system"),
        Capability::write("config"),
        Capability::read("logs"),
    ]);
    
    // All these should be satisfied
    let requirements = vec![
        Capability::write("system"), // Admin implies write
        Capability::read("config"),  // Write implies read
        Capability::read("logs"),    // Exact match
    ];
    
    assert!(cap_set.has_all_capabilities(&requirements));
    
    // This should fail
    let requirements_with_missing = vec![
        Capability::write("system"),
        Capability::admin("config"), // Only have write, not admin
        Capability::read("logs"),
    ];
    
    assert!(!cap_set.has_all_capabilities(&requirements_with_missing));
}

#[test]
fn test_capability_level_implications() {
    use CapabilityLevel::*;
    
    assert_eq!(Read.implies(), vec![Read]);
    assert_eq!(Write.implies(), vec![Read, Write]);
    assert_eq!(Execute.implies(), vec![Read, Execute]);
    assert_eq!(Admin.implies(), vec![Read, Write, Execute, Admin]);

    // Test level implications
    assert!(Admin.implies_level(&Read));
    assert!(Admin.implies_level(&Write));
    assert!(Admin.implies_level(&Execute));
    assert!(Admin.implies_level(&Admin));

    assert!(Write.implies_level(&Read));
    assert!(!Write.implies_level(&Execute));

    assert!(Execute.implies_level(&Read));
    assert!(!Execute.implies_level(&Write));

    assert!(!Read.implies_level(&Write));
    assert!(!Read.implies_level(&Execute));
    assert!(!Read.implies_level(&Admin));
}

#[test]
fn test_capability_serialization() {
    // Test capability level serialization
    let level = CapabilityLevel::Admin;
    let encoded = level.as_ssz_bytes();
    let decoded = CapabilityLevel::from_ssz_bytes(&encoded).unwrap();
    assert_eq!(level, decoded);
    
    // Test capability serialization
    let cap = Capability::write("test_resource");
    let encoded = cap.as_ssz_bytes();
    let decoded = Capability::from_ssz_bytes(&encoded).unwrap();
    assert_eq!(cap, decoded);
    
    // Test all levels
    for level in [CapabilityLevel::Read, CapabilityLevel::Write, CapabilityLevel::Execute, CapabilityLevel::Admin] {
        let cap = Capability::new("test", level.clone());
        let encoded = cap.as_ssz_bytes();
        let decoded = Capability::from_ssz_bytes(&encoded).unwrap();
        assert_eq!(cap, decoded);
    }
}

#[test]
fn test_real_world_scenarios() {
    // Scenario: File system permissions
    let mut file_system = CapabilitySet::new();
    file_system.add(Capability::read("/etc/config"));
    file_system.add(Capability::write("/tmp/cache"));
    file_system.add(Capability::execute("/usr/bin/tool"));
    file_system.add(Capability::admin("/var/log"));
    
    // User can read config
    assert!(file_system.has_capability(&Capability::read("/etc/config")));
    // User cannot write to config (only read)
    assert!(!file_system.has_capability(&Capability::write("/etc/config")));
    
    // User can read and write to cache
    assert!(file_system.has_capability(&Capability::read("/tmp/cache")));
    assert!(file_system.has_capability(&Capability::write("/tmp/cache")));
    
    // User can read and execute tool
    assert!(file_system.has_capability(&Capability::read("/usr/bin/tool")));
    assert!(file_system.has_capability(&Capability::execute("/usr/bin/tool")));
    // But cannot write to it
    assert!(!file_system.has_capability(&Capability::write("/usr/bin/tool")));
    
    // User has full admin access to logs
    assert!(file_system.has_capability(&Capability::read("/var/log")));
    assert!(file_system.has_capability(&Capability::write("/var/log")));
    assert!(file_system.has_capability(&Capability::execute("/var/log")));
    assert!(file_system.has_capability(&Capability::admin("/var/log")));
    
    // Scenario: Database permissions
    let db_caps = CapabilitySet::from_capabilities(vec![
        Capability::read("users_table"),
        Capability::write("sessions_table"),
        Capability::admin("logs_table"),
    ]);
    
    // Can perform complex operations requiring multiple capabilities
    let required_for_user_session = vec![
        Capability::read("users_table"),
        Capability::write("sessions_table"),
    ];
    assert!(db_caps.has_all_capabilities(&required_for_user_session));
    
    // Cannot perform admin operations on user data
    let admin_operation = vec![
        Capability::admin("users_table"),
    ];
    assert!(!db_caps.has_all_capabilities(&admin_operation));
}

#[test]
fn test_capability_set_conversion() {
    let rich_set = CapabilitySet::from_capabilities(vec![
        Capability::admin("system"),
        Capability::write("config"),
        Capability::read("logs"),
    ]);
    
    // Test that the set properly contains all expected capabilities
    assert!(rich_set.has_capability(&Capability::admin("system")));
    assert!(rich_set.has_capability(&Capability::write("config")));
    assert!(rich_set.has_capability(&Capability::read("logs")));
    
    // Test implication - admin("system") implies write("system") and read("system")
    assert!(rich_set.has_capability(&Capability::write("system")));
    assert!(rich_set.has_capability(&Capability::read("system")));
    
    // Test that write("config") implies read("config")
    assert!(rich_set.has_capability(&Capability::read("config")));
} 