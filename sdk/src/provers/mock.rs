#![allow(unused_variables)]
use crate::{
    Prover, SP1Proof, SP1ProofVerificationError, SP1ProofWithPublicValues
};
use anyhow::Result;
use sp1_prover::{
    SP1Prover, SP1Stdin,
};

/// An implementation of [crate::ProverClient] that can generate mock proofs.
pub struct MockProver {
    pub(crate) prover: SP1Prover,
}

impl MockProver {
    /// Creates a new [MockProver].
    pub fn new() -> Self {
        let prover = SP1Prover::new();
        Self { prover }
    }
}

impl Prover for MockProver {
    fn id(&self) -> String {
        "mock".to_string()
    }

    fn setup(&self, elf: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey) {
        self.prover.setup(elf)
    }

    fn sp1_prover(&self) -> &SP1Prover {
        unimplemented!("MockProver does not support SP1Prover")
    }

    fn prove(&self, pk: &SP1ProvingKey, stdin: SP1Stdin) -> Result<SP1Proof> {
        let public_values = SP1Prover::execute(&pk.elf, &stdin)?;
        Ok(SP1ProofWithPublicValues {
            proof: vec![],
            stdin,
            public_values,
        })
    }
}

impl Default for MockProver {
    fn default() -> Self {
        Self::new()
    }
}
