use causality::error::Result;
use causality::memory::{Memory, MemoryAllocator, MemoryMapper};

#[test]
fn test_memory_basics() {
    let mut memory = Memory::new(1024);

    // Test byte operations
    memory.store_byte(100, 42).unwrap();
    assert_eq!(memory.load_byte(100).unwrap(), 42);

    // Test out of bounds
    assert!(memory.store_byte(2000, 42).is_err());
    assert!(memory.load_byte(2000).is_err());
}

#[test]
fn test_memory_word_operations() {
    let mut memory = Memory::new(1024);

    // Test word operations
    memory.store_word(100, 0x12345678).unwrap();
    assert_eq!(memory.load_word(100).unwrap(), 0x12345678);

    // Test that bytes are stored in little-endian order
    assert_eq!(memory.load_byte(100).unwrap(), 0x78);
    assert_eq!(memory.load_byte(101).unwrap(), 0x56);
    assert_eq!(memory.load_byte(102).unwrap(), 0x34);
    assert_eq!(memory.load_byte(103).unwrap(), 0x12);

    // Test halfword operations
    memory.store_halfword(200, 0xABCD).unwrap();
    assert_eq!(memory.load_halfword(200).unwrap(), 0xABCD);

    // Test doubleword operations
    memory.store_doubleword(300, 0x123456789ABCDEF0).unwrap();
    assert_eq!(memory.load_doubleword(300).unwrap(), 0x123456789ABCDEF0);
}

#[test]
fn test_memory_fill_and_copy() {
    let mut memory = Memory::new(1024);

    // Test fill
    memory.fill(100, 150, 0xAA).unwrap();
    for i in 100..=150 {
        assert_eq!(memory.load_byte(i).unwrap(), 0xAA);
    }

    // Test copy
    memory.store_byte(200, 0x11).unwrap();
    memory.store_byte(201, 0x22).unwrap();
    memory.store_byte(202, 0x33).unwrap();

    memory.copy(200, 300, 3).unwrap();

    assert_eq!(memory.load_byte(300).unwrap(), 0x11);
    assert_eq!(memory.load_byte(301).unwrap(), 0x22);
    assert_eq!(memory.load_byte(302).unwrap(), 0x33);

    // Test overlapping copy (should still work)
    memory.copy(200, 201, 2).unwrap();
    assert_eq!(memory.load_byte(201).unwrap(), 0x11);
    assert_eq!(memory.load_byte(202).unwrap(), 0x22);
}

#[test]
fn test_memory_slice_operations() {
    let mut memory = Memory::new(1024);

    // Test store slice
    let data = [0x11, 0x22, 0x33, 0x44, 0x55];
    memory.store_slice(100, &data).unwrap();

    // Test load slice
    let loaded = memory.load_slice(100, 5).unwrap();
    assert_eq!(loaded, data);
}

#[test]
fn test_memory_index_operations() {
    let mut memory = Memory::new(1024);

    // Test index operations
    memory[100] = 42;
    assert_eq!(memory[100], 42);

    // Test indexing out of bounds via panicking
    let result = std::panic::catch_unwind(|| {
        memory[2000] = 42;
    });
    assert!(result.is_err());

    let result = std::panic::catch_unwind(|| {
        let _ = memory[2000];
    });
    assert!(result.is_err());
}

#[test]
fn test_memory_mapper() {
    let mut mapper = MemoryMapper::new();

    // Add mappings
    mapper.add_mapping(0x10000, 0x0, 0x1000); // 0x10000-0x10FFF -> 0x0-0xFFF
    mapper.add_mapping(0x20000, 0x1000, 0x1000); // 0x20000-0x20FFF -> 0x1000-0x1FFF

    // Test valid mappings
    assert_eq!(mapper.map(0x10000).unwrap(), 0x0);
    assert_eq!(mapper.map(0x10FFF).unwrap(), 0xFFF);
    assert_eq!(mapper.map(0x20000).unwrap(), 0x1000);
    assert_eq!(mapper.map(0x20FFF).unwrap(), 0x1FFF);

    // Test invalid mapping
    assert!(mapper.map(0x30000).is_err());
}

#[test]
fn test_memory_allocator() {
    let mut allocator = MemoryAllocator::new(0x1000, 0x1000);

    // Test allocation
    let addr1 = allocator.allocate(100, 8).unwrap();
    assert!(addr1 >= 0x1000);

    // Test alignment
    let addr2 = allocator.allocate(100, 16).unwrap();
    assert_eq!(addr2 % 16, 0);

    // Test allocation size tracking
    assert_eq!(allocator.get_allocation_size(addr1).unwrap(), 100);

    // Test deallocation
    assert!(allocator.deallocate(addr1).is_ok());
    assert!(allocator.get_allocation_size(addr1).is_none());

    // Test invalid deallocation
    assert!(allocator.deallocate(0x5000).is_err());

    // Test out of memory
    let large_size = 0x2000; // Larger than available memory
    assert!(allocator.allocate(large_size, 8).is_err());

    // Test reset
    allocator.reset();
    assert!(allocator.get_allocation_size(addr2).is_none());
}
