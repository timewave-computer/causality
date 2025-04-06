// Verification extensions for compatibility with the execution module
//
// This file provides adapter implementations between the verification and execution modules
// to ensure compatibility between their different interfaces.

use crate::operation::execution::VerificationResultExt;
use crate::operation::verification::VerificationResult;

// Implement VerificationResultExt for the VerificationResult from the verification module
impl VerificationResultExt for VerificationResult {
    fn is_valid(&self) -> bool {
        self.valid
    }
    
    fn reasons(&self) -> Vec<String> {
        self.reasons.clone()
    }
} 