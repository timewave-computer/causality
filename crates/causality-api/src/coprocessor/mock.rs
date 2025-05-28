// Purpose: Defines a mock implementation for the ZkCoprocessorApi trait.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::coprocessor::traits::ZkCoprocessorApi;
use crate::coprocessor::types::{
    CoprocessorApiError, Proof, ProofRequestId, ProofStatus, PublicInputs,
};
use causality_types::effect::trace::ExecutionTrace;

/// Defines the behavior of a mocked ZK Coprocessor operation.
#[derive(Clone)] // Clone needed if behaviors are set by value and stored.
pub enum MethodMockBehavior {
    SubmitTraceReturns(Result<ProofRequestId, CoprocessorApiError>),
    CheckStatusReturns(Result<ProofStatus, CoprocessorApiError>),
    GetProofReturns(Result<Proof, CoprocessorApiError>),
    VerifyProofReturns(Result<bool, CoprocessorApiError>),
    Panic(String),
}

/// Mock implementation of the `ZkCoprocessorApi` trait for testing purposes.
#[derive(Clone, Default)]
pub struct MockZkCoprocessor {
    submit_trace_behavior: Arc<Mutex<Option<MethodMockBehavior>>>,
    check_status_behaviors: Arc<Mutex<HashMap<ProofRequestId, MethodMockBehavior>>>,
    get_proof_behaviors: Arc<Mutex<HashMap<ProofRequestId, MethodMockBehavior>>>,
    verify_proof_behavior: Arc<Mutex<Option<MethodMockBehavior>>>,

    // Default behavior if no specific mock is set for a call or job ID
    default_submit_trace_behavior: Arc<Mutex<Option<MethodMockBehavior>>>,
    default_check_status_behavior: Arc<Mutex<Option<MethodMockBehavior>>>,
    default_get_proof_behavior: Arc<Mutex<Option<MethodMockBehavior>>>,
    default_verify_proof_behavior: Arc<Mutex<Option<MethodMockBehavior>>>,

    // Store for proofs to allow simulating `submit` then `get_proof` flow.
    proof_store: Arc<Mutex<HashMap<ProofRequestId, Proof>>>,
    // Counter for generating unique (for this mock instance) ProofRequestIds
    next_job_id_counter: Arc<Mutex<u32>>,
}

impl MockZkCoprocessor {
    pub fn new() -> Self {
        Default::default()
    }

    // --- Configuration methods for general behavior ---
    pub fn set_default_submit_trace_behavior(&self, behavior: MethodMockBehavior) {
        *self.default_submit_trace_behavior.lock().unwrap() = Some(behavior);
    }
    pub fn set_default_check_status_behavior(&self, behavior: MethodMockBehavior) {
        *self.default_check_status_behavior.lock().unwrap() = Some(behavior);
    }
    pub fn set_default_get_proof_behavior(&self, behavior: MethodMockBehavior) {
        *self.default_get_proof_behavior.lock().unwrap() = Some(behavior);
    }
    pub fn set_default_verify_proof_behavior(&self, behavior: MethodMockBehavior) {
        *self.default_verify_proof_behavior.lock().unwrap() = Some(behavior);
    }

    // --- Configuration methods for specific calls / job IDs ---
    pub fn expect_submit_trace(&self, behavior: MethodMockBehavior) {
        *self.submit_trace_behavior.lock().unwrap() = Some(behavior);
    }

    pub fn expect_check_status_for_job(
        &self,
        job_id: ProofRequestId,
        behavior: MethodMockBehavior,
    ) {
        self.check_status_behaviors
            .lock()
            .unwrap()
            .insert(job_id, behavior);
    }

    pub fn expect_get_proof_for_job(
        &self,
        job_id: ProofRequestId,
        behavior: MethodMockBehavior,
    ) {
        self.get_proof_behaviors
            .lock()
            .unwrap()
            .insert(job_id, behavior);
    }

    pub fn expect_verify_proof(&self, behavior: MethodMockBehavior) {
        *self.verify_proof_behavior.lock().unwrap() = Some(behavior);
    }

    /// Helper to add a proof to the internal store for testing get_proof.
    pub fn add_proof_to_store(&self, job_id: ProofRequestId, proof: Proof) {
        self.proof_store.lock().unwrap().insert(job_id, proof);
    }

    fn get_next_job_id(&self) -> ProofRequestId {
        let mut counter = self.next_job_id_counter.lock().unwrap();
        let id_val = *counter;
        *counter += 1;
        ProofRequestId(format!("mock_job_id_{}", id_val))
    }
}

#[async_trait]
impl ZkCoprocessorApi for MockZkCoprocessor {
    async fn submit_trace_for_proving(
        &self,
        _trace: ExecutionTrace,
    ) -> Result<ProofRequestId, CoprocessorApiError> {
        let behavior_opt = self
            .submit_trace_behavior
            .lock()
            .unwrap()
            .take()
            .or_else(|| self.default_submit_trace_behavior.lock().unwrap().clone());

        match behavior_opt {
            Some(MethodMockBehavior::SubmitTraceReturns(res)) => res,
            Some(MethodMockBehavior::Panic(msg)) => panic!("{}", msg),
            Some(_) => Err(CoprocessorApiError::InternalError {
                message: "Mismatched mock behavior for submit_trace_for_proving"
                    .to_string(),
            }),
            None => Ok(self.get_next_job_id()), // Default: succeed and return a new ID
        }
    }

    async fn check_proof_status(
        &self,
        job_id: &ProofRequestId,
    ) -> Result<ProofStatus, CoprocessorApiError> {
        let behavior_opt = self
            .check_status_behaviors
            .lock()
            .unwrap()
            .remove(job_id)
            .or_else(|| self.default_check_status_behavior.lock().unwrap().clone());

        match behavior_opt {
            Some(MethodMockBehavior::CheckStatusReturns(res)) => res,
            Some(MethodMockBehavior::Panic(msg)) => panic!("{}", msg),
            Some(_) => Err(CoprocessorApiError::InternalError {
                message: "Mismatched mock behavior for check_proof_status"
                    .to_string(),
            }),
            None => Ok(ProofStatus::Pending), // Default: Pending
        }
    }

    async fn get_proof(
        &self,
        job_id: &ProofRequestId,
    ) -> Result<Proof, CoprocessorApiError> {
        let behavior_opt = self
            .get_proof_behaviors
            .lock()
            .unwrap()
            .remove(job_id)
            .or_else(|| self.default_get_proof_behavior.lock().unwrap().clone());

        match behavior_opt {
            Some(MethodMockBehavior::GetProofReturns(res)) => res,
            Some(MethodMockBehavior::Panic(msg)) => panic!("{}", msg),
            Some(_) => Err(CoprocessorApiError::InternalError {
                message: "Mismatched mock behavior for get_proof".to_string(),
            }),
            None => {
                // Default: check proof store or return JobNotFound
                if let Some(proof) = self.proof_store.lock().unwrap().get(job_id) {
                    Ok(proof.clone())
                } else {
                    Err(CoprocessorApiError::JobNotFound)
                }
            }
        }
    }

    async fn verify_proof(
        &self,
        _proof: &Proof,
        _public_inputs: &PublicInputs,
    ) -> Result<bool, CoprocessorApiError> {
        let behavior_opt = self
            .verify_proof_behavior
            .lock()
            .unwrap()
            .take()
            .or_else(|| self.default_verify_proof_behavior.lock().unwrap().clone());

        match behavior_opt {
            Some(MethodMockBehavior::VerifyProofReturns(res)) => res,
            Some(MethodMockBehavior::Panic(msg)) => panic!("{}", msg),
            Some(_) => Err(CoprocessorApiError::InternalError {
                message: "Mismatched mock behavior for verify_proof".to_string(),
            }),
            None => Ok(true), // Default: succeed verification
        }
    }
}
