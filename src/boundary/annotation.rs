use std::fmt;

/// Types of system boundaries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundaryType {
    /// Inside the Causality system
    InsideSystem,
    /// Outside the Causality system
    OutsideSystem,
    /// On-Chain execution
    OnChain,
    /// Off-Chain execution
    OffChain,
    /// Ethereum Virtual Machine
    EVM,
    /// CosmWasm Virtual Machine
    CosmWasm,
    /// Local execution
    Local,
    /// Custom execution context
    Custom(String),
}

impl fmt::Display for BoundaryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoundaryType::InsideSystem => write!(f, "inside_system"),
            BoundaryType::OutsideSystem => write!(f, "outside_system"),
            BoundaryType::OnChain => write!(f, "on_chain"),
            BoundaryType::OffChain => write!(f, "off_chain"),
            BoundaryType::EVM => write!(f, "evm"),
            BoundaryType::CosmWasm => write!(f, "cosmwasm"),
            BoundaryType::Local => write!(f, "local"),
            BoundaryType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

impl BoundaryType {
    /// Parse a boundary type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "inside_system" => Some(BoundaryType::InsideSystem),
            "outside_system" => Some(BoundaryType::OutsideSystem),
            "on_chain" => Some(BoundaryType::OnChain),
            "off_chain" => Some(BoundaryType::OffChain),
            "evm" => Some(BoundaryType::EVM),
            "CosmWasm" => Some(BoundaryType::CosmWasm),
            "local" => Some(BoundaryType::Local),
            s if s.starts_with("custom:") => {
                let custom_name = s.trim_start_matches("custom:").to_string();
                Some(BoundaryType::Custom(custom_name))
            }
            _ => None,
        }
    }
}

/// Types of boundary crossings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossingType {
    /// From inside to outside
    InsideToOutside,
    /// From outside to inside
    OutsideToInside,
    /// From off-chain to on-chain
    OffChainToOnChain,
    /// From on-chain to off-chain
    OnChainToOffChain,
    /// From one chain to another
    CrossChain,
    /// Custom crossing
    Custom(String),
}

impl fmt::Display for CrossingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrossingType::InsideToOutside => write!(f, "inside_to_outside"),
            CrossingType::OutsideToInside => write!(f, "outside_to_inside"),
            CrossingType::OffChainToOnChain => write!(f, "offchain_to_onchain"),
            CrossingType::OnChainToOffChain => write!(f, "onchain_to_offchain"),
            CrossingType::CrossChain => write!(f, "cross_chain"),
            CrossingType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

impl CrossingType {
    /// Parse a crossing type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "inside_to_outside" => Some(CrossingType::InsideToOutside),
            "outside_to_inside" => Some(CrossingType::OutsideToInside),
            "offchain_to_onchain" => Some(CrossingType::OffChainToOnChain),
            "onchain_to_offchain" => Some(CrossingType::OnChainToOffChain),
            "cross_chain" => Some(CrossingType::CrossChain),
            s if s.starts_with("custom:") => {
                let custom_name = s.trim_start_matches("custom:").to_string();
                Some(CrossingType::Custom(custom_name))
            }
            _ => None,
        }
    }
}

/// Function for marking code as executing within a specific boundary
pub fn boundary(boundary_type: &str, func: impl FnOnce()) {
    // Log boundary execution in debug mode
    #[cfg(debug_assertions)]
    {
        eprintln!("Executing function in boundary: {}", boundary_type);
    }
    
    // Execute the function
    func();
}

/// Function for marking code that crosses boundaries
pub fn boundary_crossing(crossing_type: &str, func: impl FnOnce()) {
    // Log boundary crossing in debug mode
    #[cfg(debug_assertions)]
    {
        eprintln!("Crossing boundary: {}", crossing_type);
    }
    
    // Track crossing metrics
    super::metrics::record_boundary_crossing(crossing_type);
    
    // Execute the function
    func();
}

/// Runtime helper for tracking boundary execution
pub struct BoundaryTracker;

impl BoundaryTracker {
    /// Record that code is executing in a specific boundary
    pub fn track_execution(boundary: BoundaryType) {
        // In a real implementation, this would update metrics or logs
        eprintln!("Tracking execution in boundary: {}", boundary);
    }
    
    /// Record a boundary crossing
    pub fn track_crossing(crossing: CrossingType) {
        // In a real implementation, this would update metrics or logs
        eprintln!("Tracking boundary crossing: {}", crossing);
    }
}

/// Marker trait for types that are safe to cross boundaries
pub trait BoundarySafe: Send + Sync + 'static {
    /// Get the boundary where this type is intended to be used
    fn target_boundary(&self) -> BoundaryType;
    
    /// Validate that this type can be used in the given boundary
    fn validate_for_boundary(&self, boundary: BoundaryType) -> bool {
        self.target_boundary() == boundary
    }
    
    /// Prepare this type for crossing a boundary
    fn prepare_for_crossing(&self) -> Vec<u8>;
    
    /// Reconstruct this type after crossing a boundary
    fn from_crossing(data: &[u8]) -> Result<Self, String> where Self: Sized;
} 