//! Transaction utilities for the Causality framework.
//!
//! This module provides utility functions and implementation details for working with
//! Transactions, which were moved from causality-types to maintain a clean separation
//! between type definitions and implementations.

use causality_types::resource::Resource;
use causality_types::Transaction;
use causality_types::primitive::ids::DomainId;
use causality_types::expr::ValueExpr;

//-----------------------------------------------------------------------------
// Transaction Utility Functions
//-----------------------------------------------------------------------------

/// Add a resource to a transaction
pub fn add_resource(transaction: &mut Transaction, resource: Resource) {
    transaction.resources.push(resource);
}

/// Get all resources from a transaction
pub fn get_resources(transaction: &Transaction) -> &Vec<Resource> {
    &transaction.resources
}

/// Get the domain of a transaction
pub fn get_domain(transaction: &Transaction) -> DomainId {
    transaction.domain
}

//-----------------------------------------------------------------------------
// Transaction Value Conversions
//-----------------------------------------------------------------------------

/// Project a transaction to a specific type using a value expression
pub fn project_transaction<T>(
    _transaction: &Transaction,
    value_expr: &ValueExpr,
) -> Option<T>
where
    T: TryFrom<ValueExpr>,
{
    // Check if the value_expr is a Map
    if let ValueExpr::Map(map) = value_expr {
        // Try to convert the whole map to T
        T::try_from(ValueExpr::Map(map.clone())).ok()
    } else {
        None
    }
}

/// Get the raw data for a transaction
pub fn get_transaction_raw_data(transaction: &Transaction) -> Option<Vec<u8>> {
    use causality_types::serialization::Encode;
    Some(transaction.as_ssz_bytes())
}

pub fn validate_transaction(
    _transaction: &Transaction, // Prefixed with underscore as it was unused
    // _current_resource_state: &impl ResourceReader, // Example: if needed for validation
) -> Result<(), String> {
    // Placeholder for actual validation logic
    // Example:
    // if _transaction.inputs.is_empty() && _transaction.outputs.is_empty() {
    //     return Err("Transaction has no inputs or outputs".to_string());
    // }
    Ok(())
}

/// Serializes a transaction to bytes for external storage.
pub fn serialize_transaction(transaction: &Transaction) -> Option<Vec<u8>> {
    use causality_types::serialization::Encode;
    Some(transaction.as_ssz_bytes())
}
