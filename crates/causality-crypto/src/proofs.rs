// Proof verification functionality
//
// This module provides abstractions for generating and verifying proofs.

use std::fmt::Debug;
use std::marker::PhantomData;

/// A trait for proofs that can be verified
pub trait Proof: Clone + Debug {
    /// The error type returned when verification fails
    type Error;
    
    /// The type of public input used to verify this proof
    type PublicInput;
    
    /// Verify this proof against the given public input
    fn verify(&self, public_input: &Self::PublicInput) -> Result<(), Self::Error>;
    
    /// Check if this proof is valid without returning details
    fn is_valid(&self, public_input: &Self::PublicInput) -> bool {
        self.verify(public_input).is_ok()
    }
}

/// A trait for proof generators
pub trait Prover<T> {
    /// The error type returned when proof generation fails
    type Error;
    
    /// The type of proof produced by this prover
    type Proof;
    
    /// The type of public input that will be used to verify the proof
    type PublicInput;
    
    /// Generate a proof for the given data
    fn prove(&self, data: &T) -> Result<Self::Proof, Self::Error>;
    
    /// Generate a proof for the given data with the given public input
    fn prove_with(&self, data: &T, public_input: &Self::PublicInput) -> Result<Self::Proof, Self::Error>;
    
    /// Extract the public input from the given data
    fn extract_public_input(&self, data: &T) -> Self::PublicInput;
}

/// A trait for proof verifiers
pub trait ProofVerifier<P> {
    /// The error type returned when verification fails
    type Error;
    
    /// The type of public input used to verify proofs
    type PublicInput;
    
    /// Verify a proof against the given public input
    fn verify(&self, proof: &P, public_input: &Self::PublicInput) -> Result<(), Self::Error>;
    
    /// Check if a proof is valid without returning details
    fn is_valid(&self, proof: &P, public_input: &Self::PublicInput) -> bool {
        self.verify(proof, public_input).is_ok()
    }
}

/// A generic proof container
#[derive(Debug, Clone)]
pub struct GenericProof<T> {
    /// The proof data
    data: Vec<u8>,
    
    /// Phantom data for the type of data this proof is for
    _phantom: PhantomData<T>,
}

impl<T> GenericProof<T> {
    /// Create a new proof from raw data
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            _phantom: PhantomData,
        }
    }
    
    /// Get the raw proof data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// Convert the proof into raw data
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
    
    /// Get the length of the proof in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if the proof is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// A generic prover that uses a proving function
pub struct FnProver<T, P, I, E, F1, F2> {
    /// The proving function
    prove_fn: F1,
    
    /// The function to extract public input
    extract_fn: F2,
    
    /// Phantom data for the types
    _phantom: PhantomData<(T, P, I, E)>,
}

impl<T, P, I, E, F1, F2> FnProver<T, P, I, E, F1, F2>
where
    F1: Fn(&T, &I) -> Result<P, E>,
    F2: Fn(&T) -> I,
{
    /// Create a new function-based prover
    pub fn new(prove_fn: F1, extract_fn: F2) -> Self {
        Self {
            prove_fn,
            extract_fn,
            _phantom: PhantomData,
        }
    }
}

impl<T, P, I, E, F1, F2> Prover<T> for FnProver<T, P, I, E, F1, F2>
where
    F1: Fn(&T, &I) -> Result<P, E>,
    F2: Fn(&T) -> I,
{
    type Error = E;
    type Proof = P;
    type PublicInput = I;
    
    fn prove(&self, data: &T) -> Result<Self::Proof, Self::Error> {
        let public_input = self.extract_public_input(data);
        self.prove_with(data, &public_input)
    }
    
    fn prove_with(&self, data: &T, public_input: &Self::PublicInput) -> Result<Self::Proof, Self::Error> {
        (self.prove_fn)(data, public_input)
    }
    
    fn extract_public_input(&self, data: &T) -> Self::PublicInput {
        (self.extract_fn)(data)
    }
}

/// A generic verifier that uses a verification function
pub struct FnVerifier<P, I, E, F> {
    /// The verification function
    verify_fn: F,
    
    /// Phantom data for the types
    _phantom: PhantomData<(P, I, E)>,
}

impl<P, I, E, F> FnVerifier<P, I, E, F>
where
    F: Fn(&P, &I) -> Result<(), E>,
{
    /// Create a new function-based verifier
    pub fn new(verify_fn: F) -> Self {
        Self {
            verify_fn,
            _phantom: PhantomData,
        }
    }
}

impl<P, I, E, F> ProofVerifier<P> for FnVerifier<P, I, E, F>
where
    F: Fn(&P, &I) -> Result<(), E>,
{
    type Error = E;
    type PublicInput = I;
    
    fn verify(&self, proof: &P, public_input: &Self::PublicInput) -> Result<(), Self::Error> {
        (self.verify_fn)(proof, public_input)
    }
}

/// A data structure that contains a proof and the public input
#[derive(Debug, Clone)]
pub struct ProofWithInput<P, I> {
    /// The proof
    proof: P,
    
    /// The public input
    public_input: I,
}

impl<P, I> ProofWithInput<P, I> {
    /// Create a new proof with input
    pub fn new(proof: P, public_input: I) -> Self {
        Self {
            proof,
            public_input,
        }
    }
    
    /// Get the proof
    pub fn proof(&self) -> &P {
        &self.proof
    }
    
    /// Get the public input
    pub fn public_input(&self) -> &I {
        &self.public_input
    }
    
    /// Split into proof and public input
    pub fn into_parts(self) -> (P, I) {
        (self.proof, self.public_input)
    }
}

impl<P, I, E> ProofWithInput<P, I>
where
    P: Proof<Error = E, PublicInput = I>,
{
    /// Verify this proof with its own public input
    pub fn verify(&self) -> Result<(), E> {
        self.proof.verify(&self.public_input)
    }
    
    /// Check if this proof is valid with its own public input
    pub fn is_valid(&self) -> bool {
        self.proof.is_valid(&self.public_input)
    }
}

/// Helper functions for proof generation and verification
pub mod helpers {
    use super::*;
    
    /// Create a prover from a function
    pub fn prover_from_fn<T, P, I, E, F1, F2>(
        prove_fn: F1,
        extract_fn: F2,
    ) -> impl Prover<T, Error = E, Proof = P, PublicInput = I>
    where
        F1: Fn(&T, &I) -> Result<P, E>,
        F2: Fn(&T) -> I,
    {
        FnProver::new(prove_fn, extract_fn)
    }
    
    /// Create a verifier from a function
    pub fn verifier_from_fn<P, I, E, F>(
        verify_fn: F,
    ) -> impl ProofVerifier<P, Error = E, PublicInput = I>
    where
        F: Fn(&P, &I) -> Result<(), E>,
    {
        FnVerifier::new(verify_fn)
    }
    
    /// Generate a proof with its public input
    pub fn prove_with_input<T, P, I, E, V>(
        data: &T,
        prover: &V,
    ) -> Result<ProofWithInput<P, I>, E>
    where
        V: Prover<T, Error = E, Proof = P, PublicInput = I>,
    {
        let public_input = prover.extract_public_input(data);
        let proof = prover.prove_with(data, &public_input)?;
        Ok(ProofWithInput::new(proof, public_input))
    }
    
    /// Verify a collection of proofs with their public inputs
    pub fn verify_all<P, I, E, V, IT>(
        proofs_with_inputs: IT,
        verifier: &V,
    ) -> Result<(), E>
    where
        V: ProofVerifier<P, Error = E, PublicInput = I>,
        IT: IntoIterator<Item = ProofWithInput<P, I>>,
    {
        for proof_with_input in proofs_with_inputs {
            verifier.verify(proof_with_input.proof(), proof_with_input.public_input())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;
    use std::fmt;
    
    // A simple error type for proof verification
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ProofError(String);
    
    impl fmt::Display for ProofError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Proof error: {}", self.0)
        }
    }
    
    impl StdError for ProofError {}
    
    // A simple test proof
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestProof(Vec<u8>);
    
    impl Proof for TestProof {
        type Error = ProofError;
        type PublicInput = String;
        
        fn verify(&self, public_input: &Self::PublicInput) -> Result<(), Self::Error> {
            // For this test, the proof is valid if it contains the public input
            let data = String::from_utf8_lossy(&self.0);
            if data.contains(public_input) {
                Ok(())
            } else {
                Err(ProofError("Proof doesn't contain public input".to_string()))
            }
        }
    }
    
    // A simple test prover
    struct TestProver;
    
    impl Prover<String> for TestProver {
        type Error = ProofError;
        type Proof = TestProof;
        type PublicInput = String;
        
        fn prove(&self, data: &String) -> Result<Self::Proof, Self::Error> {
            let public_input = self.extract_public_input(data);
            self.prove_with(data, &public_input)
        }
        
        fn prove_with(&self, data: &String, public_input: &Self::PublicInput) -> Result<Self::Proof, Self::Error> {
            // For this test, the proof is the data with the public input appended
            let mut proof_data = data.clone();
            proof_data.push_str(" with ");
            proof_data.push_str(public_input);
            Ok(TestProof(proof_data.into_bytes()))
        }
        
        fn extract_public_input(&self, data: &String) -> Self::PublicInput {
            // For this test, the public input is the first word of the data
            data.split_whitespace()
                .next()
                .unwrap_or("")
                .to_string()
        }
    }
    
    // A simple test verifier
    struct TestVerifier;
    
    impl ProofVerifier<TestProof> for TestVerifier {
        type Error = ProofError;
        type PublicInput = String;
        
        fn verify(&self, proof: &TestProof, public_input: &Self::PublicInput) -> Result<(), Self::Error> {
            proof.verify(public_input)
        }
    }
    
    #[test]
    fn test_generic_proof() {
        let data = vec![1, 2, 3, 4];
        let proof = GenericProof::<String>::new(data.clone());
        
        assert_eq!(proof.as_bytes(), &data);
        assert_eq!(proof.len(), 4);
        assert!(!proof.is_empty());
        assert_eq!(proof.into_bytes(), data);
    }
    
    #[test]
    fn test_proof() {
        let proof = TestProof("hello world".to_string().into_bytes());
        
        assert!(proof.verify(&"hello".to_string()).is_ok());
        assert!(proof.verify(&"world".to_string()).is_ok());
        assert!(proof.verify(&"goodbye".to_string()).is_err());
    }
    
    #[test]
    fn test_prover_and_verifier() {
        let prover = TestProver;
        let verifier = TestVerifier;
        
        let data = "hello world".to_string();
        let proof = prover.prove(&data).unwrap();
        
        // The public input extracted should be "hello"
        let public_input = "hello".to_string();
        
        assert!(verifier.verify(&proof, &public_input).is_ok());
        assert!(verifier.is_valid(&proof, &public_input));
        
        // Test with a different public input that wasn't extracted
        let wrong_input = "goodbye".to_string();
        assert!(verifier.verify(&proof, &wrong_input).is_err());
        assert!(!verifier.is_valid(&proof, &wrong_input));
    }
    
    #[test]
    fn test_proof_with_input() {
        let prover = TestProver;
        
        let data = "hello world".to_string();
        let proof_with_input = helpers::prove_with_input(&data, &prover).unwrap();
        
        assert_eq!(proof_with_input.public_input(), &"hello".to_string());
        assert!(proof_with_input.verify().is_ok());
        assert!(proof_with_input.is_valid());
    }
    
    #[test]
    fn test_helper_functions() {
        // Test prover_from_fn
        let prover = helpers::prover_from_fn(
            |data: &String, public_input: &String| -> Result<TestProof, ProofError> {
                let mut proof_data = data.clone();
                proof_data.push_str(" with ");
                proof_data.push_str(public_input);
                Ok(TestProof(proof_data.into_bytes()))
            },
            |data: &String| -> String {
                data.split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string()
            },
        );
        
        // Test verifier_from_fn
        let verifier = helpers::verifier_from_fn(
            |proof: &TestProof, public_input: &String| -> Result<(), ProofError> {
                proof.verify(public_input)
            },
        );
        
        let data = "hello world".to_string();
        let proof = prover.prove(&data).unwrap();
        let public_input = "hello".to_string();
        
        assert!(verifier.verify(&proof, &public_input).is_ok());
        
        // Test prove_with_input
        let proof_with_input = helpers::prove_with_input(&data, &prover).unwrap();
        assert!(proof_with_input.verify().is_ok());
        
        // Test verify_all
        let proofs = vec![proof_with_input];
        assert!(helpers::verify_all(proofs, &verifier).is_ok());
    }
} 