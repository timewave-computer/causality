// Deprecation utilities for the Resource Management System
//
// This module provides macros and utilities for deprecating legacy code
// in a staged manner, allowing for a smooth transition to the new APIs.

/// Deprecation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeprecationLevel {
    /// Warning level - code will continue to work but warn users
    Warning,
    
    /// Error level - code will not compile, forcing migration
    Error,
}

/// Macro for marking code as deprecated with a warning
///
/// # Example
///
/// ```
/// #[deprecated_warning(
///    since = "0.2.0",
///    note = "Use ResourceAccess trait implementations in causality-effects instead"
/// )]
/// pub fn legacy_function() {
///    // ...
/// }
/// ```
#[macro_export]
macro_rules! deprecated_warning {
    (since = $since:expr, note = $note:expr) => {
        #[deprecated(since = $since, note = $note)]
    };
}

/// Macro for marking code as deprecated with an error
///
/// Note: This is controlled by the `deprecation-error` feature flag
/// and will only cause errors when that flag is enabled.
///
/// # Example
///
/// ```
/// #[deprecated_error(
///    since = "0.2.0",
///    note = "Use ResourceAccess trait implementations in causality-effects instead"
/// )]
/// pub fn legacy_function() {
///    // ...
/// }
/// ```
#[macro_export]
macro_rules! deprecated_error {
    (since = $since:expr, note = $note:expr) => {
        #[cfg(not(feature = "allow-deprecated"))]
        #[cfg_attr(feature = "deprecation-error", deprecated(since = $since, note = $note))]
        #[cfg_attr(not(feature = "deprecation-error"), deprecated(since = $since, note = $note))]
        
        #[cfg(feature = "allow-deprecated")]
        // No deprecation when explicitly allowed
    };
}

/// Helper function to emit a deprecation warning at runtime
///
/// This is useful for code paths that cannot be statically marked as deprecated
///
/// # Example
///
/// ```
/// pub fn legacy_function() {
///    emit_deprecation_warning(
///       "legacy_function",
///       "0.2.0",
///       "Use ResourceAccess trait implementations in causality-effects instead"
///    );
///    // ...
/// }
/// ```
pub fn emit_deprecation_warning(function_name: &str, since: &str, note: &str) {
    #[cfg(not(feature = "suppress-deprecation-warnings"))]
    {
        eprintln!(
            "Warning: {} is deprecated since {} and will be removed in a future version. {}",
            function_name, since, note
        );
    }
}

/// Helper function to check if a feature is deprecated and should not be used
///
/// Returns true if the feature is deprecated and the deprecation level is set to error
///
/// # Example
///
/// ```
/// pub fn legacy_function() {
///    if is_feature_deprecated("legacy_function") {
///       // Return early or use alternative implementation
///       return;
///    }
///    // ...
/// }
/// ```
pub fn is_feature_deprecated(feature_name: &str) -> bool {
    #[cfg(feature = "deprecation-error")]
    {
        // Check if this specific feature is in the allow list
        let allow_list = std::env::var("CAUSALITY_ALLOW_DEPRECATED")
            .unwrap_or_else(|_| String::new());
        
        !allow_list.split(',').any(|allowed| allowed.trim() == feature_name)
    }
    
    #[cfg(not(feature = "deprecation-error"))]
    {
        false
    }
}

/// Constants to help with deprecation messages
pub mod messages {
    /// Generic message for deprecated resource access functionality
    pub const RESOURCE_ACCESS_DEPRECATED: &str = 
        "Use ResourceAccess trait implementations in causality-effects::resource::access instead";
    
    /// Generic message for deprecated lifecycle management functionality
    pub const LIFECYCLE_DEPRECATED: &str = 
        "Use ResourceLifecycle trait implementations in causality-effects::resource::lifecycle instead";
    
    /// Generic message for deprecated locking functionality
    pub const LOCKING_DEPRECATED: &str = 
        "Use ResourceLocking trait implementations in causality-effects::resource::locking instead";
    
    /// Generic message for deprecated dependency functionality
    pub const DEPENDENCY_DEPRECATED: &str = 
        "Use ResourceDependency trait implementations in causality-effects::resource::dependency instead";
    
    /// Message for entire deprecated modules
    pub const MODULE_DEPRECATED: &str = 
        "This module is deprecated. Use the unified resource management system in causality-effects::resource instead";
    
    /// Since version for all deprecations
    pub const SINCE_VERSION: &str = "0.2.0";
} 