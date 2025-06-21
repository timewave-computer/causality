//! Location type system for unified computation and communication
//!
//! This module defines the location algebra that enables location-aware types
//! and supports both local computation and distributed communication through
//! a unified abstraction.
//!
//! **Design Principles**:
//! - Location composition for complex distributed systems
//! - Location routing for message passing
//! - Location unification for type inference
//! - Location transparency where appropriate

use crate::system::deterministic::DeterministicSystem;
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};
use ssz::{Encode, Decode, DecodeError};
use crate::system::DecodeWithRemainder;

/// Location in the distributed system
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Location {
    /// Local execution context
    Local,
    
    /// Remote location identified by address
    Remote(String),
    
    /// Domain-based location (for compatibility)
    Domain(String),
    
    /// Composite location (parallel composition)
    Composite(Vec<Location>),
    
    /// Location variable for inference
    Variable(String),
    
    /// Any location (top type)
    Any,
    
    /// No location (bottom type)
    None,
}

/// Location algebra operations
impl Location {
    /// Create a new remote location
    pub fn remote(address: impl Into<String>) -> Self {
        Location::Remote(address.into())
    }
    
    /// Create a new domain location
    pub fn domain(name: impl Into<String>) -> Self {
        Location::Domain(name.into())
    }
    
    /// Create a location variable
    pub fn variable(name: impl Into<String>) -> Self {
        Location::Variable(name.into())
    }
    
    /// Compose two locations in parallel
    pub fn compose(self, other: Location) -> Location {
        match (self, other) {
            (Location::None, loc) | (loc, Location::None) => loc,
            (Location::Any, _) | (_, Location::Any) => Location::Any,
            (Location::Composite(mut locs1), Location::Composite(locs2)) => {
                locs1.extend(locs2);
                Location::Composite(locs1)
            }
            (Location::Composite(mut locs), loc) | (loc, Location::Composite(mut locs)) => {
                locs.push(loc);
                Location::Composite(locs)
            }
            (loc1, loc2) if loc1 == loc2 => loc1,
            (loc1, loc2) => Location::Composite(vec![loc1, loc2]),
        }
    }
    
    /// Check if this location is local
    pub fn is_local(&self) -> bool {
        matches!(self, Location::Local)
    }
    
    /// Check if this location is remote
    pub fn is_remote(&self) -> bool {
        matches!(self, Location::Remote(_) | Location::Domain(_))
    }
    
    /// Check if this location is composite
    pub fn is_composite(&self) -> bool {
        matches!(self, Location::Composite(_))
    }
    
    /// Check if this location is a variable
    pub fn is_variable(&self) -> bool {
        matches!(self, Location::Variable(_))
    }
    
    /// Check if this location is concrete (no variables)
    pub fn is_concrete(&self) -> bool {
        match self {
            Location::Local | Location::Remote(_) | Location::Domain(_) => true,
            Location::Composite(locs) => locs.iter().all(|loc| loc.is_concrete()),
            Location::Variable(_) | Location::Any | Location::None => false,
        }
    }
    
    /// Get all concrete locations in this location
    pub fn concrete_locations(&self) -> BTreeSet<Location> {
        let mut result = BTreeSet::new();
        self.collect_concrete_locations(&mut result);
        result
    }
    
    fn collect_concrete_locations(&self, result: &mut BTreeSet<Location>) {
        match self {
            Location::Local | Location::Remote(_) | Location::Domain(_) => {
                result.insert(self.clone());
            }
            Location::Composite(locs) => {
                for loc in locs {
                    loc.collect_concrete_locations(result);
                }
            }
            _ => {} // Variables and special locations don't contribute concrete locations
        }
    }
    
    /// Get the distance between two locations (for routing)
    pub fn distance_to(&self, other: &Location) -> Option<u32> {
        match (self, other) {
            (loc1, loc2) if loc1 == loc2 => Some(0),
            (Location::Local, Location::Remote(_)) | (Location::Remote(_), Location::Local) => Some(1),
            (Location::Local, Location::Domain(_)) | (Location::Domain(_), Location::Local) => Some(1),
            (Location::Remote(_), Location::Remote(_)) => Some(2), // Via intermediate
            (Location::Domain(_), Location::Domain(_)) => Some(2), // Via intermediate
            (Location::Remote(_), Location::Domain(_)) | (Location::Domain(_), Location::Remote(_)) => Some(3),
            _ => None, // Cannot compute distance for variables or special locations
        }
    }
    
    /// Check if this location can reach another location
    pub fn can_reach(&self, other: &Location) -> bool {
        self.distance_to(other).is_some()
    }
    
    /// Find the shortest path between two locations
    pub fn route_to(&self, other: &Location) -> Option<Vec<Location>> {
        if self == other {
            return Some(vec![self.clone()]);
        }
        
        match (self, other) {
            (Location::Local, Location::Remote(addr)) => {
                Some(vec![Location::Local, Location::Remote(addr.clone())])
            }
            (Location::Remote(addr), Location::Local) => {
                Some(vec![Location::Remote(addr.clone()), Location::Local])
            }
            (Location::Local, Location::Domain(domain)) => {
                Some(vec![Location::Local, Location::Domain(domain.clone())])
            }
            (Location::Domain(domain), Location::Local) => {
                Some(vec![Location::Domain(domain.clone()), Location::Local])
            }
            (Location::Remote(addr1), Location::Remote(addr2)) => {
                // Route via local (simplified routing)
                Some(vec![
                    Location::Remote(addr1.clone()),
                    Location::Local,
                    Location::Remote(addr2.clone())
                ])
            }
            (Location::Domain(d1), Location::Domain(d2)) => {
                // Route via local (simplified routing)
                Some(vec![
                    Location::Domain(d1.clone()),
                    Location::Local,
                    Location::Domain(d2.clone())
                ])
            }
            _ => None, // Cannot route between these locations
        }
    }
    
    /// Substitute location variables with concrete locations
    pub fn substitute(&self, substitutions: &BTreeMap<String, Location>) -> Location {
        match self {
            Location::Variable(name) => {
                substitutions.get(name).cloned().unwrap_or_else(|| self.clone())
            }
            Location::Composite(locs) => {
                let substituted: Vec<_> = locs.iter()
                    .map(|loc| loc.substitute(substitutions))
                    .collect();
                Location::Composite(substituted)
            }
            _ => self.clone(),
        }
    }
    
    /// Get all location variables in this location
    pub fn variables(&self) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        self.collect_variables(&mut result);
        result
    }
    
    fn collect_variables(&self, result: &mut BTreeSet<String>) {
        match self {
            Location::Variable(name) => {
                result.insert(name.clone());
            }
            Location::Composite(locs) => {
                for loc in locs {
                    loc.collect_variables(result);
                }
            }
            _ => {}
        }
    }
    
    /// Check if this location is more general than another (for subtyping)
    pub fn is_more_general_than(&self, other: &Location) -> bool {
        match (self, other) {
            (Location::Any, _) => true,
            (_, Location::None) => true,
            (Location::Variable(_), _) => true, // Variables are more general
            (loc1, loc2) if loc1 == loc2 => true,
            (Location::Composite(locs1), Location::Composite(locs2)) => {
                // Check if all locations in locs2 are covered by locs1
                locs2.iter().all(|loc2| {
                    locs1.iter().any(|loc1| loc1.is_more_general_than(loc2))
                })
            }
            _ => false,
        }
    }
    
    /// Unify two locations (for type inference)
    pub fn unify(&self, other: &Location) -> Option<LocationUnification> {
        LocationUnifier::new().unify(self, other)
    }
    
    /// Generate a fresh location variable
    pub fn fresh_variable(deterministic: &mut DeterministicSystem) -> Location {
        let id = deterministic.deterministic_u64();
        Location::Variable(format!("L{}", id))
    }
}

/// Result of location unification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationUnification {
    /// The unified location
    pub unified: Location,
    
    /// Substitutions for location variables
    pub substitutions: BTreeMap<String, Location>,
    
    /// Constraints that must be satisfied
    pub constraints: Vec<LocationConstraint>,
}

/// Constraints on location relationships
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocationConstraint {
    /// Two locations must be equal
    Equal(Location, Location),
    
    /// One location must be reachable from another
    Reachable(Location, Location),
    
    /// Location must be concrete (no variables)
    Concrete(Location),
    
    /// Location must be local
    Local(Location),
    
    /// Location must be remote
    Remote(Location),
    
    /// Locations must be co-located
    CoLocated(Vec<Location>),
}

/// Location unification algorithm
pub struct LocationUnifier {
    substitutions: BTreeMap<String, Location>,
    constraints: Vec<LocationConstraint>,
}

impl LocationUnifier {
    pub fn new() -> Self {
        LocationUnifier {
            substitutions: BTreeMap::new(),
            constraints: Vec::new(),
        }
    }
    
    pub fn unify(&mut self, loc1: &Location, loc2: &Location) -> Option<LocationUnification> {
        if self.unify_locations(loc1, loc2) {
            Some(LocationUnification {
                unified: loc1.substitute(&self.substitutions),
                substitutions: self.substitutions.clone(),
                constraints: self.constraints.clone(),
            })
        } else {
            None
        }
    }
    
    fn unify_locations(&mut self, loc1: &Location, loc2: &Location) -> bool {
        match (loc1, loc2) {
            // Identical locations unify trivially
            (loc1, loc2) if loc1 == loc2 => true,
            
            // Variables unify with anything
            (Location::Variable(name), loc) | (loc, Location::Variable(name)) => {
                self.bind_variable(name.clone(), loc.clone())
            }
            
            // Any unifies with anything
            (Location::Any, _) | (_, Location::Any) => true,
            
            // None unifies with nothing (except itself)
            (Location::None, _) | (_, Location::None) => false,
            
            // Composite locations
            (Location::Composite(locs1), Location::Composite(locs2)) => {
                if locs1.len() != locs2.len() {
                    return false;
                }
                
                // Try to unify corresponding locations
                locs1.iter().zip(locs2.iter()).all(|(l1, l2)| {
                    self.unify_locations(l1, l2)
                })
            }
            
            // Different concrete locations don't unify
            _ => false,
        }
    }
    
    fn bind_variable(&mut self, var: String, location: Location) -> bool {
        // Check if variable is already bound
        if let Some(existing) = self.substitutions.get(&var).cloned() {
            return self.unify_locations(&existing, &location);
        }
        
        // Check for occurs check (variable occurs in location)
        if location.variables().contains(&var) {
            return false;
        }
        
        self.substitutions.insert(var, location);
        true
    }
}

/// Location context for tracking location assignments
#[derive(Debug, Clone)]
pub struct LocationContext {
    /// Variable to location bindings
    bindings: BTreeMap<String, Location>,
    
    /// Location constraints
    constraints: Vec<LocationConstraint>,
    
    /// Deterministic system for generating fresh variables
    deterministic: DeterministicSystem,
}

impl LocationContext {
    pub fn new() -> Self {
        LocationContext {
            bindings: BTreeMap::new(),
            constraints: Vec::new(),
            deterministic: DeterministicSystem::new(),
        }
    }
    
    /// Bind a variable to a location
    pub fn bind(&mut self, var: String, location: Location) {
        self.bindings.insert(var, location);
    }
    
    /// Look up a variable's location
    pub fn lookup(&self, var: &str) -> Option<&Location> {
        self.bindings.get(var)
    }
    
    /// Add a location constraint
    pub fn add_constraint(&mut self, constraint: LocationConstraint) {
        self.constraints.push(constraint);
    }
    
    /// Generate a fresh location variable
    pub fn fresh_location(&mut self) -> Location {
        Location::fresh_variable(&mut self.deterministic)
    }
    
    /// Solve all location constraints
    pub fn solve_constraints(&mut self) -> Result<(), LocationError> {
        // Simple constraint solver - in practice this would be more sophisticated
        for constraint in &self.constraints.clone() {
            match constraint {
                LocationConstraint::Equal(loc1, loc2) => {
                    let mut unifier = LocationUnifier::new();
                    if unifier.unify(loc1, loc2).is_none() {
                        return Err(LocationError::UnificationFailed(loc1.clone(), loc2.clone()));
                    }
                    // Apply substitutions
                    for (var, loc) in unifier.substitutions {
                        self.bindings.insert(var, loc);
                    }
                }
                LocationConstraint::Concrete(loc) => {
                    if !loc.substitute(&self.bindings).is_concrete() {
                        return Err(LocationError::NotConcrete(loc.clone()));
                    }
                }
                LocationConstraint::Local(loc) => {
                    let resolved = loc.substitute(&self.bindings);
                    if !resolved.is_local() {
                        return Err(LocationError::NotLocal(resolved));
                    }
                }
                LocationConstraint::Remote(loc) => {
                    let resolved = loc.substitute(&self.bindings);
                    if !resolved.is_remote() {
                        return Err(LocationError::NotRemote(resolved));
                    }
                }
                LocationConstraint::Reachable(from, to) => {
                    let from_resolved = from.substitute(&self.bindings);
                    let to_resolved = to.substitute(&self.bindings);
                    if !from_resolved.can_reach(&to_resolved) {
                        return Err(LocationError::NotReachable(from_resolved, to_resolved));
                    }
                }
                LocationConstraint::CoLocated(locs) => {
                    let resolved: Vec<_> = locs.iter()
                        .map(|loc| loc.substitute(&self.bindings))
                        .collect();
                    
                    // Check that all locations are the same
                    if let Some(first) = resolved.first() {
                        if !resolved.iter().all(|loc| loc == first) {
                            return Err(LocationError::NotCoLocated(resolved));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Errors in location operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocationError {
    /// Location unification failed
    UnificationFailed(Location, Location),
    
    /// Location is not concrete
    NotConcrete(Location),
    
    /// Location is not local
    NotLocal(Location),
    
    /// Location is not remote
    NotRemote(Location),
    
    /// Locations are not reachable
    NotReachable(Location, Location),
    
    /// Locations are not co-located
    NotCoLocated(Vec<Location>),
    
    /// Variable not found
    VariableNotFound(String),
    
    /// Cyclic location dependency
    CyclicDependency(Vec<Location>),
}

impl std::fmt::Display for LocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocationError::UnificationFailed(loc1, loc2) => {
                write!(f, "Cannot unify locations {:?} and {:?}", loc1, loc2)
            }
            LocationError::NotConcrete(loc) => write!(f, "Location {:?} is not concrete", loc),
            LocationError::NotLocal(loc) => write!(f, "Location {:?} is not local", loc),
            LocationError::NotRemote(loc) => write!(f, "Location {:?} is not remote", loc),
            LocationError::NotReachable(from, to) => {
                write!(f, "Location {:?} cannot reach {:?}", from, to)
            }
            LocationError::NotCoLocated(locs) => {
                write!(f, "Locations {:?} are not co-located", locs)
            }
            LocationError::VariableNotFound(var) => write!(f, "Location variable '{}' not found", var),
            LocationError::CyclicDependency(locs) => {
                write!(f, "Cyclic location dependency: {:?}", locs)
            }
        }
    }
}

impl std::error::Error for LocationError {}

// SSZ implementation for Location
impl Encode for Location {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        1 + match self {
            Location::Local => 0,
            Location::Remote(s) | Location::Domain(s) | Location::Variable(s) => 4 + s.len(),
            Location::Composite(locs) => 4 + locs.iter().map(|loc| loc.ssz_bytes_len()).sum::<usize>(),
            Location::Any | Location::None => 0,
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        match self {
            Location::Local => {
                0u8.ssz_append(buf);
            }
            Location::Remote(s) => {
                1u8.ssz_append(buf);
                (s.len() as u32).ssz_append(buf);
                buf.extend_from_slice(s.as_bytes());
            }
            Location::Domain(s) => {
                2u8.ssz_append(buf);
                (s.len() as u32).ssz_append(buf);
                buf.extend_from_slice(s.as_bytes());
            }
            Location::Composite(locs) => {
                3u8.ssz_append(buf);
                (locs.len() as u32).ssz_append(buf);
                for loc in locs {
                    loc.ssz_append(buf);
                }
            }
            Location::Variable(s) => {
                4u8.ssz_append(buf);
                (s.len() as u32).ssz_append(buf);
                buf.extend_from_slice(s.as_bytes());
            }
            Location::Any => {
                5u8.ssz_append(buf);
            }
            Location::None => {
                6u8.ssz_append(buf);
            }
        }
    }
}

impl Decode for Location {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (result, remainder) = Self::decode_with_remainder(bytes)?;
        if !remainder.is_empty() {
            return Err(DecodeError::BytesInvalid("Trailing bytes after decoding".to_string()));
        }
        Ok(result)
    }
}

impl DecodeWithRemainder for Location {
    fn decode_with_remainder(bytes: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError::InvalidByteLength { len: 0, expected: 1 });
        }
        
        let variant = bytes[0];
        let mut offset = 1;
        
        match variant {
            0 => Ok((Location::Local, &bytes[offset..])),
            1 | 2 | 4 => { // Remote, Domain, Variable
                if offset + 4 > bytes.len() {
                    return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 4 });
                }
                
                let len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
                offset += 4;
                
                if offset + len > bytes.len() {
                    return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: len });
                }
                
                let s = String::from_utf8(bytes[offset..offset+len].to_vec())
                    .map_err(|_| DecodeError::BytesInvalid("Invalid UTF-8".into()))?;
                offset += len;
                
                let location = match variant {
                    1 => Location::Remote(s),
                    2 => Location::Domain(s),
                    4 => Location::Variable(s),
                    _ => unreachable!(),
                };
                
                Ok((location, &bytes[offset..]))
            }
            3 => { // Composite
                if offset + 4 > bytes.len() {
                    return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 4 });
                }
                
                let count = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
                offset += 4;
                
                let mut locs = Vec::new();
                let mut remaining = &bytes[offset..];
                
                for _ in 0..count {
                    let (loc, new_remaining) = Location::decode_with_remainder(remaining)?;
                    locs.push(loc);
                    remaining = new_remaining;
                }
                
                Ok((Location::Composite(locs), remaining))
            }
            5 => Ok((Location::Any, &bytes[offset..])),
            6 => Ok((Location::None, &bytes[offset..])),
            _ => Err(DecodeError::BytesInvalid(format!("Invalid Location variant: {}", variant))),
        }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Local => write!(f, "local"),
            Location::Remote(id) => write!(f, "remote:{}", id),
            Location::Domain(id) => write!(f, "domain:{}", id),
            Location::Composite(locs) => {
                write!(f, "composite(")?;
                for (i, loc) in locs.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", loc)?;
                }
                write!(f, ")")
            }
            Location::Variable(name) => write!(f, "var:{}", name),
            Location::Any => write!(f, "any"),
            Location::None => write!(f, "none"),
        }
    }
}

impl Default for Location {
    fn default() -> Self {
        Location::Local
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_location_composition() {
        let local = Location::Local;
        let remote = Location::remote("server1");
        
        let composed = local.compose(remote.clone());
        assert_eq!(composed, Location::Composite(vec![Location::Local, remote]));
    }
    
    #[test]
    fn test_location_distance() {
        let local = Location::Local;
        let remote = Location::remote("server1");
        
        assert_eq!(local.distance_to(&local), Some(0));
        assert_eq!(local.distance_to(&remote), Some(1));
        assert_eq!(remote.distance_to(&local), Some(1));
    }
    
    #[test]
    fn test_location_routing() {
        let local = Location::Local;
        let remote = Location::remote("server1");
        
        let route = local.route_to(&remote).unwrap();
        assert_eq!(route, vec![Location::Local, Location::remote("server1")]);
    }
    
    #[test]
    fn test_location_unification() {
        let var = Location::variable("X");
        let local = Location::Local;
        
        let unification = var.unify(&local).unwrap();
        assert_eq!(unification.unified, local);
        assert_eq!(unification.substitutions.get("X"), Some(&local));
    }
    
    #[test]
    fn test_location_context() {
        let mut ctx = LocationContext::new();
        
        ctx.bind("x".to_string(), Location::Local);
        assert_eq!(ctx.lookup("x"), Some(&Location::Local));
        
        ctx.add_constraint(LocationConstraint::Local(Location::variable("x")));
        assert!(ctx.solve_constraints().is_ok());
    }
} 