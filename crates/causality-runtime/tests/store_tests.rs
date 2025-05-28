//-----------------------------------------------------------------------------
// Runtime Store Test
//-----------------------------------------------------------------------------

use causality_runtime::store::RuntimeValueStore;
use causality_types::{
    expression::value::{ValueExpr, Number},
    primitive::string::Str,
};

//-----------------------------------------------------------------------------
// Helper Function REMOVED (create_test_resource was for ResourceStore)
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// Resource Store Test REMOVED
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// Value Store Test
//-----------------------------------------------------------------------------

#[tokio::test]
async fn test_value_store_creation() {
    let store = RuntimeValueStore::new();
    assert_eq!(store.len(), 0);
}

#[tokio::test]
async fn test_store_and_retrieve_values() {
    let mut store = RuntimeValueStore::new();
    
    let value1 = ValueExpr::Number(Number::Integer(42));
    let value2 = ValueExpr::String(Str::from("test"));
    
    let id1 = store.add_value(value1.clone()).unwrap();
    let id2 = store.add_value(value2.clone()).unwrap();
    
    let retrieved1 = store.lookup(&hex::encode(id1.0)).await.unwrap().unwrap();
    let retrieved2 = store.lookup(&hex::encode(id2.0)).await.unwrap().unwrap();
    
    assert_eq!(retrieved1, value1);
    assert_eq!(retrieved2, value2);
    
    assert_eq!(store.len(), 2);
}

#[tokio::test]
async fn test_value_removal() {
    let mut store = RuntimeValueStore::new();
    
    let value = ValueExpr::Number(Number::Integer(100));
    let id = store.add_value(value.clone()).unwrap();
    
    let retrieved = store.lookup(&hex::encode(id.0)).await.unwrap().unwrap();
    assert_eq!(retrieved, value);
    
    store.unregister(&hex::encode(id.0)).await.unwrap();
    
    let result = store.lookup(&hex::encode(id.0)).await.unwrap();
    assert!(result.is_none(), "Value should be removed");
    
    assert_eq!(store.len(), 0, "Store should be empty after removal");
}

#[tokio::test]
async fn test_multiple_values() {
    let mut store = RuntimeValueStore::new();
    
    let value2 = ValueExpr::String(Str::from("hello"));
    
    let _id1 = store.add_value(ValueExpr::Number(Number::Integer(1))).unwrap();
    let _id2 = store.add_value(value2).unwrap();
    let _id3 = store.add_value(ValueExpr::Number(Number::Integer(3))).unwrap();
    
    assert_eq!(store.len(), 3, "Should have 3 values stored");
}
