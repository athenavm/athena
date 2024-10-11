//! # Athena SDK
//!
//! A library for interacting with the Athena RISC-V VM.

pub use athena_core::io::{AthenaPublicValues, AthenaStdin};
use athena_core::runtime::{ExecutionError, Program, Runtime};
use athena_core::utils::AthenaCoreOpts;
use athena_interface::{AthenaContext, HostInterface, MethodSelector, METHOD_SELECTOR_DEFAULT};

/// A client for interacting with Athena.
pub struct ExecutionClient;

impl ExecutionClient {
  /// Creates a new [ExecutionClient].
  ///
  /// ### Examples
  ///
  /// ```no_run
  /// use athena_sdk::ExecutionClient;
  ///
  /// let client = ExecutionClient::new();
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
  /// use athena_sdk::{ExecutionClient, AthenaStdin};
  ///
  /// // Load the program.
  /// let elf = include_bytes!("../../examples/fibonacci/program/elf/fibonacci-program");
  ///
  /// // Initialize the execution client.
  /// let client = ExecutionClient::new();
  ///
  /// // Setup the inputs.
  /// let mut stdin = AthenaStdin::new();
  /// stdin.write(&10usize);
  ///
  /// // Execute the program on the inputs.
  /// let (public_values, gas_left) = client.execute(elf, stdin, None, None, None).unwrap();
  /// ```
  pub fn execute(
    &self,
    elf: &[u8],
    stdin: AthenaStdin,
    host: Option<&mut dyn HostInterface>,
    max_gas: Option<u32>,
    context: Option<AthenaContext>,
  ) -> Result<(AthenaPublicValues, Option<u32>), ExecutionError> {
    let program = Program::from(elf);
    let opts = match max_gas {
      None => AthenaCoreOpts::default(),
      Some(max_gas) => {
        AthenaCoreOpts::default().with_options(vec![athena_core::utils::with_max_gas(max_gas)])
      }
    };
    let mut runtime = Runtime::new(program, host, opts, context);
    runtime.write_vecs(&stdin.buffer);
    runtime.execute().map(|gas_left| {
      (
        AthenaPublicValues::from(&runtime.state.public_values_stream),
        gas_left,
      )
    })
  }

  pub fn execute_function(
    &self,
    elf: &[u8],
    selector: &MethodSelector,
    stdin: AthenaStdin,
    host: Option<&mut dyn HostInterface>,
    max_gas: Option<u32>,
    context: Option<AthenaContext>,
  ) -> Result<(AthenaPublicValues, Option<u32>), ExecutionError> {
    let program = Program::from(elf);
    let opts = match max_gas {
      None => AthenaCoreOpts::default(),
      Some(max_gas) => {
        AthenaCoreOpts::default().with_options(vec![athena_core::utils::with_max_gas(max_gas)])
      }
    };
    let mut runtime = Runtime::new(program, host, opts, context);
    runtime.write_vecs(&stdin.buffer);
    let execute_fn = if selector == &METHOD_SELECTOR_DEFAULT {
      runtime.execute()
    } else {
      runtime.execute_selector(selector)
    };
    execute_fn.map(|gas_left| {
      (
        AthenaPublicValues::from(&runtime.state.public_values_stream),
        gas_left,
      )
    })
  }
}

impl Default for ExecutionClient {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {

  use crate::{AthenaStdin, ExecutionClient};
  use athena_interface::{AthenaContext, MockHost, ADDRESS_ALICE};

  fn setup_logger() {
    let _ = tracing_subscriber::fmt()
      .with_test_writer()
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .try_init();
  }

  #[test]
  fn test_execute() {
    setup_logger();

    let client = ExecutionClient::new();
    let elf = include_bytes!("../../examples/fibonacci/program/elf/fibonacci-program");
    let mut stdin = AthenaStdin::new();
    stdin.write(&10usize);
    client.execute(elf, stdin, None, None, None).unwrap();
  }

  #[test]
  fn test_host() {
    setup_logger();

    let client = ExecutionClient::new();
    let elf = include_bytes!("../../tests/host/elf/host-test");
    let stdin = AthenaStdin::new();
    let mut host = MockHost::new();
    let ctx = AthenaContext::new(ADDRESS_ALICE, ADDRESS_ALICE, 0);
    client
      .execute(elf, stdin, Some(&mut host), Some(1_000_000), Some(ctx))
      .unwrap();
  }

  #[test]
  #[should_panic]
  fn test_missing_host() {
    setup_logger();

    let client = ExecutionClient::new();
    let elf = include_bytes!("../../tests/host/elf/host-test");
    let stdin = AthenaStdin::new();
    client.execute(elf, stdin, None, None, None).unwrap();
  }

  #[test]
  #[should_panic]
  fn test_execute_panic() {
    setup_logger();

    let client = ExecutionClient::new();
    let elf = include_bytes!("../../tests/panic/elf/panic-test");
    let mut stdin = AthenaStdin::new();
    stdin.write(&10usize);
    client.execute(elf, stdin, None, None, None).unwrap();
  }
}
