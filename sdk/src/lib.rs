//! # Athena SDK
//!
//! A library for interacting with the Athena RISC-V VM.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod utils {
    pub use athena_core::utils::setup_logger;
}

use std::{env, fmt::Debug};

use anyhow::{Ok, Result};
use athena_core::runtime::{ExecutionError, Runtime};
use athena_core::io::{AthenaPublicValues, AthenaStdin};

/// A client for interacting with Athena.
pub struct ProverClient {}

impl ProverClient {
    /// Creates a new [ProverClient].
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use athena_sdk::ProverClient;
    ///
    /// let client = ProverClient::new();
    /// ```
    pub fn new() -> Self {
      Self {}
    }

    /// Executes the given program on the given input.
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
    pub fn execute(&self, elf: &[u8], stdin: AthenaStdin) -> Result<AthenaPublicValues> {
      let program = Program::from(elf);
      let opts = AthenaCoreOpts::default();
      let mut runtime = Runtime::new(program, opts);
      runtime.write_vecs(&stdin.buffer);
      runtime.run_untraced()?;
      Ok(AthenaPublicValues::from(&runtime.state.public_values_stream))
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
        let client = ProverClient::new();
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
        let client = ProverClient::new();
        let elf = include_bytes!("../../tests/panic/elf/riscv32im-succinct-zkvm-elf");
        let mut stdin = AthenaStdin::new();
        stdin.write(&10usize);
        client.execute(elf, stdin).unwrap();
    }
}
