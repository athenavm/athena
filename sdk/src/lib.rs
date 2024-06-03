//! # Athena SDK
//!
//! A library for interacting with the Athena RISC-V VM.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod provers;
pub mod utils {
    pub use athena_core::utils::setup_logger;
}

use std::{env, fmt::Debug};

use anyhow::{Ok, Result};
pub use provers::{LocalProver, MockProver, Prover};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub use sp1_prover::{
    SP1Prover, SP1PublicValues, AthenaStdin,
};

/// A client for interacting with Athena.
pub struct ProverClient {
    /// The underlying prover implementation.
    pub prover: Box<dyn Prover>,
}

/// A proof generated with Athena.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "P: Serialize + Debug + Clone"))]
#[serde(bound(deserialize = "P: DeserializeOwned + Debug + Clone"))]
pub struct SP1ProofWithPublicValues<P> {
    pub proof: P,
    pub stdin: AthenaStdin,
    pub public_values: SP1PublicValues,
}

impl ProverClient {
    /// Creates a new [ProverClient].
    ///
    /// Setting the `ATHENA_PROVER` enviroment variable can change the prover used under the hood.
    /// - `local` (default): Uses [LocalProver]. Recommended for proving end-to-end locally.
    /// - `mock`: Uses [MockProver]. Recommended for testing and development.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use athena_sdk::ProverClient;
    ///
    /// std::env::set_var("ATHENA_PROVER", "local");
    /// let client = ProverClient::new();
    /// ```
    pub fn new() -> Self {
        match env::var("ATHENA_PROVER")
            .unwrap_or("local".to_string())
            .to_lowercase()
            .as_str()
        {
            "mock" => Self {
                prover: Box::new(MockProver::new()),
            },
            "local" => Self {
                prover: Box::new(LocalProver::new()),
            },
            "network" => Self {
                prover: Box::new(NetworkProver::new()),
            },
            _ => panic!(
                "invalid value for ATHENA_PROVER enviroment variable: expected 'local', 'mock', or 'remote'"
            ),
        }
    }

    /// Creates a new [ProverClient] with the mock prover.
    ///
    /// Recommended for testing and development. You can also use [ProverClient::new] to set the
    /// prover to `mock` with the `ATHENA_PROVER` enviroment variable.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use athena_sdk::ProverClient;
    ///
    /// let client = ProverClient::mock();
    /// ```
    pub fn mock() -> Self {
        Self {
            prover: Box::new(MockProver::new()),
        }
    }

    /// Creates a new [ProverClient] with the local prover.
    ///
    /// Recommended for proving end-to-end locally. You can also use [ProverClient::new] to set the
    /// prover to `local` with the `ATHENA_PROVER` enviroment variable.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use athena_sdk::ProverClient;
    ///
    /// let client = ProverClient::local();
    /// ```
    pub fn local() -> Self {
        Self {
            prover: Box::new(LocalProver::new()),
        }
    }

    /// Executes the given program on the given input (without generating a proof).
    ///
    /// Returns the public values of the program after it has been executed.
    ///
    ///
    /// ### Examples
    /// ```no_run
    /// use athena_sdk::{ProverClient, AthenaStdin};
    ///
    /// // Load the program.
    /// let elf = include_bytes!("../../examples/fibonacci/program/elf/riscv32im-succinct-zkvm-elf");
    ///
    /// // Initialize the prover client.
    /// let client = ProverClient::new();
    ///
    /// // Setup the inputs.
    /// let mut stdin = AthenaStdin::new();
    /// stdin.write(&10usize);
    ///
    /// // Execute the program on the inputs.
    /// let public_values = client.execute(elf, stdin).unwrap();
    /// ```
    pub fn execute(&self, elf: &[u8], stdin: AthenaStdin) -> Result<SP1PublicValues> {
        Ok(SP1Prover::execute(elf, &stdin)?)
    }

    /// Setup a program to be proven and verified by the SP1 RISC-V zkVM by computing the proving
    /// and verifying keys.
    ///
    /// The proving key and verifying key essentially embed the program, as well as other auxiliary
    /// data (such as lookup tables) that are used to prove the program's correctness.
    ///
    /// ### Examples
    /// ```no_run
    /// use athena_sdk::{ProverClient, AthenaStdin};
    ///
    /// let elf = include_bytes!("../../examples/fibonacci/program/elf/riscv32im-succinct-zkvm-elf");
    /// let client = ProverClient::new();
    /// let mut stdin = AthenaStdin::new();
    /// stdin.write(&10usize);
    /// let (pk, vk) = client.setup(elf);
    /// ```
    pub fn setup(&self, elf: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey) {
        self.prover.setup(elf)
    }
}

impl Default for ProverClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use crate::{utils, ProverClient, AthenaStdin};

    #[test]
    fn test_execute() {
        utils::setup_logger();
        let client = ProverClient::local();
        let elf =
            include_bytes!("../../examples/fibonacci/program/elf/riscv32im-succinct-zkvm-elf");
        let mut stdin = AthenaStdin::new();
        stdin.write(&10usize);
        client.execute(elf, stdin).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_execute_panic() {
        utils::setup_logger();
        let client = ProverClient::local();
        let elf = include_bytes!("../../tests/panic/elf/riscv32im-succinct-zkvm-elf");
        let mut stdin = AthenaStdin::new();
        stdin.write(&10usize);
        client.execute(elf, stdin).unwrap();
    }

    #[test]
    fn test_e2e_prove_local() {
        utils::setup_logger();
        let client = ProverClient::local();
        let elf =
            include_bytes!("../../examples/fibonacci/program/elf/riscv32im-succinct-zkvm-elf");
        let (pk, vk) = client.setup(elf);
        let mut stdin = AthenaStdin::new();
        stdin.write(&10usize);
        let proof = client.prove(&pk, stdin).unwrap();
        client.verify(&proof, &vk).unwrap();
    }

    #[test]
    fn test_e2e_prove_mock() {
        utils::setup_logger();
        let client = ProverClient::mock();
        let elf =
            include_bytes!("../../examples/fibonacci/program/elf/riscv32im-succinct-zkvm-elf");
        let (pk, vk) = client.setup(elf);
        let mut stdin = AthenaStdin::new();
        stdin.write(&10usize);
        let proof = client.prove(&pk, stdin).unwrap();
        client.verify(&proof, &vk).unwrap();
    }
}
