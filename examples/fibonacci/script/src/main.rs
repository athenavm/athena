use athena_interface::MockHost;
use athena_sdk::{utils, AthenaPublicValues, AthenaStdin, ExecutionClient};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_bytes!("../../program/elf/fibonacci-program");

fn main() {
  // Setup logging.
  utils::setup_logger();

  // Create an input stream and write '500' to it.
  let n = 500u32;

  let mut stdin = AthenaStdin::new();
  stdin.write(&n);

  // Generate the proof for the given program and input.
  let client = ExecutionClient::new();
  let (mut output, _opt): (AthenaPublicValues, Option<u32>) = client
    .execute::<MockHost>(ELF, stdin, None, None, None)
    .expect("execution failed");

  println!("executed program");

  // Read and verify the output.
  let _ = output.read::<u32>();
  let a = output.read::<u32>();
  let b = output.read::<u32>();

  println!("a: {}", a);
  println!("b: {}", b);
}
