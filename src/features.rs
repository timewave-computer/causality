// Feature flags for conditional compilation
// This allows us to disable parts of the code with compilation issues

/// Whether to include the domain system with async operations
#[cfg(feature = "domain")]
pub const DOMAIN_ENABLED: bool = true;
#[cfg(not(feature = "domain"))]
pub const DOMAIN_ENABLED: bool = false;

/// Whether to include the full effect system
#[cfg(feature = "full-effect")]
pub const FULL_EFFECT_ENABLED: bool = true;
#[cfg(not(feature = "full-effect"))]
pub const FULL_EFFECT_ENABLED: bool = false;

/// Whether to include the code repository system
#[cfg(feature = "code-repo")]
pub const CODE_REPO_ENABLED: bool = true;
#[cfg(not(feature = "code-repo"))]
pub const CODE_REPO_ENABLED: bool = false;

/// Check if domain feature is available
/// Use this in runtime logic that needs to conditionally use domain features
pub fn has_domain() -> bool {
    DOMAIN_ENABLED
}

/// Check if full effect system is available
pub fn has_full_effect() -> bool {
    FULL_EFFECT_ENABLED
}

/// Check if code repository is available
pub fn has_code_repo() -> bool {
    CODE_REPO_ENABLED
} 