use athena_sdk::{AthenaStdin, ExecutionClient};

/// The ELF we want to execute inside the zkVM.
// const ELF: &[u8] = include_bytes!("../../program/elf/hasher-program");
const ELF: &[u8] =
  include_bytes!("../../program/target/riscv32em-athena-zkvm-elf/debug/hasher-program");

fn main() {
  tracing_subscriber::fmt::init();

  // Create an input stream.
  let stdin = AthenaStdin::new();

  // Run the given program.
  let client = ExecutionClient::new();
  // client.execute(ELF, stdin, None, None, None).unwrap();
  client
    .execute_with_gdb(ELF, stdin, None, None, None, 9001)
    .unwrap();

  println!("successful execution")
}
