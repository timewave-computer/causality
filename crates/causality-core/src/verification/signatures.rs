// Signature verification functionality
//
// This module provides abstractions for signature verification.

use std::fmt::Debug;
use std::marker::PhantomData;

use causality_error::Error;

/// A generic signature type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signature<T> {
    /// The raw signature bytes
    bytes: Vec<u8>,
    
    /// Phantom data for the signed data type
    _phantom: PhantomData<T>,
}

impl<T> Signature<T> {
    /// Create a new signature from raw bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            _phantom: PhantomData,
        }
    }
    
    /// Get the raw signature bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    
    /// Convert the signature into raw bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
    
    /// Get the length of the signature in bytes
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    
    /// Check if the signature is empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// A trait for signers that can sign data
pub trait Signer<T> {
    /// The error type returned when signing fails
    type Error;
    
    /// The signature type produced by this signer
    type Signature;
    
    /// Sign the given data
    fn sign(&self, data: &T) -> Result<Self::Signature, Self::Error>;
}

/// A trait for verifiers that can verify signatures
pub trait Verifier<T> {
    /// The error type returned when verification fails
    type Error;
    
    /// The signature type verified by this verifier
    type Signature;
    
    /// Verify the signature for the given data
    fn verify(&self, data: &T, signature: &Self::Signature) -> Result<(), Self::Error>;
    
    /// Check if the signature is valid without returning details
    fn is_valid(&self, data: &T, signature: &Self::Signature) -> bool {
        self.verify(data, signature).is_ok()
    }
}

/// A trait for signed data that carries its own signature
pub trait Signed {
    /// The error type returned when verification fails
    type Error;
    
    /// The type of data that was signed
    type Data;
    
    /// The signature type
    type Signature;
    
    /// Get the data
    fn data(&self) -> &Self::Data;
    
    /// Get the signature
    fn signature(&self) -> &Self::Signature;
    
    /// Verify the signature using the provided verifier
    fn verify_with<V>(&self, verifier: &V) -> Result<(), Self::Error>
    where
        V: Verifier<Self::Data, Signature = Self::Signature, Error = Self::Error>,
    {
        verifier.verify(self.data(), self.signature())
    }
}

/// A generic signed data wrapper
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedData<T, S> {
    /// The data that was signed
    data: T,
    
    /// The signature
    signature: S,
}

impl<T, S> SignedData<T, S> {
    /// Create a new signed data wrapper
    pub fn new(data: T, signature: S) -> Self {
        Self { data, signature }
    }
    
    /// Get the data
    pub fn data(&self) -> &T {
        &self.data
    }
    
    /// Get the signature
    pub fn signature(&self) -> &S {
        &self.signature
    }
    
    /// Split into data and signature
    pub fn into_parts(self) -> (T, S) {
        (self.data, self.signature)
    }
}

impl<T, S> Signed for SignedData<T, S> {
    type Error = Error;
    type Data = T;
    type Signature = S;
    
    fn data(&self) -> &Self::Data {
        &self.data
    }
    
    fn signature(&self) -> &Self::Signature {
        &self.signature
    }
}

/// A generic signer implementation that uses a signing function
pub struct FnSigner<T, S, E, F> {
    /// The signing function
    sign_fn: F,
    
    /// Phantom data for the types
    _phantom: PhantomData<(T, S, E)>,
}

impl<T, S, E, F> FnSigner<T, S, E, F>
where
    F: Fn(&T) -> Result<S, E>,
{
    /// Create a new function-based signer
    pub fn new(sign_fn: F) -> Self {
        Self {
            sign_fn,
            _phantom: PhantomData,
        }
    }
}

impl<T, S, E, F> Signer<T> for FnSigner<T, S, E, F>
where
    F: Fn(&T) -> Result<S, E>,
{
    type Error = E;
    type Signature = S;
    
    fn sign(&self, data: &T) -> Result<Self::Signature, Self::Error> {
        (self.sign_fn)(data)
    }
}

/// A generic verifier implementation that uses a verification function
pub struct FnVerifier<T, S, E, F> {
    /// The verification function
    verify_fn: F,
    
    /// Phantom data for the types
    _phantom: PhantomData<(T, S, E)>,
}

impl<T, S, E, F> FnVerifier<T, S, E, F>
where
    F: Fn(&T, &S) -> Result<(), E>,
{
    /// Create a new function-based verifier
    pub fn new(verify_fn: F) -> Self {
        Self {
            verify_fn,
            _phantom: PhantomData,
        }
    }
}

impl<T, S, E, F> Verifier<T> for FnVerifier<T, S, E, F>
where
    F: Fn(&T, &S) -> Result<(), E>,
{
    type Error = E;
    type Signature = S;
    
    fn verify(&self, data: &T, signature: &Self::Signature) -> Result<(), Self::Error> {
        (self.verify_fn)(data, signature)
    }
}

/// Helper functions for signature verification
pub mod helpers {
    use super::*;
    
    /// Create a signer from a function
    pub fn signer_from_fn<T, S, E, F>(sign_fn: F) -> impl Signer<T, Error = E, Signature = S>
    where
        F: Fn(&T) -> Result<S, E>,
    {
        FnSigner::new(sign_fn)
    }
    
    /// Create a verifier from a function
    pub fn verifier_from_fn<T, S, E, F>(verify_fn: F) -> impl Verifier<T, Error = E, Signature = S>
    where
        F: Fn(&T, &S) -> Result<(), E>,
    {
        FnVerifier::new(verify_fn)
    }
    
    /// Sign data and create a signed data wrapper
    pub fn sign_data<T, S, E, V>(data: T, signer: &V) -> Result<SignedData<T, S>, E>
    where
        V: Signer<T, Error = E, Signature = S>,
        T: Clone,
    {
        let signature = signer.sign(&data)?;
        Ok(SignedData::new(data, signature))
    }
    
    /// Verify a collection of signed data items
    pub fn verify_all_signed<T, S, E, V, I>(signed_items: I, verifier: &V) -> Result<(), E>
    where
        V: Verifier<T, Error = E, Signature = S>,
        I: IntoIterator<Item = SignedData<T, S>>,
    {
        for item in signed_items {
            verifier.verify(item.data(), item.signature())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // A simple test signer that just stores the data bytes
    struct TestSigner;
    
    impl Signer<Vec<u8>> for TestSigner {
        type Error = Error;
        type Signature = Signature<Vec<u8>>;
        
        fn sign(&self, data: &Vec<u8>) -> Result<Self::Signature, Self::Error> {
            // Just create a copy of the data as the signature for testing
            Ok(Signature::new(data.clone()))
        }
    }
    
    // A simple test verifier that checks if the signature matches the data
    struct TestVerifier;
    
    impl Verifier<Vec<u8>> for TestVerifier {
        type Error = Error;
        type Signature = Signature<Vec<u8>>;
        
        fn verify(&self, data: &Vec<u8>, signature: &Self::Signature) -> Result<(), Self::Error> {
            if data == signature.as_bytes() {
                Ok(())
            } else {
                Err(Error::verification("Signature doesn't match data"))
            }
        }
    }
    
    #[test]
    fn test_signature() {
        let bytes = vec![1, 2, 3, 4];
        let signature = Signature::<Vec<u8>>::new(bytes.clone());
        
        assert_eq!(signature.as_bytes(), &bytes);
        assert_eq!(signature.len(), 4);
        assert!(!signature.is_empty());
        assert_eq!(signature.into_bytes(), bytes);
    }
    
    #[test]
    fn test_signer_and_verifier() {
        let signer = TestSigner;
        let verifier = TestVerifier;
        
        let data = vec![1, 2, 3, 4];
        let signature = signer.sign(&data).unwrap();
        
        assert!(verifier.verify(&data, &signature).is_ok());
        assert!(verifier.is_valid(&data, &signature));
        
        let wrong_data = vec![5, 6, 7, 8];
        assert!(verifier.verify(&wrong_data, &signature).is_err());
        assert!(!verifier.is_valid(&wrong_data, &signature));
    }
    
    #[test]
    fn test_signed_data() {
        let signer = TestSigner;
        let verifier = TestVerifier;
        
        let data = vec![1, 2, 3, 4];
        let signature = signer.sign(&data).unwrap();
        let signed_data = SignedData::new(data.clone(), signature);
        
        assert_eq!(signed_data.data(), &data);
        assert_eq!(signed_data.signature().as_bytes(), &data);
        
        assert!(signed_data.verify_with::<TestVerifier>(&verifier).is_ok());
    }
    
    #[test]
    fn test_helper_functions() {
        // Test signer_from_fn
        let signer = helpers::signer_from_fn(|data: &Vec<u8>| -> Result<Signature<Vec<u8>>, Error> {
            Ok(Signature::new(data.clone()))
        });
        
        // Test verifier_from_fn
        let verifier = helpers::verifier_from_fn(|data: &Vec<u8>, sig: &Signature<Vec<u8>>| -> Result<(), Error> {
            if data == sig.as_bytes() {
                Ok(())
            } else {
                Err(Error::verification("Signature doesn't match data"))
            }
        });
        
        let data = vec![1, 2, 3, 4];
        let signature = signer.sign(&data).unwrap();
        
        assert!(verifier.verify(&data, &signature).is_ok());
        
        // Test sign_data
        let signed_data = helpers::sign_data(data.clone(), &signer).unwrap();
        assert_eq!(signed_data.data(), &data);
        assert_eq!(signed_data.signature().as_bytes(), &data);
        
        // Test verify_all_signed
        let signed_items = vec![signed_data];
        assert!(helpers::verify_all_signed(signed_items, &verifier).is_ok());
    }
} 