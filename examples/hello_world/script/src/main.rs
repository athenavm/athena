use athena_sdk::{utils, AthenaStdin, ExecutionClient};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_bytes!("../../program/elf/hello-world-program");

fn main() {
  // Setup a tracer for logging.
  utils::setup_logger();

  // Create an input stream.
  let stdin = AthenaStdin::new();

  // Run the given program.
  let client = ExecutionClient::new();
  client.execute(ELF, stdin, None, None, None).unwrap();

  println!("successful execution")
}
