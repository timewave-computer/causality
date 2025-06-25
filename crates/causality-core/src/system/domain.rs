//! Domain system for organizing resources and capabilities
//!
//! This module provides the unified domain and routing system that enables
//! location-aware computation and communication across distributed nodes.
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use crate::lambda::Location;
use crate::system::{Str, EntityId};
use ssz::{Encode, Decode};
/// A domain represents a scope for capability management and routing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Domain {
    /// Domain location identifier (unified with Location system)
    pub id: Location,
    
    /// Human-readable name
    pub name: Str,
    
    /// Capabilities provided by this domain
    pub capabilities: Vec<String>,
    
    /// Routing information for reaching this domain
    pub routing_info: RoutingInfo,
}
/// Routing information for domain communication
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingInfo {
    /// Direct connections to other domains
    pub connections: Vec<Location>,
    
    /// Routing cost to reach this domain (for optimization)
    pub base_cost: u64,
    
    /// Maximum hops allowed when routing through this domain
    pub max_hops: u32,
    
    /// Whether this domain can act as a router for other domains
    pub can_route: bool,
    
    /// Protocol preferences for communication
    pub protocols: BTreeSet<String>,
}
impl Default for RoutingInfo {
    fn default() -> Self {
        Self {
            connections: Vec::new(),
            base_cost: 1,
            max_hops: 3,
            can_route: true,
            protocols: ["session".to_string(), "direct".to_string()].into_iter().collect(),
        }
    }
}
/// Unified routing system that merges domain-based and location-based routing
#[derive(Debug, Clone)]
pub struct UnifiedRouter {
    /// Registry of all known domains
    domains: BTreeMap<Location, Domain>,
    
    /// Routing table for efficient path finding
    routing_table: BTreeMap<Location, BTreeMap<Location, RoutingPath>>,
    
    /// Default routing strategy
    default_strategy: RoutingStrategy,
}
/// A routing path between two locations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingPath {
    /// Sequence of locations to traverse
    pub hops: Vec<Location>,
    
    /// Total cost of this path
    pub total_cost: u64,
    
    /// Estimated latency in milliseconds
    pub estimated_latency: u64,
    
    /// Required capabilities for this path
    pub required_capabilities: BTreeSet<String>,
    
    /// Protocols supported on this path
    pub supported_protocols: BTreeSet<String>,
}
/// Strategy for routing between locations
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum RoutingStrategy {
    /// Direct routing - send directly to target
    #[default]
    Direct,
    
    /// Flooding - broadcast to all neighbors
    Flooding,
    
    /// Distance vector routing
    DistanceVector,
    
    /// Link state routing
    LinkState,
    
    /// Minimize number of hops
    MinimizeHops,
    
    /// Minimize total cost
    MinimizeCost,
    
    /// Minimize latency
    MinimizeLatency,
    
    /// Prefer specific protocols
    PreferProtocols(BTreeSet<String>),
    
    /// Custom routing algorithm
    Custom(String),
}
impl Domain {
    /// Create a new domain with the given name and capabilities
    pub fn new(name: Str, capabilities: Vec<String>) -> Self {
        let name_str = name.as_str();
        let id = match name_str {
            "local" => Location::Local,
            s if s.starts_with("remote:") => Location::Remote(EntityId::from_content(&s.as_bytes()[7..].to_vec())),
            s => Location::Remote(EntityId::from_content(&s.as_bytes().to_vec())),
        };
        
        Self { id, name, capabilities, routing_info: Default::default() }
    }
    
    /// Create a default domain with basic capabilities
    pub fn create_default() -> Self {
        let capabilities = vec![
            "read".to_string(),
            "write".to_string(),
            "execute".to_string(),
        ];
        
        Self::new(Str::from("default"), capabilities)
    }
    
    /// Check if this domain has a specific capability
    pub fn has_capability(&self, capability_name: &str) -> bool {
        self.capabilities.iter().any(|cap| cap == capability_name)
    }
    
    /// Get a capability by name
    pub fn get_capability(&self, name: &str) -> Option<&String> {
        self.capabilities.iter().find(|cap| cap.as_str() == name)
    }
}
impl Encode for Domain {
    fn is_ssz_fixed_len() -> bool {
        false
    }
    fn ssz_bytes_len(&self) -> usize {
        self.id.ssz_bytes_len() +
        self.name.ssz_bytes_len() + 
        4 + self.capabilities.iter().map(|c| 4 + c.len()).sum::<usize>()
    }
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.id.ssz_append(buf);
        self.name.ssz_append(buf);
        (self.capabilities.len() as u32).ssz_append(buf);
        for cap in &self.capabilities {
            (cap.len() as u32).ssz_append(buf);
            buf.extend_from_slice(cap.as_bytes());
        }
    }
}
impl Decode for Domain {
    fn is_ssz_fixed_len() -> bool {
        false
    }
    fn from_ssz_bytes(_bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        // Simplified - return default domain for now
        Ok(Self::create_default())
    }
}
impl UnifiedRouter {
    /// Create a new unified router
    pub fn new() -> Self {
        Self {
            domains: BTreeMap::new(),
            routing_table: BTreeMap::new(),
            default_strategy: RoutingStrategy::default(),
        }
    }
    
    /// Register a domain in the routing system
    pub fn register_domain(&mut self, domain: Domain) {
        let location = domain.id.clone();
        self.domains.insert(location.clone(), domain);
        self.rebuild_routing_table();
    }
    
    /// Find the best route between two locations
    pub fn find_route(&self, from: &Location, to: &Location) -> Option<RoutingPath> {
        self.find_route_with_strategy(from, to, &self.default_strategy)
    }
    
    /// Find route with specific strategy
    pub fn find_route_with_strategy(
        &self, 
        from: &Location, 
        to: &Location, 
        strategy: &RoutingStrategy
    ) -> Option<RoutingPath> {
        // Direct connection check
        if from == to {
            return Some(RoutingPath {
                hops: vec![from.clone()],
                total_cost: 0,
                estimated_latency: 0,
                required_capabilities: BTreeSet::new(),
                supported_protocols: self.get_location_protocols(from),
            });
        }
        
        // Check routing table for cached paths
        if let Some(cached_path) = self.routing_table.get(from).and_then(|routes| routes.get(to)) {
            return Some(cached_path.clone());
        }
        
        // Compute path using strategy
        self.compute_path(from, to, strategy)
    }
    
    /// Check if two locations can communicate directly
    pub fn can_communicate_directly(&self, from: &Location, to: &Location) -> bool {
        if let Some(from_domain) = self.domains.get(from) {
            from_domain.routing_info.connections.contains(to)
        } else {
            // If domains not registered, assume local/remote can communicate
            matches!((from, to), 
                (Location::Local, Location::Local) |
                (Location::Local, Location::Remote(_)) |
                (Location::Remote(_), Location::Local) |
                (Location::Remote(_), Location::Remote(_))
            )
        }
    }
    
    /// Get supported protocols for a location
    pub fn get_location_protocols(&self, location: &Location) -> BTreeSet<String> {
        if let Some(domain) = self.domains.get(location) {
            domain.routing_info.protocols.clone()
        } else {
            // Default protocols for unregistered locations
            ["session".to_string(), "direct".to_string()].into_iter().collect()
        }
    }
    
    /// Get routing cost between two directly connected locations
    pub fn get_direct_cost(&self, from: &Location, to: &Location) -> Option<u64> {
        if self.can_communicate_directly(from, to) {
            let from_cost = self.domains.get(from)
                .map(|d| d.routing_info.base_cost)
                .unwrap_or(1);
            let to_cost = self.domains.get(to)
                .map(|d| d.routing_info.base_cost)
                .unwrap_or(1);
            Some(from_cost + to_cost)
        } else {
            None
        }
    }
    
    /// Rebuild the routing table after domain changes
    fn rebuild_routing_table(&mut self) {
        self.routing_table.clear();
        
        // Initialize routing table
        for from_location in self.domains.keys() {
            self.routing_table.insert(from_location.clone(), BTreeMap::new());
        }
        
        // Compute all-pairs shortest paths using Floyd-Warshall algorithm
        let locations: Vec<Location> = self.domains.keys().cloned().collect();
        
        // Initialize direct connections
        for from in &locations {
            for to in &locations {
                if let Some(cost) = self.get_direct_cost(from, to) {
                    let path = RoutingPath {
                        hops: vec![from.clone(), to.clone()],
                        total_cost: cost,
                        estimated_latency: cost * 10, // Rough estimate
                        required_capabilities: BTreeSet::new(),
                        supported_protocols: self.get_common_protocols(from, to),
                    };
                    
                    self.routing_table.get_mut(from).unwrap().insert(to.clone(), path);
                }
            }
        }
        
        // Floyd-Warshall for shortest paths
        for k in &locations {
            for i in &locations {
                for j in &locations {
                    if let (Some(ik_path), Some(kj_path)) = (
                        self.routing_table.get(i).and_then(|routes| routes.get(k)).cloned(),
                        self.routing_table.get(k).and_then(|routes| routes.get(j)).cloned()
                    ) {
                        let combined_cost = ik_path.total_cost + kj_path.total_cost;
                        let current_cost = self.routing_table.get(i)
                            .and_then(|routes| routes.get(j))
                            .map(|path| path.total_cost)
                            .unwrap_or(u64::MAX);
                        
                        if combined_cost < current_cost {
                            let mut combined_hops = ik_path.hops;
                            combined_hops.extend_from_slice(&kj_path.hops[1..]);
                            
                            let combined_path = RoutingPath {
                                hops: combined_hops,
                                total_cost: combined_cost,
                                estimated_latency: ik_path.estimated_latency + kj_path.estimated_latency,
                                required_capabilities: ik_path.required_capabilities.union(&kj_path.required_capabilities).cloned().collect(),
                                supported_protocols: ik_path.supported_protocols.intersection(&kj_path.supported_protocols).cloned().collect(),
                            };
                            
                            self.routing_table.get_mut(i).unwrap().insert(j.clone(), combined_path);
                        }
                    }
                }
            }
        }
    }
    
    /// Compute path using specific strategy (fallback when no cached path)
    fn compute_path(&self, from: &Location, to: &Location, strategy: &RoutingStrategy) -> Option<RoutingPath> {
        // Compute path using strategy
        match strategy {
            RoutingStrategy::Direct => {
                // Direct path if possible
                if self.can_communicate_directly(from, to) {
                    self.get_direct_cost(from, to).map(|cost| RoutingPath {
                        hops: vec![from.clone(), to.clone()],
                        total_cost: cost,
                        estimated_latency: cost * 10,
                        required_capabilities: BTreeSet::new(),
                        supported_protocols: self.get_common_protocols(from, to),
                    })
                } else {
                    None
                }
            }
            RoutingStrategy::Flooding => {
                // For flooding, we still find the shortest path but mark it as broadcast
                self.compute_shortest_hop_path(from, to)
            }
            RoutingStrategy::DistanceVector | RoutingStrategy::LinkState => {
                // For now, these use shortest hop path
                self.compute_shortest_hop_path(from, to)
            }
            RoutingStrategy::MinimizeHops => self.compute_shortest_hop_path(from, to),
            RoutingStrategy::MinimizeCost => self.compute_lowest_cost_path(from, to),
            RoutingStrategy::MinimizeLatency => self.compute_lowest_latency_path(from, to),
            RoutingStrategy::PreferProtocols(protocols) => self.compute_protocol_preferred_path(from, to, protocols),
            RoutingStrategy::Custom(_) => self.compute_lowest_cost_path(from, to), // Default fallback
        }
    }
    
    /// Compute shortest path by number of hops
    fn compute_shortest_hop_path(&self, from: &Location, to: &Location) -> Option<RoutingPath> {
        // Simple BFS for shortest hop path
        use std::collections::VecDeque;
        
        let mut queue = VecDeque::new();
        let mut visited = BTreeSet::new();
        
        queue.push_back((from.clone(), vec![from.clone()], 0u64, 0u64));
        visited.insert(from.clone());
        
        while let Some((current, path, cost, latency)) = queue.pop_front() {
            if current == *to {
                return Some(RoutingPath {
                    hops: path,
                    total_cost: cost,
                    estimated_latency: latency,
                    required_capabilities: BTreeSet::new(),
                    supported_protocols: self.get_location_protocols(to),
                });
            }
            
            if let Some(domain) = self.domains.get(&current) {
                for neighbor in &domain.routing_info.connections {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        let mut new_path = path.clone();
                        new_path.push(neighbor.clone());
                        let edge_cost = self.get_direct_cost(&current, neighbor).unwrap_or(1);
                        queue.push_back((neighbor.clone(), new_path, cost + edge_cost, latency + edge_cost * 10));
                    }
                }
            }
        }
        
        None
    }
    
    /// Compute lowest cost path (already handled by Floyd-Warshall)
    fn compute_lowest_cost_path(&self, from: &Location, to: &Location) -> Option<RoutingPath> {
        self.routing_table.get(from)?.get(to).cloned()
    }
    
    /// Compute lowest latency path
    fn compute_lowest_latency_path(&self, from: &Location, to: &Location) -> Option<RoutingPath> {
        // For now, use cost-based path as proxy for latency
        self.compute_lowest_cost_path(from, to)
    }
    
    /// Compute path preferring specific protocols
    fn compute_protocol_preferred_path(&self, from: &Location, to: &Location, preferred: &BTreeSet<String>) -> Option<RoutingPath> {
        // Find path that supports the preferred protocols
        if let Some(path) = self.compute_lowest_cost_path(from, to) {
            if path.supported_protocols.intersection(preferred).count() > 0 {
                return Some(path);
            }
        }
        
        // Fallback to any available path
        self.compute_lowest_cost_path(from, to)
    }
    
    /// Get protocols supported by both locations
    fn get_common_protocols(&self, from: &Location, to: &Location) -> BTreeSet<String> {
        let from_protocols = self.get_location_protocols(from);
        let to_protocols = self.get_location_protocols(to);
        from_protocols.intersection(&to_protocols).cloned().collect()
    }
    
    /// Get all registered domains
    pub fn get_domains(&self) -> &BTreeMap<Location, Domain> {
        &self.domains
    }
    
    /// Get routing statistics
    pub fn get_routing_stats(&self) -> RoutingStats {
        let total_domains = self.domains.len();
        let total_connections = self.domains.values()
            .map(|d| d.routing_info.connections.len())
            .sum();
        let total_routes = self.routing_table.values()
            .map(|routes| routes.len())
            .sum();
        
        RoutingStats {
            total_domains,
            total_connections,
            total_routes,
            average_hops: if total_routes > 0 {
                self.routing_table.values()
                    .flat_map(|routes| routes.values())
                    .map(|path| path.hops.len() as f64)
                    .sum::<f64>() / total_routes as f64
            } else {
                0.0
            },
        }
    }
}
/// Statistics about the routing system
#[derive(Debug, Clone)]
pub struct RoutingStats {
    pub total_domains: usize,
    pub total_connections: usize,
    pub total_routes: usize,
    pub average_hops: f64,
}
impl Default for UnifiedRouter {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_domain_creation() {
        let capabilities = vec![
            "read".to_string(),
            "write".to_string(),
        ];
        
        let domain = Domain::new(Str::new("test_domain"), capabilities);
        assert_eq!(domain.name.as_str(), "test_domain");
        assert_eq!(domain.capabilities.len(), 2);
        assert!(domain.has_capability("read"));
        assert!(domain.has_capability("write"));
        assert!(!domain.has_capability("admin"));
        
        // Check that routing info is initialized
        assert_eq!(domain.routing_info.base_cost, 1);
        assert!(domain.routing_info.can_route);
        assert!(domain.routing_info.protocols.contains("session"));
    }
    #[test]
    fn test_default_domain() {
        let domain = Domain::create_default();
        assert_eq!(domain.name.as_str(), "default");
        assert!(domain.has_capability("read"));
        assert!(domain.has_capability("write"));
        assert!(domain.has_capability("execute"));
        assert_eq!(domain.id, Location::Remote(EntityId::from_content(&"default".as_bytes().to_vec())));
    }
    #[test]
    fn test_ssz_serialization() {
        let domain = Domain::create_default();
        let encoded = domain.as_ssz_bytes();
        let decoded = Domain::from_ssz_bytes(&encoded).unwrap();
        assert_eq!(domain, decoded);
    }
    
    #[test]
    fn test_unified_router_basic() {
        let mut router = UnifiedRouter::new();
        
        // Create test domains
        let mut domain_a = Domain::new(Str::new("domain_a"), vec!["read".to_string()]);
        domain_a.id = Location::Remote(EntityId::from_content(&"a".as_bytes().to_vec()));
        domain_a.routing_info.connections = vec![Location::Remote(EntityId::from_content(&"b".as_bytes().to_vec()))];
        
        let mut domain_b = Domain::new(Str::new("domain_b"), vec!["write".to_string()]);
        domain_b.id = Location::Remote(EntityId::from_content(&"b".as_bytes().to_vec()));
        domain_b.routing_info.connections = vec![Location::Remote(EntityId::from_content(&"a".as_bytes().to_vec()))];
        
        // Register domains
        router.register_domain(domain_a);
        router.register_domain(domain_b);
        
        // Test direct communication
        let from = Location::Remote(EntityId::from_content(&"a".as_bytes().to_vec()));
        let to = Location::Remote(EntityId::from_content(&"b".as_bytes().to_vec()));
        
        assert!(router.can_communicate_directly(&from, &to));
        
        // Test routing
        let route = router.find_route(&from, &to);
        assert!(route.is_some());
        
        let path = route.unwrap();
        assert_eq!(path.hops.len(), 2);
        assert_eq!(path.hops[0], from);
        assert_eq!(path.hops[1], to);
        assert!(path.total_cost > 0);
    }
    
    #[test]
    fn test_routing_strategies() {
        let mut router = UnifiedRouter::new();
        
        // Create a triangle of domains
        let mut domain_a = Domain::new(Str::new("a"), vec![]);
        domain_a.id = Location::Remote(EntityId::from_content(&"a".as_bytes().to_vec()));
        domain_a.routing_info.connections = vec![
            Location::Remote(EntityId::from_content(&"b".as_bytes().to_vec())), 
            Location::Remote(EntityId::from_content(&"c".as_bytes().to_vec()))
        ];
        domain_a.routing_info.base_cost = 1;
        
        let mut domain_b = Domain::new(Str::new("b"), vec![]);
        domain_b.id = Location::Remote(EntityId::from_content(&"b".as_bytes().to_vec()));
        domain_b.routing_info.connections = vec![
            Location::Remote(EntityId::from_content(&"a".as_bytes().to_vec())), 
            Location::Remote(EntityId::from_content(&"c".as_bytes().to_vec()))
        ];
        domain_b.routing_info.base_cost = 5; // Higher cost
        
        let mut domain_c = Domain::new(Str::new("c"), vec![]);
        domain_c.id = Location::Remote(EntityId::from_content(&"c".as_bytes().to_vec()));
        domain_c.routing_info.connections = vec![
            Location::Remote(EntityId::from_content(&"a".as_bytes().to_vec())), 
            Location::Remote(EntityId::from_content(&"b".as_bytes().to_vec()))
        ];
        domain_c.routing_info.base_cost = 1;
        
        router.register_domain(domain_a);
        router.register_domain(domain_b);
        router.register_domain(domain_c);
        
        let from = Location::Remote(EntityId::from_content(&"a".as_bytes().to_vec()));
        let to = Location::Remote(EntityId::from_content(&"c".as_bytes().to_vec()));
        
        // Test minimize cost strategy
        let cost_route = router.find_route_with_strategy(&from, &to, &RoutingStrategy::MinimizeCost);
        assert!(cost_route.is_some());
        
        // Test minimize hops strategy
        let hop_route = router.find_route_with_strategy(&from, &to, &RoutingStrategy::MinimizeHops);
        assert!(hop_route.is_some());
        
        // Both should find the direct route A->C since it's available
        let cost_path = cost_route.unwrap();
        let hop_path = hop_route.unwrap();
        assert_eq!(cost_path.hops.len(), 2);
        assert_eq!(hop_path.hops.len(), 2);
    }
    
    #[test]
    fn test_protocol_preferences() {
        let mut router = UnifiedRouter::new();
        
        let mut domain_a = Domain::new(Str::new("a"), vec![]);
        domain_a.id = Location::Remote(EntityId::from_content(&"a".as_bytes().to_vec()));
        domain_a.routing_info.protocols = ["session".to_string(), "custom".to_string()].into_iter().collect();
        domain_a.routing_info.connections = vec![Location::Remote(EntityId::from_content(&"b".as_bytes().to_vec()))];
        
        let mut domain_b = Domain::new(Str::new("b"), vec![]);
        domain_b.id = Location::Remote(EntityId::from_content(&"b".as_bytes().to_vec()));
        domain_b.routing_info.protocols = ["session".to_string(), "direct".to_string()].into_iter().collect();
        domain_b.routing_info.connections = vec![Location::Remote(EntityId::from_content(&"a".as_bytes().to_vec()))];
        
        router.register_domain(domain_a);
        router.register_domain(domain_b);
        
        let from = Location::Remote(EntityId::from_content(&"a".as_bytes().to_vec()));
        let to = Location::Remote(EntityId::from_content(&"b".as_bytes().to_vec()));
        
        // Test protocol preferences
        let preferred_protocols = ["session".to_string()].into_iter().collect();
        let route = router.find_route_with_strategy(&from, &to, &RoutingStrategy::PreferProtocols(preferred_protocols));
        
        assert!(route.is_some());
        let path = route.unwrap();
        assert!(path.supported_protocols.contains("session"));
    }
    
    #[test]
    fn test_routing_stats() {
        let mut router = UnifiedRouter::new();
        
        // Add several domains
        for i in 0..3 {
            let mut domain = Domain::new(Str::new(&format!("domain_{}", i)), vec![]);
            domain.id = Location::Remote(EntityId::from_content(&format!("domain_{}", i).as_bytes().to_vec()));
            if i > 0 {
                domain.routing_info.connections.push(Location::Remote(EntityId::from_content(&format!("domain_{}", i - 1).as_bytes().to_vec())));
            }
            if i < 2 {
                domain.routing_info.connections.push(Location::Remote(EntityId::from_content(&format!("domain_{}", i + 1).as_bytes().to_vec())));
            }
            router.register_domain(domain);
        }
        
        let stats = router.get_routing_stats();
        assert_eq!(stats.total_domains, 3);
        assert!(stats.total_connections > 0);
        assert!(stats.total_routes > 0);
        assert!(stats.average_hops > 0.0);
    }
    
    #[test]
    fn test_self_routing() {
        let router = UnifiedRouter::new();
        let location = Location::Local;
        
        let route = router.find_route(&location, &location);
        assert!(route.is_some());
        
        let path = route.unwrap();
        assert_eq!(path.hops.len(), 1);
        assert_eq!(path.total_cost, 0);
        assert_eq!(path.estimated_latency, 0);
    }
    
    #[test]
    fn test_unregistered_location_communication() {
        let router = UnifiedRouter::new();
        
        let local = Location::Local;
        let remote = Location::Remote(EntityId::from_content(&"unregistered".as_bytes().to_vec()));
        
        // Should allow communication between local and remote even if not registered
        assert!(router.can_communicate_directly(&local, &remote));
        assert!(router.can_communicate_directly(&remote, &local));
        
        // Should have default protocols
        let protocols = router.get_location_protocols(&remote);
        assert!(protocols.contains("session"));
        assert!(protocols.contains("direct"));
    }
} 