//! Unified Type System FFI bindings for OCaml
//!
//! This module provides FFI bindings for the unified type system that seamlessly 
//! integrates structured types, session types, and location awareness.

use ocaml::{FromValue, ToValue};
#[cfg(feature = "ocaml-ffi")]
use ocaml_derive::{FromValue as DeriveFromValue, ToValue as DeriveToValue};

use causality_core::{Location, SessionType as CoreSessionType, TypeInner,
        system::content_addressing::EntityId};

/// OCaml-compatible location type
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, PartialEq, Eq, DeriveFromValue, DeriveToValue)]
pub enum OcamlLocation {
    Local,
    Remote(String),
    Domain(String),
    Any,
}

#[cfg(feature = "ocaml-ffi")]
impl From<Location> for OcamlLocation {
    fn from(loc: Location) -> Self {
        match loc {
            Location::Local => OcamlLocation::Local,
            Location::Remote(s) => OcamlLocation::Remote(s.to_hex()),
            Location::Domain(s) => OcamlLocation::Domain(s.clone()),
            Location::Any => OcamlLocation::Any,
            // Map other variants to reasonable defaults
            Location::Distributed(_) => OcamlLocation::Any,
            Location::Edge(s) => OcamlLocation::Domain(s),
            Location::Cloud(s) => OcamlLocation::Domain(s),
            Location::Composite(_) => OcamlLocation::Any,
            Location::Variable(_) => OcamlLocation::Any,
            Location::None => OcamlLocation::Local,
        }
    }
}

#[cfg(feature = "ocaml-ffi")]
impl From<OcamlLocation> for Location {
    fn from(loc: OcamlLocation) -> Self {
        match loc {
            OcamlLocation::Local => Location::Local,
            OcamlLocation::Remote(s) => Location::Remote(EntityId::from_hex(&s).unwrap_or(EntityId::ZERO)),
            OcamlLocation::Domain(s) => Location::Domain(s),
            OcamlLocation::Any => Location::Any,
        }
    }
}

/// OCaml-compatible session type
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, PartialEq, Eq, DeriveFromValue, DeriveToValue)]
pub enum OcamlSessionType {
    Send { ty: Box<OcamlTypeInner>, cont: Box<OcamlSessionType> },
    Receive { ty: Box<OcamlTypeInner>, cont: Box<OcamlSessionType> },
    InternalChoice(Vec<(String, OcamlSessionType)>),
    ExternalChoice(Vec<(String, OcamlSessionType)>),
    End,
    Recursive(String, Box<OcamlSessionType>),
    Variable(String),
}

/// OCaml-compatible type inner
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, PartialEq, Eq, DeriveFromValue, DeriveToValue)]
pub enum OcamlTypeInner {
    Base(OcamlBaseType),
    Product(Box<OcamlTypeInner>, Box<OcamlTypeInner>),
    Sum(Box<OcamlTypeInner>, Box<OcamlTypeInner>),
    LinearFunction(Box<OcamlTypeInner>, Box<OcamlTypeInner>),
    Session(OcamlSessionType),
    Transform {
        input: Box<OcamlTypeInner>,
        output: Box<OcamlTypeInner>,
        location: OcamlLocation,
    },
    Located(Box<OcamlTypeInner>, OcamlLocation),
}

/// OCaml-compatible base type
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, PartialEq, Eq, DeriveFromValue, DeriveToValue)]
pub enum OcamlBaseType {
    Unit,
    Bool,
    Int,
    Symbol,
}

/// Location operations
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn location_local() -> OcamlLocation {
    OcamlLocation::Local
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn location_remote(server: String) -> OcamlLocation {
    OcamlLocation::Remote(server)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn location_domain(domain: String) -> OcamlLocation {
    OcamlLocation::Domain(domain)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn location_any() -> OcamlLocation {
    OcamlLocation::Any
}

/// Type constructors
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn type_base_unit() -> OcamlTypeInner {
    OcamlTypeInner::Base(OcamlBaseType::Unit)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn type_base_bool() -> OcamlTypeInner {
    OcamlTypeInner::Base(OcamlBaseType::Bool)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn type_base_int() -> OcamlTypeInner {
    OcamlTypeInner::Base(OcamlBaseType::Int)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn type_base_symbol() -> OcamlTypeInner {
    OcamlTypeInner::Base(OcamlBaseType::Symbol)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn type_product(left: OcamlTypeInner, right: OcamlTypeInner) -> OcamlTypeInner {
    OcamlTypeInner::Product(Box::new(left), Box::new(right))
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn type_sum(left: OcamlTypeInner, right: OcamlTypeInner) -> OcamlTypeInner {
    OcamlTypeInner::Sum(Box::new(left), Box::new(right))
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn type_linear_function(input: OcamlTypeInner, output: OcamlTypeInner) -> OcamlTypeInner {
    OcamlTypeInner::LinearFunction(Box::new(input), Box::new(output))
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn type_transform(input: OcamlTypeInner, output: OcamlTypeInner, location: OcamlLocation) -> OcamlTypeInner {
    OcamlTypeInner::Transform {
        input: Box::new(input),
        output: Box::new(output),
        location,
    }
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn type_located(ty: OcamlTypeInner, location: OcamlLocation) -> OcamlTypeInner {
    OcamlTypeInner::Located(Box::new(ty), location)
}

/// Session type constructors
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn session_send(ty: OcamlTypeInner, cont: OcamlSessionType) -> OcamlSessionType {
    OcamlSessionType::Send {
        ty: Box::new(ty),
        cont: Box::new(cont),
    }
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn session_receive(ty: OcamlTypeInner, cont: OcamlSessionType) -> OcamlSessionType {
    OcamlSessionType::Receive {
        ty: Box::new(ty),
        cont: Box::new(cont),
    }
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn session_end() -> OcamlSessionType {
    OcamlSessionType::End
}

/// Check if location is local
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn location_is_local(loc: OcamlLocation) -> bool {
    matches!(loc, OcamlLocation::Local)
}

/// Check if location is remote
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn location_is_remote(loc: OcamlLocation) -> bool {
    matches!(loc, OcamlLocation::Remote(_))
} 