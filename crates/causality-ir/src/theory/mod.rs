// Theory module for the category theoretic foundation of the TEG
// This module contains the traits and implementations for functors, adjunctions,
// and monoidal structures used in the TEG.

pub mod functor;
pub mod adjunction;
pub mod monoidal;

pub use functor::{Functor, TelObject, TegObject, TelMorphism, TegMorphism};
pub use adjunction::{Adjunction, NaturalTransformation, UnitTransformation, CounitTransformation};
pub use monoidal::{Monoidal, SymmetricMonoidal, StrongMonoidalFunctor, Presheaf};
