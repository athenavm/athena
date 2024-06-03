mod local;
mod mock;
mod network;

use anyhow::Result;
pub use local::LocalProver;
pub use mock::MockProver;
use sp1_prover::SP1Prover;
use sp1_prover::SP1Stdin;

/// An implementation of [crate::ProverClient].
pub trait Prover: Send + Sync {
    fn id(&self) -> String;

    fn sp1_prover(&self) -> &SP1Prover;

    fn setup(&self, elf: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey);

    /// Prove the execution of a RISCV ELF with the given inputs.
    fn prove(&self, pk: &SP1ProvingKey, stdin: SP1Stdin) -> Result<SP1Proof>;
}
