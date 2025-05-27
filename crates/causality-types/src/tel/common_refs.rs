// Purpose: Defines common reference types for the Temporal Effect Language (TEL).

use crate::primitive::ids::ResourceId;
use crate::serialization::{Decode, DecodeError, Encode, SimpleSerialize};

/// A reference to a resource, typically by its ID.
///
/// This newtype wrapper provides a distinct type for resource references within
/// the TEL graph structures, improving type safety.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub struct ResourceRef(pub ResourceId);

impl Encode for ResourceRef {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}

impl Decode for ResourceRef {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(ResourceRef(ResourceId::from_ssz_bytes(bytes)?))
    }
}

impl SimpleSerialize for ResourceRef {}

impl ResourceRef {
    /// Creates a new ResourceRef from a ResourceId.
    pub fn new(id: ResourceId) -> Self {
        Self(id)
    }

    /// Gets the underlying ResourceId.
    pub fn id(&self) -> ResourceId {
        self.0
    }
}

impl From<ResourceId> for ResourceRef {
    fn from(id: ResourceId) -> Self {
        ResourceRef(id)
    }
}

impl AsRef<ResourceId> for ResourceRef {
    fn as_ref(&self) -> &ResourceId {
        &self.0
    }
}

//-----------------------------------------------------------------------------
// S-expression Serialization Support (for DSL output)
//-----------------------------------------------------------------------------

#[cfg(feature = "sexpr")]
use crate::expr::sexpr::{ToSexpr, FromSexpr, tagged_sexpr, validate_tag, extract_field};
#[cfg(feature = "sexpr")]
use crate::primitive::ids::AsId;
#[cfg(feature = "sexpr")]
use anyhow::{anyhow, Result};
#[cfg(feature = "sexpr")]
use lexpr::Value as SexprValue;

#[cfg(feature = "sexpr")]
impl ToSexpr for ResourceRef {
    fn to_sexpr(&self) -> SexprValue {
        tagged_sexpr("resource-ref", vec![
            SexprValue::string(&*self.0.to_hex())
        ])
    }
}

#[cfg(feature = "sexpr")]
impl FromSexpr for ResourceRef {
    fn from_sexpr(sexpr: &SexprValue) -> Result<Self> {
        validate_tag(sexpr, "resource-ref")?;
        
        let elements = crate::expr::sexpr::get_list_elements(sexpr)
            .ok_or_else(|| anyhow!("ResourceRef S-expression must be a list"))?;
            
        if elements.len() != 2 {
            return Err(anyhow!("ResourceRef S-expression must have exactly one resource ID"));
        }
        
        let id_hex = crate::expr::sexpr::get_string_value(&elements[1])
            .ok_or_else(|| anyhow!("Resource ID must be a string"))?;
        let id = ResourceId::from_hex(id_hex).map_err(|_| anyhow!("Invalid resource ID hex"))?;
        
        Ok(ResourceRef(id))
    }
}
