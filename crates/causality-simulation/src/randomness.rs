//! Deterministic Randomness
//!
//! Provides deterministic random number generation for simulations,
//! ensuring reproducible execution with the same seed.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use rand::prelude::{RngCore, SeedableRng, StdRng};
use rand::Error as RandError;
use rand::Rng; // Required for Rng::gen for from_entropy to generate a seed

/// A wrapper around a seeded Pseudo-Random Number Generator (PRNG)
/// to ensure deterministic randomness in simulations.
#[derive(Debug, Clone)] // Clone is useful for snapshotting if the RNG state needs to be part of it.
pub struct SeededRng {
    rng: StdRng,
    seed: u64,
}

impl SeededRng {
    /// Creates a new RNG instance seeded with the given 64-bit seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            seed,
        }
    }

    /// Creates a new RNG instance from entropy.
    /// A seed is generated from entropy, stored, and used to initialize the RNG.
    /// This ensures that the simulation is replayable if its state (including this seed) is saved.
    pub fn from_entropy() -> Self {
        let mut entropy_rng = StdRng::from_entropy();
        let seed = entropy_rng.next_u64(); // Generate a seed from entropy
        Self {
            rng: StdRng::seed_from_u64(seed),
            seed,
        }
    }

    /// Returns the seed used to initialize this RNG.
    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    /// Generate a random boolean value
    pub fn gen_bool(&mut self) -> bool {
        self.rng.gen()
    }

    /// Generate a random value in the given range
    pub fn gen_range<T, R>(&mut self, range: R) -> T
    where
        T: rand::distributions::uniform::SampleUniform,
        R: rand::distributions::uniform::SampleRange<T>,
    {
        self.rng.gen_range(range)
    }

    /// Generate a random UUID v4
    pub fn gen_uuid(&mut self) -> String {
        let mut bytes = [0u8; 16];
        self.fill_bytes(&mut bytes);
        
        // Set version (4) and variant bits according to RFC 4122
        bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
        bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 10
        
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    }

    /// Sample a random element from a slice
    pub fn sample<T>(&mut self, slice: &[T]) -> Option<&T> {
        if slice.is_empty() {
            None
        } else {
            let index = self.gen_range(0..slice.len());
            slice.get(index)
        }
    }

    /// Shuffle a mutable slice in place using Fisher-Yates algorithm
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.gen_range(0..=i);
            slice.swap(i, j);
        }
    }

    /// Choose a random element based on weights
    pub fn weighted_choice<T>(&mut self, items: &[(T, f64)]) -> Option<&T>
    where
        T: Clone,
    {
        if items.is_empty() {
            return None;
        }

        let total_weight: f64 = items.iter().map(|(_, weight)| weight).sum();
        if total_weight <= 0.0 {
            return None;
        }

        let mut random_weight = self.rng.gen::<f64>() * total_weight;
        
        for (item, weight) in items {
            random_weight -= weight;
            if random_weight <= 0.0 {
                return Some(item);
            }
        }

        // Fallback to last item (shouldn't happen with proper weights)
        items.last().map(|(item, _)| item)
    }
}

// Implement RngCore so that SeededRng can be used wherever RngCore is expected.
impl RngCore for SeededRng {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RandError> {
        self.rng.try_fill_bytes(dest)
    }
}

// Note: StdRng does not implement CryptoRng, so SeededRng won't either unless we wrap a CryptoRng.
// For simulation purposes, CryptoRng is typically not a requirement.

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------



#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng; // Import Rng trait for convenience methods like gen_range

    #[test]
    fn test_seeded_rng_deterministic() {
        let mut rng1 = SeededRng::new(12345);
        let mut rng2 = SeededRng::new(12345);

        let val1_rng1 = rng1.next_u32();
        let val1_rng2 = rng2.next_u32();
        assert_eq!(val1_rng1, val1_rng2, "First generated value should be the same for the same seed.");

        let val2_rng1 = rng1.gen_range(0..100);
        let val2_rng2 = rng2.gen_range(0..100);
        assert_eq!(val2_rng1, val2_rng2, "Second generated value (range) should be the same.");
    }

    #[test]
    fn test_seeded_rng_different_seeds() {
        let mut rng1 = SeededRng::new(12345);
        let mut rng2 = SeededRng::new(54321);

        // It's highly improbable they will be the same, but not strictly guaranteed for a single draw.
        // A sequence of draws would be better for a stronger test.
        let val_rng1 = rng1.next_u64();
        let val_rng2 = rng2.next_u64();
        assert_ne!(val_rng1, val_rng2, "Values from different seeds should typically differ.");
    }

    #[test]
    fn test_fill_bytes() {
        let mut rng1 = SeededRng::new(67890);
        let mut rng2 = SeededRng::new(67890);

        let mut bytes1 = [0u8; 16];
        let mut bytes2 = [0u8; 16];

        rng1.fill_bytes(&mut bytes1);
        rng2.fill_bytes(&mut bytes2);

        assert_eq!(bytes1, bytes2, "fill_bytes should produce the same sequence for the same seed.");
    }
    #[test]
    fn test_from_entropy() {
        // This test mainly checks that it doesn't panic.
        // We can't assert determinism here across test runs easily.
        let mut rng = SeededRng::from_entropy();
        let _val = rng.next_u32(); // Generate a value
        assert!(true); // If we reached here, it didn't panic.
    }
} 