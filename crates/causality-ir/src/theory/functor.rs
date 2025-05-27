// Functor.rs - Defines the functorial properties for the category theoretic model
// This trait represents a functor between two categories, which preserves 
// identity morphisms and composition.

use std::fmt::Debug;

/// Trait defining the functorial properties between categories A and B
pub trait Functor<A, B> {
    /// Apply the functor to an object of category A to produce an object of category B
    fn map(&self, a: A) -> B;
    
    /// Verify that the functor preserves identity morphisms
    /// For any object a in A, F(id_A(a)) = id_B(F(a))
    fn preserves_identity(&self, a: &A) -> bool
    where
        A: Clone,
        B: PartialEq;
    
    /// Verify that the functor preserves composition
    /// For any composable morphisms f: A -> A and g: A -> A,
    /// F(g ∘ f) = F(g) ∘ F(f)
    fn preserves_composition<F, G>(&self, f: F, g: G, a: &A) -> bool
    where
        F: Fn(A) -> A,
        G: Fn(A) -> A,
        A: Clone,
        B: PartialEq;
}

/// Marker trait for objects in the TEL category
pub trait TelObject: Clone + Debug + PartialEq {}

/// Marker trait for objects in the TEG category
pub trait TegObject: Clone + Debug + PartialEq {}

/// Trait for morphisms in the TEL category
pub trait TelMorphism<A: TelObject>: Send + Sync {
    fn apply(&self, a: A) -> A;
    
    /// Compose this morphism with another (g ∘ f)
    fn compose<'a>(&'a self, f: &'a dyn TelMorphism<A>) -> Box<dyn TelMorphism<A> + 'a>;
    
    /// Create an identity morphism for the given object type
    fn identity(&self, a: &A) -> Box<dyn TelMorphism<A>>;
}

/// Trait for morphisms in the TEG category
pub trait TegMorphism<B: TegObject>: Send + Sync {
    fn apply(&self, b: B) -> B;
    
    /// Compose this morphism with another (g ∘ f)
    fn compose<'a>(&'a self, f: &'a dyn TegMorphism<B>) -> Box<dyn TegMorphism<B> + 'a>;
    
    /// Get the identity morphism for an object
    fn identity() -> Box<dyn TegMorphism<B>> where Self: Sized;
}
