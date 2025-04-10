// Adjunction.rs - Defines the adjunction properties for the category theoretic model
// This represents the adjunction between the TEL and TEG categories.

use super::functor::{Functor, TelObject, TegObject};
use std::fmt::Debug;

/// Trait defining a natural transformation between functors
pub trait NaturalTransformation<A, B, F1, F2>
where
    F1: Functor<A, B>,
    F2: Functor<A, B>,
{
    /// Apply the natural transformation at object a
    fn transform(&self, a: &A) -> B;
    
    /// Verify the naturality condition
    /// For any morphism f: a -> a', the following diagram commutes:
    /// F(a) --η_a--> G(a)
    ///  |              |
    /// F(f)           G(f)
    ///  |              |
    ///  v              v
    /// F(a') --η_a'--> G(a')
    fn verify_naturality<F>(&self, a: &A, f: F) -> bool
    where
        F: Fn(A) -> A,
        A: Clone,
        B: PartialEq;
}

/// Unit natural transformation for an adjunction F ⊣ G
/// Unit: 1_A → G∘F, where 1_A is the identity functor on category A
pub trait UnitTransformation<A, B, F, G>
where
    F: Functor<A, B>,
    G: Functor<B, A>,
    A: Clone,
{
    /// Apply the unit transformation at object a
    /// η_a: a → G(F(a))
    fn unit(&self, a: &A) -> A;
    
    /// Verify the naturality condition for the unit
    fn verify_unit_naturality<M>(&self, a: &A, f: M) -> bool
    where
        M: Fn(A) -> A,
        A: PartialEq;
}

/// Counit natural transformation for an adjunction F ⊣ G
/// Counit: F∘G → 1_B, where 1_B is the identity functor on category B
pub trait CounitTransformation<A, B, F, G>
where
    F: Functor<A, B>,
    G: Functor<B, A>,
    B: Clone,
{
    /// Apply the counit transformation at object b
    /// ε_b: F(G(b)) → b
    fn counit(&self, b: &B) -> B;
    
    /// Verify the naturality condition for the counit
    fn verify_counit_naturality<M>(&self, b: &B, f: M) -> bool
    where
        M: Fn(B) -> B,
        B: PartialEq;
}

/// Trait defining the adjunction property
/// An adjunction F ⊣ G means:
/// - F: A → B is the left adjoint functor
/// - G: B → A is the right adjoint functor
/// - There is a natural isomorphism: Hom_B(F(a), b) ≅ Hom_A(a, G(b))
pub trait Adjunction<A, B>
where
    A: TelObject,
    B: TegObject,
{
    /// The left adjoint functor (F: TEL → TEG)
    type LeftAdjoint: Functor<A, B>;
    
    /// The right adjoint functor (G: TEG → TEL)
    type RightAdjoint: Functor<B, A>;
    
    /// The unit natural transformation (η: 1_A → G∘F)
    type Unit: UnitTransformation<A, B, Self::LeftAdjoint, Self::RightAdjoint>;
    
    /// The counit natural transformation (ε: F∘G → 1_B)
    type Counit: CounitTransformation<A, B, Self::LeftAdjoint, Self::RightAdjoint>;
    
    /// Get the left adjoint functor
    fn left_adjoint(&self) -> &Self::LeftAdjoint;
    
    /// Get the right adjoint functor
    fn right_adjoint(&self) -> &Self::RightAdjoint;
    
    /// Get the unit natural transformation
    fn unit(&self) -> &Self::Unit;
    
    /// Get the counit natural transformation
    fn counit(&self) -> &Self::Counit;
    
    /// Verify the triangle identity: (ε_F(a) ∘ F(η_a)) = id_F(a)
    fn verify_triangle_identity_1(&self, a: &A) -> bool
    where
        A: Clone,
        B: Clone + PartialEq;
    
    /// Verify the triangle identity: (G(ε_b) ∘ η_G(b)) = id_G(b)
    fn verify_triangle_identity_2(&self, b: &B) -> bool
    where
        A: Clone + PartialEq,
        B: Clone;
    
    /// Verify the natural isomorphism: Hom_B(F(a), b) ≅ Hom_A(a, G(b))
    fn verify_natural_isomorphism(&self, a: &A, b: &B) -> bool
    where
        A: Clone + PartialEq,
        B: Clone + PartialEq;
}
