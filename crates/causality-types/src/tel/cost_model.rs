//! Cost modeling types for runtime optimization
//!
//! This module defines cost models and estimation utilities for different TypedDomains,
//! enabling optimization strategies to make informed decisions about resource allocation
//! and execution planning.

use crate::{
    core::{
        str::Str,
        time::Timestamp,
    },
    tel::optimization::TypedDomain,
    serialization::{Encode, Decode, DecodeError, SimpleSerialize},
};
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// Cost Model Types
//-----------------------------------------------------------------------------

/// Cost model for effects with TypedDomain-specific considerations
#[derive(Debug, Clone, PartialEq)]
pub struct EffectCostModel {
    /// Base computational cost
    pub base_compute_cost: u64,
    /// Base storage cost
    pub base_storage_cost: u64,
    /// Base network cost
    pub base_network_cost: u64,
    /// ZK-specific costs for VerifiableDomain
    pub zk_proof_cost: u64,
    /// ZK verification cost
    pub zk_verification_cost: u64,
    /// API/RPC costs for ServiceDomain
    pub api_call_cost: u64,
    /// External service interaction cost
    pub external_service_cost: u64,
    /// Cost multipliers per TypedDomain
    pub domain_multipliers: HashMap<TypedDomain, f64>,
    /// Cross-domain transfer costs
    pub cross_domain_transfer_cost: u64,
    /// Complexity multiplier (1.0 = baseline)
    pub complexity_multiplier: f64,
}

impl Eq for EffectCostModel {}

impl EffectCostModel {
    /// Create a new cost model with default values
    pub fn new() -> Self {
        Self {
            base_compute_cost: 100,
            base_storage_cost: 50,
            base_network_cost: 25,
            zk_proof_cost: 1000,
            zk_verification_cost: 100,
            api_call_cost: 200,
            external_service_cost: 500,
            domain_multipliers: HashMap::new(),
            cross_domain_transfer_cost: 300,
            complexity_multiplier: 1.0,
        }
    }

    /// Calculate total cost for a specific TypedDomain
    pub fn calculate_cost(&self, source_domain: &TypedDomain, target_domain: &TypedDomain) -> u64 {
        let mut total_cost = self.base_compute_cost + self.base_storage_cost + self.base_network_cost;

        // Add domain-specific costs
        match source_domain {
            TypedDomain::VerifiableDomain(_) => {
                total_cost += self.zk_proof_cost;
            }
            TypedDomain::ServiceDomain(_) => {
                total_cost += self.api_call_cost;
            }
        }

        match target_domain {
            TypedDomain::VerifiableDomain(_) => {
                total_cost += self.zk_verification_cost;
            }
            TypedDomain::ServiceDomain(_) => {
                total_cost += self.external_service_cost;
            }
        }

        // Add cross-domain cost if domains differ
        if source_domain != target_domain {
            total_cost += self.cross_domain_transfer_cost;
        }

        // Apply domain-specific multipliers
        let source_multiplier = self.domain_multipliers.get(source_domain).unwrap_or(&1.0);
        let target_multiplier = self.domain_multipliers.get(target_domain).unwrap_or(&1.0);
        let avg_multiplier = (source_multiplier + target_multiplier) / 2.0;

        // Apply complexity multiplier
        let final_cost = (total_cost as f64 * avg_multiplier * self.complexity_multiplier) as u64;
        final_cost
    }

    /// Set domain-specific multiplier
    pub fn set_domain_multiplier(&mut self, domain: TypedDomain, multiplier: f64) {
        self.domain_multipliers.insert(domain, multiplier);
    }

    /// Get estimated ZK circuit size for VerifiableDomain operations
    pub fn estimate_zk_circuit_size(&self, complexity: u32) -> u32 {
        // Base circuit size plus complexity factor
        1000 + (complexity * 100)
    }

    /// Get estimated API latency for ServiceDomain operations
    pub fn estimate_api_latency_ms(&self, api_complexity: u32) -> u64 {
        // Base latency plus complexity factor
        100 + (api_complexity as u64 * 50)
    }
}

impl Default for EffectCostModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Encode for EffectCostModel {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize basic fields
        bytes.extend_from_slice(&self.base_compute_cost.to_le_bytes());
        bytes.extend_from_slice(&self.base_storage_cost.to_le_bytes());
        bytes.extend_from_slice(&self.base_network_cost.to_le_bytes());
        bytes.extend_from_slice(&self.zk_proof_cost.to_le_bytes());
        bytes.extend_from_slice(&self.zk_verification_cost.to_le_bytes());
        bytes.extend_from_slice(&self.api_call_cost.to_le_bytes());
        bytes.extend_from_slice(&self.external_service_cost.to_le_bytes());
        bytes.extend_from_slice(&self.cross_domain_transfer_cost.to_le_bytes());
        bytes.extend_from_slice(&self.complexity_multiplier.to_le_bytes());
        
        // Convert HashMap to Vec of pairs for serialization
        let domain_pairs: Vec<(TypedDomain, f64)> = self.domain_multipliers.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        bytes.extend_from_slice(&domain_pairs.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for EffectCostModel {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 72 { // 9 * 8 bytes for basic fields
            return Err(DecodeError::new("Insufficient data for EffectCostModel"));
        }
        
        let mut offset = 0;
        
        let base_compute_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let base_storage_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let base_network_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let zk_proof_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let zk_verification_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let api_call_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let external_service_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let cross_domain_transfer_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let complexity_multiplier = f64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        // Deserialize domain multipliers from Vec of pairs
        let domain_pairs: Vec<(TypedDomain, f64)> = Vec::from_ssz_bytes(&bytes[offset..])?;
        let domain_multipliers: HashMap<TypedDomain, f64> = domain_pairs.into_iter().collect();
        
        Ok(EffectCostModel {
            base_compute_cost,
            base_storage_cost,
            base_network_cost,
            zk_proof_cost,
            zk_verification_cost,
            api_call_cost,
            external_service_cost,
            domain_multipliers,
            cross_domain_transfer_cost,
            complexity_multiplier,
        })
    }
}

impl SimpleSerialize for EffectCostModel {}

//-----------------------------------------------------------------------------
// Resource Usage Estimation
//-----------------------------------------------------------------------------

/// Resource usage estimation with TypedDomain-specific considerations
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceUsageEstimate {
    /// Compute resource usage range (min, max)
    pub compute_range: (u64, u64),
    /// Storage resource usage range (min, max)
    pub storage_range: (u64, u64),
    /// Network resource usage range (min, max)
    pub network_range: (u64, u64),
    /// Memory usage range (min, max)
    pub memory_range: (u64, u64),
    /// Time estimate range in milliseconds (min, max)
    pub time_range_ms: (u64, u64),
    /// TypedDomain-specific estimates
    pub domain_specific_estimates: HashMap<TypedDomain, DomainSpecificEstimate>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Estimation timestamp
    pub estimated_at: Timestamp,
}

impl Eq for ResourceUsageEstimate {}

/// Domain-specific resource usage estimates
#[derive(Debug, Clone, PartialEq)]
pub struct DomainSpecificEstimate {
    /// ZK circuit constraints (for VerifiableDomain)
    pub zk_constraints: Option<u32>,
    /// ZK witness size (for VerifiableDomain)
    pub zk_witness_size: Option<u32>,
    /// API call count (for ServiceDomain)
    pub api_call_count: Option<u32>,
    /// External dependency count (for ServiceDomain)
    pub external_dependencies: Option<u32>,
    /// Custom metrics
    pub custom_metrics: HashMap<Str, u64>,
}

impl Eq for DomainSpecificEstimate {}

impl ResourceUsageEstimate {
    /// Create a new resource usage estimate
    pub fn new() -> Self {
        Self {
            compute_range: (0, 0),
            storage_range: (0, 0),
            network_range: (0, 0),
            memory_range: (0, 0),
            time_range_ms: (0, 0),
            domain_specific_estimates: HashMap::new(),
            confidence: 0.5,
            estimated_at: Timestamp::now(),
        }
    }

    /// Get estimate for a specific TypedDomain
    pub fn get_domain_estimate(&self, domain: &TypedDomain) -> Option<&DomainSpecificEstimate> {
        self.domain_specific_estimates.get(domain)
    }

    /// Set domain-specific estimate
    pub fn set_domain_estimate(&mut self, domain: TypedDomain, estimate: DomainSpecificEstimate) {
        self.domain_specific_estimates.insert(domain, estimate);
    }

    /// Calculate weighted average of ranges
    pub fn get_weighted_compute_estimate(&self) -> u64 {
        let (min, max) = self.compute_range;
        ((min as f64 * (1.0 - self.confidence)) + (max as f64 * self.confidence)) as u64
    }

    /// Get total estimated cost using the provided cost model
    pub fn estimate_total_cost(&self, cost_model: &EffectCostModel, source_domain: &TypedDomain, target_domain: &TypedDomain) -> u64 {
        let base_cost = cost_model.calculate_cost(source_domain, target_domain);
        let compute_factor = self.get_weighted_compute_estimate() as f64 / 1000.0; // Normalize
        (base_cost as f64 * (1.0 + compute_factor)) as u64
    }
}

impl Default for ResourceUsageEstimate {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainSpecificEstimate {
    /// Create a new domain-specific estimate
    pub fn new() -> Self {
        Self {
            zk_constraints: None,
            zk_witness_size: None,
            api_call_count: None,
            external_dependencies: None,
            custom_metrics: HashMap::new(),
        }
    }

    /// Create estimate for VerifiableDomain
    pub fn for_verifiable_domain(constraints: u32, witness_size: u32) -> Self {
        Self {
            zk_constraints: Some(constraints),
            zk_witness_size: Some(witness_size),
            api_call_count: None,
            external_dependencies: None,
            custom_metrics: HashMap::new(),
        }
    }

    /// Create estimate for ServiceDomain
    pub fn for_service_domain(api_calls: u32, dependencies: u32) -> Self {
        Self {
            zk_constraints: None,
            zk_witness_size: None,
            api_call_count: Some(api_calls),
            external_dependencies: Some(dependencies),
            custom_metrics: HashMap::new(),
        }
    }
}

impl Default for DomainSpecificEstimate {
    fn default() -> Self {
        Self::new()
    }
}

impl Encode for DomainSpecificEstimate {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize Option<u32> fields
        match self.zk_constraints {
            Some(val) => {
                bytes.push(1); // Some indicator
                bytes.extend_from_slice(&val.to_le_bytes());
            }
            None => {
                bytes.push(0); // None indicator
                bytes.extend_from_slice(&[0u8; 4]); // Padding
            }
        }
        
        match self.zk_witness_size {
            Some(val) => {
                bytes.push(1);
                bytes.extend_from_slice(&val.to_le_bytes());
            }
            None => {
                bytes.push(0);
                bytes.extend_from_slice(&[0u8; 4]);
            }
        }
        
        match self.api_call_count {
            Some(val) => {
                bytes.push(1);
                bytes.extend_from_slice(&val.to_le_bytes());
            }
            None => {
                bytes.push(0);
                bytes.extend_from_slice(&[0u8; 4]);
            }
        }
        
        match self.external_dependencies {
            Some(val) => {
                bytes.push(1);
                bytes.extend_from_slice(&val.to_le_bytes());
            }
            None => {
                bytes.push(0);
                bytes.extend_from_slice(&[0u8; 4]);
            }
        }
        
        // Convert HashMap to Vec of pairs for serialization
        let custom_pairs: Vec<(Str, u64)> = self.custom_metrics.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        let custom_pairs_bytes = custom_pairs.as_ssz_bytes();
        bytes.extend_from_slice(&custom_pairs_bytes);
        
        bytes
    }
}

impl Decode for DomainSpecificEstimate {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 20 { // 4 * (1 + 4) bytes for Option fields
            return Err(DecodeError::new("Insufficient data for DomainSpecificEstimate"));
        }
        
        let mut offset = 0;
        
        let zk_constraints = if bytes[offset] == 1 {
            offset += 1;
            let val = u32::from_le_bytes([
                bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            ]);
            offset += 4;
            Some(val)
        } else {
            offset += 5; // Skip indicator + padding
            None
        };
        
        let zk_witness_size = if bytes[offset] == 1 {
            offset += 1;
            let val = u32::from_le_bytes([
                bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            ]);
            offset += 4;
            Some(val)
        } else {
            offset += 5;
            None
        };
        
        let api_call_count = if bytes[offset] == 1 {
            offset += 1;
            let val = u32::from_le_bytes([
                bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            ]);
            offset += 4;
            Some(val)
        } else {
            offset += 5;
            None
        };
        
        let external_dependencies = if bytes[offset] == 1 {
            offset += 1;
            let val = u32::from_le_bytes([
                bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            ]);
            offset += 4;
            Some(val)
        } else {
            offset += 5;
            None
        };
        
        // Deserialize custom metrics from Vec of pairs
        let custom_pairs: Vec<(Str, u64)> = Vec::from_ssz_bytes(&bytes[offset..])?;
        let custom_metrics: HashMap<Str, u64> = custom_pairs.into_iter().collect();
        
        Ok(DomainSpecificEstimate {
            zk_constraints,
            zk_witness_size,
            api_call_count,
            external_dependencies,
            custom_metrics,
        })
    }
}

impl SimpleSerialize for DomainSpecificEstimate {}

impl Encode for ResourceUsageEstimate {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize tuple fields as pairs of u64
        bytes.extend_from_slice(&self.compute_range.0.to_le_bytes());
        bytes.extend_from_slice(&self.compute_range.1.to_le_bytes());
        bytes.extend_from_slice(&self.storage_range.0.to_le_bytes());
        bytes.extend_from_slice(&self.storage_range.1.to_le_bytes());
        bytes.extend_from_slice(&self.network_range.0.to_le_bytes());
        bytes.extend_from_slice(&self.network_range.1.to_le_bytes());
        bytes.extend_from_slice(&self.memory_range.0.to_le_bytes());
        bytes.extend_from_slice(&self.memory_range.1.to_le_bytes());
        bytes.extend_from_slice(&self.time_range_ms.0.to_le_bytes());
        bytes.extend_from_slice(&self.time_range_ms.1.to_le_bytes());
        
        // Serialize confidence and timestamp
        bytes.extend_from_slice(&self.confidence.to_le_bytes());
        bytes.extend_from_slice(&self.estimated_at.as_ssz_bytes());
        
        // Convert HashMap to Vec of pairs for serialization
        let estimate_pairs: Vec<(TypedDomain, DomainSpecificEstimate)> = self.domain_specific_estimates.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        bytes.extend_from_slice(&estimate_pairs.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for ResourceUsageEstimate {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 88 { // 10 * 8 bytes for ranges + 8 for confidence + timestamp
            return Err(DecodeError::new("Insufficient data for ResourceUsageEstimate"));
        }
        
        let mut offset = 0;
        
        let compute_range = (
            u64::from_le_bytes([
                bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
                bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
            ]),
            u64::from_le_bytes([
                bytes[offset + 8], bytes[offset + 9], bytes[offset + 10], bytes[offset + 11],
                bytes[offset + 12], bytes[offset + 13], bytes[offset + 14], bytes[offset + 15],
            ])
        );
        offset += 16;
        
        let storage_range = (
            u64::from_le_bytes([
                bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
                bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
            ]),
            u64::from_le_bytes([
                bytes[offset + 8], bytes[offset + 9], bytes[offset + 10], bytes[offset + 11],
                bytes[offset + 12], bytes[offset + 13], bytes[offset + 14], bytes[offset + 15],
            ])
        );
        offset += 16;
        
        let network_range = (
            u64::from_le_bytes([
                bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
                bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
            ]),
            u64::from_le_bytes([
                bytes[offset + 8], bytes[offset + 9], bytes[offset + 10], bytes[offset + 11],
                bytes[offset + 12], bytes[offset + 13], bytes[offset + 14], bytes[offset + 15],
            ])
        );
        offset += 16;
        
        let memory_range = (
            u64::from_le_bytes([
                bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
                bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
            ]),
            u64::from_le_bytes([
                bytes[offset + 8], bytes[offset + 9], bytes[offset + 10], bytes[offset + 11],
                bytes[offset + 12], bytes[offset + 13], bytes[offset + 14], bytes[offset + 15],
            ])
        );
        offset += 16;
        
        let time_range_ms = (
            u64::from_le_bytes([
                bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
                bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
            ]),
            u64::from_le_bytes([
                bytes[offset + 8], bytes[offset + 9], bytes[offset + 10], bytes[offset + 11],
                bytes[offset + 12], bytes[offset + 13], bytes[offset + 14], bytes[offset + 15],
            ])
        );
        offset += 16;
        
        let confidence = f64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let estimated_at = Timestamp::from_ssz_bytes(&bytes[offset..])?;
        offset += estimated_at.as_ssz_bytes().len();
        
        // Deserialize domain specific estimates from Vec of pairs
        let estimate_pairs: Vec<(TypedDomain, DomainSpecificEstimate)> = Vec::from_ssz_bytes(&bytes[offset..])?;
        let domain_specific_estimates: HashMap<TypedDomain, DomainSpecificEstimate> = estimate_pairs.into_iter().collect();
        
        Ok(ResourceUsageEstimate {
            compute_range,
            storage_range,
            network_range,
            memory_range,
            time_range_ms,
            domain_specific_estimates,
            confidence,
            estimated_at,
        })
    }
}

impl SimpleSerialize for ResourceUsageEstimate {}

//-----------------------------------------------------------------------------
// Cost Calculation Utilities
//-----------------------------------------------------------------------------

/// Calculate cross-domain transfer cost
pub fn calculate_cross_domain_cost(
    source_typed_domain: &TypedDomain,
    target_typed_domain: &TypedDomain,
    base_cost: u64,
) -> u64 {
    if source_typed_domain == target_typed_domain {
        return base_cost;
    }

    let transfer_multiplier = match (source_typed_domain, target_typed_domain) {
        (TypedDomain::VerifiableDomain(_), TypedDomain::ServiceDomain(_)) => 2.0, // ZK to Service
        (TypedDomain::ServiceDomain(_), TypedDomain::VerifiableDomain(_)) => 3.0, // Service to ZK (more expensive)
        (TypedDomain::VerifiableDomain(_), TypedDomain::VerifiableDomain(_)) => 1.5, // ZK to ZK
        (TypedDomain::ServiceDomain(_), TypedDomain::ServiceDomain(_)) => 1.2, // Service to Service
    };

    (base_cost as f64 * transfer_multiplier) as u64
}

/// Estimate execution time based on TypedDomain characteristics
pub fn estimate_execution_time(
    typed_domain: &TypedDomain,
    complexity: u32,
    base_time_ms: u64,
) -> u64 {
    let domain_multiplier = match typed_domain {
        TypedDomain::VerifiableDomain(_) => {
            // ZK operations are slower due to proof generation
            2.0 + (complexity as f64 * 0.1)
        }
        TypedDomain::ServiceDomain(_) => {
            // Service operations depend on network latency
            1.5 + (complexity as f64 * 0.05)
        }
    };

    (base_time_ms as f64 * domain_multiplier) as u64
}

/// Calculate resource efficiency score
pub fn calculate_efficiency_score(
    actual_usage: &ResourceUsageEstimate,
    estimated_usage: &ResourceUsageEstimate,
) -> f64 {
    let compute_efficiency = if estimated_usage.compute_range.1 > 0 {
        1.0 - (actual_usage.get_weighted_compute_estimate() as f64 / estimated_usage.compute_range.1 as f64).min(1.0)
    } else {
        1.0
    };

    let time_efficiency = if estimated_usage.time_range_ms.1 > 0 {
        let actual_time = (actual_usage.time_range_ms.0 + actual_usage.time_range_ms.1) / 2;
        1.0 - (actual_time as f64 / estimated_usage.time_range_ms.1 as f64).min(1.0)
    } else {
        1.0
    };

    (compute_efficiency + time_efficiency) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::ids::DomainId;

    #[test]
    fn test_effect_cost_model() {
        let mut cost_model = EffectCostModel::new();
        let domain1 = TypedDomain::VerifiableDomain(DomainId::new([1u8; 32]));
        let domain2 = TypedDomain::ServiceDomain(DomainId::new([2u8; 32]));

        // Test same domain cost
        let same_domain_cost = cost_model.calculate_cost(&domain1, &domain1);
        assert!(same_domain_cost > 0);

        // Test cross-domain cost (should be higher)
        let cross_domain_cost = cost_model.calculate_cost(&domain1, &domain2);
        assert!(cross_domain_cost > same_domain_cost);

        // Test domain multiplier
        cost_model.set_domain_multiplier(domain1.clone(), 2.0);
        let multiplied_cost = cost_model.calculate_cost(&domain1, &domain1);
        assert!(multiplied_cost > same_domain_cost);
    }

    #[test]
    fn test_resource_usage_estimate() {
        let mut estimate = ResourceUsageEstimate::new();
        estimate.compute_range = (100, 200);
        estimate.confidence = 0.8;

        let weighted_estimate = estimate.get_weighted_compute_estimate();
        assert_eq!(weighted_estimate, 180); // 100 * 0.2 + 200 * 0.8

        // Test domain-specific estimates
        let domain = TypedDomain::VerifiableDomain(DomainId::new([1u8; 32]));
        let domain_estimate = DomainSpecificEstimate::for_verifiable_domain(1000, 500);
        estimate.set_domain_estimate(domain.clone(), domain_estimate);

        assert!(estimate.get_domain_estimate(&domain).is_some());
    }

    #[test]
    fn test_cross_domain_cost_calculation() {
        let domain1 = TypedDomain::VerifiableDomain(DomainId::new([1u8; 32]));
        let domain2 = TypedDomain::ServiceDomain(DomainId::new([2u8; 32]));
        let base_cost = 1000;

        let same_domain_cost = calculate_cross_domain_cost(&domain1, &domain1, base_cost);
        assert_eq!(same_domain_cost, base_cost);

        let cross_domain_cost = calculate_cross_domain_cost(&domain1, &domain2, base_cost);
        assert!(cross_domain_cost > base_cost);
    }

    #[test]
    fn test_serialization_round_trip() {
        // Test EffectCostModel with empty HashMap (should work)
        let cost_model = EffectCostModel::new();
        let bytes = cost_model.as_ssz_bytes();
        let decoded = EffectCostModel::from_ssz_bytes(&bytes).unwrap();
        assert_eq!(cost_model, decoded);

        // Test ResourceUsageEstimate with empty HashMap (should work)
        let estimate = ResourceUsageEstimate::new();
        let bytes = estimate.as_ssz_bytes();
        let decoded = ResourceUsageEstimate::from_ssz_bytes(&bytes).unwrap();
        assert_eq!(estimate, decoded);
    }
} 