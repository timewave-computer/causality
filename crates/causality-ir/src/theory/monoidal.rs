// Monoidal.rs - Defines the monoidal structure properties for the category theoretic model
// This represents the symmetric monoidal structure used for resource composition.

use std::fmt::Debug;

/// Trait defining a monoidal structure on a category
/// A monoidal category has a tensor product operation ⊗ and a unit object I,
/// satisfying associativity and unit laws.
pub trait Monoidal<A> {
    /// The unit object of the monoidal structure
    fn unit(&self) -> A;
    
    /// Tensor product operation (A ⊗ B)
    fn tensor(&self, a: &A, b: &A) -> A;
    
    /// Left unitor: I ⊗ A ≅ A
    fn left_unitor(&self, a: &A) -> A;
    
    /// Right unitor: A ⊗ I ≅ A
    fn right_unitor(&self, a: &A) -> A;
    
    /// Associator: (A ⊗ B) ⊗ C ≅ A ⊗ (B ⊗ C)
    fn associator(&self, a: &A, b: &A, c: &A) -> A;
    
    /// Verify the associativity of the tensor product
    /// (a ⊗ b) ⊗ c ≅ a ⊗ (b ⊗ c)
    fn verify_associativity(&self, a: &A, b: &A, c: &A) -> bool
    where
        A: PartialEq + Clone;
    
    /// Verify the left unit law: I ⊗ a ≅ a
    fn verify_left_unit(&self, a: &A) -> bool
    where
        A: PartialEq + Clone;
    
    /// Verify the right unit law: a ⊗ I ≅ a
    fn verify_right_unit(&self, a: &A) -> bool
    where
        A: PartialEq + Clone;
}

/// Trait defining a symmetric monoidal structure
/// A symmetric monoidal category has a braiding isomorphism that allows
/// "swapping" the order of objects in a tensor product.
pub trait SymmetricMonoidal<A>: Monoidal<A> {
    /// Braiding: A ⊗ B ≅ B ⊗ A
    fn braiding(&self, a: &A, b: &A) -> A;
    
    /// Verify the symmetry property: braiding(a, b) ≅ braiding(b, a)
    fn verify_symmetry(&self, a: &A, b: &A) -> bool
    where
        A: PartialEq + Clone;
    
    /// Verify the hexagon identity
    /// This ensures the braiding is compatible with associativity
    fn verify_hexagon_identity(&self, a: &A, b: &A, c: &A) -> bool
    where
        A: PartialEq + Clone;
}

/// Trait for a strong monoidal functor
/// A strong monoidal functor preserves the monoidal structure.
pub trait StrongMonoidalFunctor<A, B, M1, M2>
where
    M1: Monoidal<A>,
    M2: Monoidal<B>,
{
    /// Map an object from category A to category B
    fn map(&self, a: &A) -> B;
    
    /// Verify that F preserves the unit object: F(I_A) ≅ I_B
    fn verify_preserves_unit(&self, m1: &M1, m2: &M2) -> bool
    where
        A: Clone,
        B: PartialEq;
    
    /// Verify that F preserves the tensor product: F(a ⊗_A b) ≅ F(a) ⊗_B F(b)
    fn verify_preserves_tensor(&self, a: &A, b: &A, m1: &M1, m2: &M2) -> bool
    where
        A: Clone,
        B: PartialEq;
}

/// A presheaf over the category of resources
/// Maps resource configurations to the set of capabilities required to manipulate them
pub trait Presheaf<R, C> {
    /// Map a resource to its required capabilities
    fn capabilities(&self, resource: &R) -> C;
    
    /// Map a morphism in the resource category to a morphism in the capability category
    /// For f: r → r', this gives f*: Cap(r') → Cap(r)
    fn map_morphism<F>(&self, f: F, target: &R) -> C
    where
        F: Fn(R) -> R,
        R: Clone;
    
    /// Verify the functorial property of the presheaf
    /// For morphisms f: r → r' and g: r' → r'', it holds that (g ∘ f)* = f* ∘ g*
    fn verify_functorial_property<F, G>(&self, f: F, g: G, target: &R) -> bool
    where
        F: Fn(R) -> R,
        G: Fn(R) -> R,
        R: Clone,
        C: PartialEq;
}
