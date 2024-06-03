use anyhow::Result;
use cfg_if::cfg_if;
use sp1_prover::{SP1Prover, SP1Stdin};

use crate::{
    Prover, SP1Proof, SP1ProofWithPublicValues,
};

/// An implementation of [crate::ProverClient] that can generate end-to-end proofs locally.
pub struct LocalProver {
    prover: SP1Prover,
}

impl LocalProver {
    /// Creates a new [LocalProver].
    pub fn new() -> Self {
        let prover = SP1Prover::new();
        Self { prover }
    }
}

impl Prover for LocalProver {
    fn id(&self) -> String {
        "local".to_string()
    }

    fn setup(&self, elf: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey) {
        self.prover.setup(elf)
    }

    fn sp1_prover(&self) -> &SP1Prover {
        &self.prover
    }

    fn prove(&self, pk: &SP1ProvingKey, stdin: SP1Stdin) -> Result<SP1Proof> {
        let proof = self.prover.prove_core(pk, &stdin)?;
        Ok(SP1ProofWithPublicValues {
            proof: proof.proof.0,
            stdin: proof.stdin,
            public_values: proof.public_values,
        })
    }
}

impl Default for LocalProver {
    fn default() -> Self {
        Self::new()
    }
}
