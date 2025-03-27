// Actor addressing system
//
// This module provides abstractions for addressing actors in the actor system.

use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use serde::{Serialize, Deserialize};
use crate::actor::message::Message;
use crate::capability::{ContentHash, ContentAddressed, ContentAddressingError};
use crate::serialization::Serializer;

/// The internal representation of an address
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct AddressInner {
    /// The unique identifier of the address (content hash)
    pub id: ContentHash,
    
    /// The name of the actor
    pub name: String,
    
    /// The type of the actor
    pub actor_type: String,
}

impl AddressInner {
    /// Create a new address inner with a content hash
    fn new(name: String, actor_type: String) -> Result<Self, ContentAddressingError> {
        // First create without the hash
        let address_data = Self {
            // Temporary placeholder
            id: ContentHash::from_bytes([0; 32]),
            name,
            actor_type,
        };
        
        // Calculate the content hash
        let hash = ContentHash::for_object(&address_data)?;
        
        // Return the complete address inner
        Ok(Self {
            id: hash,
            name: address_data.name,
            actor_type: address_data.actor_type,
        })
    }
}

/// An address that uniquely identifies an actor in the system
#[derive(Clone)]
pub struct Address<M: Message> {
    /// The internal representation of the address
    pub(crate) inner: Arc<AddressInner>,
    
    /// Phantom data for the message type
    pub(crate) _phantom: std::marker::PhantomData<M>,
}

impl<M: Message> Address<M> {
    /// Create a new address for an actor
    pub fn new(name: impl Into<String>, actor_type: impl Into<String>) -> Self {
        let name = name.into();
        let actor_type = actor_type.into();
        
        let inner = AddressInner::new(name.clone(), actor_type.clone())
            .unwrap_or_else(|_| {
                // If content addressing fails, create with a random hash as fallback
                let random_bytes = rand::random::<[u8; 32]>();
                AddressInner {
                    id: ContentHash::from_bytes(random_bytes),
                    name,
                    actor_type,
                }
            });
        
        Self {
            inner: Arc::new(inner),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Get the content hash that uniquely identifies this actor
    pub fn content_hash(&self) -> ContentHash {
        self.inner.id
    }
    
    /// Get the name of the actor
    pub fn name(&self) -> &str {
        &self.inner.name
    }
    
    /// Get the type of the actor
    pub fn actor_type(&self) -> &str {
        &self.inner.actor_type
    }
    
    /// Create an address to represent an anonymous actor
    pub fn anonymous(actor_type: impl Into<String>) -> Self {
        let unique_id = format!("{:x}", rand::random::<u64>());
        Self::new(format!("anonymous-{}", unique_id), actor_type)
    }
    
    /// Check if this address refers to an anonymous actor
    pub fn is_anonymous(&self) -> bool {
        self.inner.name.starts_with("anonymous-")
    }
    
    /// Create a typed address for another message type
    pub fn typed<N: Message>(&self) -> Address<N> {
        Address {
            inner: self.inner.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Verify that the address's content hash is valid
    pub fn verify(&self) -> Result<bool, ContentAddressingError> {
        // Create a version for verification (without the hash)
        let for_verification = AddressInner {
            id: ContentHash::from_bytes([0; 32]),
            name: self.inner.name.clone(),
            actor_type: self.inner.actor_type.clone(),
        };
        
        // Calculate the hash
        let calculated_hash = ContentHash::for_object(&for_verification)?;
        
        // Compare with the stored hash
        Ok(calculated_hash == self.inner.id)
    }
}

impl<M: Message> ContentAddressed for Address<M> {
    fn content_hash(&self) -> ContentHash {
        self.inner.id
    }
    
    fn verify(&self) -> bool {
        self.verify().unwrap_or(false)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError> {
        // Serialize the address inner
        Serializer::to_bytes(&self.inner)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError> {
        // Deserialize the address inner
        let inner = Serializer::from_bytes::<AddressInner>(bytes)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))?;
            
        Ok(Self {
            inner: Arc::new(inner),
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<M: Message> Debug for Address<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Address")
            .field("id", &self.inner.id)
            .field("name", &self.inner.name)
            .field("actor_type", &self.inner.actor_type)
            .finish()
    }
}

impl<M: Message> Display for Address<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.inner.name, self.inner.id)
    }
}

impl<M: Message> PartialEq for Address<M> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}

impl<M: Message> Eq for Address<M> {}

impl<M: Message> Hash for Address<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.id.hash(state);
    }
}

/// A trait for resolving actor addresses
pub trait AddressResolver {
    /// The error type returned when address resolution fails
    type Error: std::error::Error;
    
    /// Resolve an actor address by name
    fn resolve_by_name<M: Message>(&self, name: &str) -> Result<Address<M>, Self::Error>;
    
    /// Resolve an actor address by content hash
    fn resolve_by_hash<M: Message>(&self, hash: ContentHash) -> Result<Address<M>, Self::Error>;
    
    /// Find all actors of a specific type
    fn find_by_type<M: Message>(&self, actor_type: &str) -> Result<Vec<Address<M>>, Self::Error>;
    
    /// Check if an actor exists
    fn exists(&self, address: &Address<impl Message>) -> bool;
}

/// Address path for hierarchical actor systems
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddressPath {
    /// The components of the path
    components: Vec<String>,
}

impl AddressPath {
    /// Create a new address path
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
    
    /// Create an address path from a string
    pub fn from_string(path: impl AsRef<str>) -> Self {
        let components = path
            .as_ref()
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        
        Self { components }
    }
    
    /// Get the components of the path
    pub fn components(&self) -> &[String] {
        &self.components
    }
    
    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
    
    /// Get the length of the path
    pub fn len(&self) -> usize {
        self.components.len()
    }
    
    /// Append a component to the path
    pub fn append(&mut self, component: impl Into<String>) {
        self.components.push(component.into());
    }
    
    /// Append a component to the path and return a new path
    pub fn join(&self, component: impl AsRef<str>) -> Self {
        let mut components = self.components.clone();
        components.push(component.as_ref().to_string());
        Self { components }
    }
    
    /// Get the parent path
    pub fn parent(&self) -> Option<Self> {
        if self.components.is_empty() {
            None
        } else {
            let mut components = self.components.clone();
            components.pop();
            Some(Self { components })
        }
    }
    
    /// Get the last component of the path
    pub fn last(&self) -> Option<&str> {
        self.components.last().map(|s| s.as_str())
    }
}

impl Display for AddressPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.components.join("/"))
    }
}

impl From<&str> for AddressPath {
    fn from(s: &str) -> Self {
        Self::from_string(s)
    }
}

impl From<String> for AddressPath {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

/// A hierarchical address for actor systems with parent-child relationships
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct HierarchicalAddress<M: Message> {
    /// The base address
    address: Address<M>,
    
    /// The path in the hierarchy
    path: AddressPath,
}

impl<M: Message> HierarchicalAddress<M> {
    /// Create a new hierarchical address
    pub fn new(address: Address<M>, path: AddressPath) -> Self {
        Self { address, path }
    }
    
    /// Get the base address
    pub fn address(&self) -> &Address<M> {
        &self.address
    }
    
    /// Get the path
    pub fn path(&self) -> &AddressPath {
        &self.path
    }
    
    /// Create a child address
    pub fn child(&self, name: impl AsRef<str>) -> Self {
        Self {
            address: self.address.clone(),
            path: self.path.join(name),
        }
    }
    
    /// Get the parent address, if any
    pub fn parent(&self) -> Option<Self> {
        self.path.parent().map(|path| Self {
            address: self.address.clone(),
            path,
        })
    }
    
    /// Check if this address is a child of another
    pub fn is_child_of(&self, other: &Self) -> bool {
        // Must be on the same actor
        if self.address != other.address {
            return false;
        }
        
        // Other path must be prefix of this path
        if self.path.len() <= other.path.len() {
            return false;
        }
        
        let self_components = self.path.components();
        let other_components = other.path.components();
        
        other_components
            .iter()
            .zip(self_components.iter())
            .all(|(a, b)| a == b)
    }
    
    /// Check if this address is a parent of another
    pub fn is_parent_of(&self, other: &Self) -> bool {
        other.is_child_of(self)
    }
}

impl<M: Message> ContentAddressed for HierarchicalAddress<M> {
    fn content_hash(&self) -> ContentHash {
        // Hierarchical addresses add path information to the hash
        let path_str = self.path.to_string();
        let address_hash = self.address.content_hash();
        
        // Create a combined hash
        let combined = format!("{}:{}", address_hash.as_bytes().iter().fold(String::new(), |acc, b| format!("{}{:02x}", acc, b)), path_str);
        ContentHash::for_content(combined.as_bytes())
    }
    
    fn verify(&self) -> bool {
        // First verify the base address
        if !self.address.verify() {
            return false;
        }
        
        // For hierarchical addresses, we just need to make sure the content hash is consistent
        // since the path is part of the object itself
        true
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError> {
        // Create a serializable structure
        #[derive(Serialize)]
        struct HierarchicalAddressData {
            address_bytes: Vec<u8>,
            path_components: Vec<String>,
        }
        
        // Get the address bytes
        let address_bytes = self.address.to_bytes()?;
        
        // Create the serializable data
        let data = HierarchicalAddressData {
            address_bytes,
            path_components: self.path.components().to_vec(),
        };
        
        // Serialize it
        Serializer::to_bytes(&data)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError> {
        // Deserialize the hierarchical address data
        #[derive(Deserialize)]
        struct HierarchicalAddressData {
            address_bytes: Vec<u8>,
            path_components: Vec<String>,
        }
        
        // Deserialize the data
        let data = Serializer::from_bytes::<HierarchicalAddressData>(bytes)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))?;
            
        // Deserialize the address
        let address = Address::<M>::from_bytes(&data.address_bytes)?;
        
        // Create the path
        let path = AddressPath {
            components: data.path_components,
        };
        
        Ok(Self {
            address,
            path,
        })
    }
}

impl<M: Message> Debug for HierarchicalAddress<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HierarchicalAddress")
            .field("address", &self.address)
            .field("path", &self.path)
            .finish()
    }
}

impl<M: Message> Display for HierarchicalAddress<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.address, self.path)
    }
}

/// Helper functions for working with addresses
pub mod helpers {
    use super::*;
    
    /// Create a random address
    pub fn random_address<M: Message>(name: impl Into<String>, actor_type: impl Into<String>) -> Address<M> {
        Address::new(name, actor_type)
    }
    
    /// Create an anonymous address
    pub fn anonymous_address<M: Message>(actor_type: impl Into<String>) -> Address<M> {
        Address::anonymous(actor_type)
    }
    
    /// Create a hierarchical address
    pub fn hierarchical_address<M: Message>(
        name: impl Into<String>,
        actor_type: impl Into<String>,
        path: impl Into<AddressPath>,
    ) -> HierarchicalAddress<M> {
        let address = Address::new(name, actor_type);
        let path = path.into();
        HierarchicalAddress::new(address, path)
    }
    
    /// Convert an address to a hierarchical address at the root
    pub fn to_hierarchical<M: Message>(address: Address<M>) -> HierarchicalAddress<M> {
        HierarchicalAddress::new(address, AddressPath::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::message::Message;
    
    struct TestMessage;
    
    impl Message for TestMessage {
        type Response = ();
    }
    
    #[test]
    fn test_address() {
        let addr = Address::<TestMessage>::new("test-actor", "test");
        assert_eq!(addr.name(), "test-actor");
        assert_eq!(addr.actor_type(), "test");
        
        // Make sure content hash is derived correctly
        let addr2 = Address::<TestMessage>::new("test-actor", "test");
        assert_eq!(addr.content_hash(), addr2.content_hash());
        
        // Different names should have different hashes
        let addr3 = Address::<TestMessage>::new("different-actor", "test");
        assert_ne!(addr.content_hash(), addr3.content_hash());
        
        // Test anonymous address
        let anon = Address::<TestMessage>::anonymous("anon-type");
        assert!(anon.is_anonymous());
    }
    
    #[test]
    fn test_address_equality() {
        let addr1 = Address::<TestMessage>::new("test-actor", "test");
        let addr2 = Address::<TestMessage>::new("test-actor", "test");
        let addr3 = Address::<TestMessage>::new("another-actor", "test");
        
        assert_eq!(addr1, addr2);
        assert_ne!(addr1, addr3);
        
        // Same name/type should produce the same content hash
        assert_eq!(addr1.content_hash(), addr2.content_hash());
        
        // Type conversion shouldn't change the address
        let typed: Address<()> = addr1.typed();
        assert_eq!(typed.content_hash(), addr1.content_hash());
    }
    
    #[test]
    fn test_address_path() {
        let path = AddressPath::new();
        assert!(path.is_empty());
        assert_eq!(path.len(), 0);
        assert_eq!(path.to_string(), "/");
        
        let path = AddressPath::from_string("/a/b/c");
        assert_eq!(path.components(), &["a", "b", "c"]);
        assert_eq!(path.len(), 3);
        assert_eq!(path.to_string(), "/a/b/c");
        
        // Test joining
        let path2 = path.join("d");
        assert_eq!(path2.to_string(), "/a/b/c/d");
        
        // Test parent
        let parent = path2.parent().unwrap();
        assert_eq!(parent.to_string(), "/a/b/c");
        
        // Test last
        assert_eq!(path.last(), Some("c"));
        assert_eq!(path2.last(), Some("d"));
        
        // Empty path has no parent or last component
        let empty = AddressPath::new();
        assert!(empty.parent().is_none());
        assert!(empty.last().is_none());
    }
    
    #[test]
    fn test_hierarchical_address() {
        let addr = Address::<TestMessage>::new("test-actor", "test");
        let path = AddressPath::from_string("/system/user");
        
        let h_addr = HierarchicalAddress::new(addr.clone(), path);
        assert_eq!(h_addr.address(), &addr);
        assert_eq!(h_addr.path().to_string(), "/system/user");
        
        // Test child
        let child = h_addr.child("child");
        assert_eq!(child.path().to_string(), "/system/user/child");
        assert!(child.is_child_of(&h_addr));
        assert!(h_addr.is_parent_of(&child));
        
        // Test parent
        let parent = child.parent().unwrap();
        assert_eq!(parent, h_addr);
        
        // Test root hierarchy
        let root = helpers::to_hierarchical(addr);
        assert_eq!(root.path().to_string(), "/");
    }
    
    #[test]
    fn test_content_addressing() {
        let addr = Address::<TestMessage>::new("test-actor", "test");
        
        // Verify should succeed for a proper address
        assert!(addr.verify().unwrap());
        
        // Content hash should be deterministic
        let addr2 = Address::<TestMessage>::new("test-actor", "test");
        assert_eq!(addr.content_hash(), addr2.content_hash());
        
        // Hierarchical addressing
        let h_addr = HierarchicalAddress::new(addr.clone(), AddressPath::from_string("/test/path"));
        
        // Verify should also work for hierarchical addresses
        assert!(h_addr.verify());
        
        // Different paths should result in different hashes
        let h_addr2 = HierarchicalAddress::new(addr, AddressPath::from_string("/different/path"));
        assert_ne!(h_addr.content_hash(), h_addr2.content_hash());
    }
} 