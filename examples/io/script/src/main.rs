use athena_sdk::{AthenaStdin, ExecutionClient};
use serde::{Deserialize, Serialize};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_bytes!("../../program/elf/io-program");

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MyPointUnaligned {
  pub x: usize,
  pub y: usize,
  pub b: bool,
}

fn main() {
  tracing_subscriber::fmt::init();

  // Create an input stream.
  let mut stdin = AthenaStdin::new();
  let p = MyPointUnaligned {
    x: 1,
    y: 2,
    b: true,
  };
  let q = MyPointUnaligned {
    x: 3,
    y: 4,
    b: false,
  };
  stdin.write(&p);
  stdin.write(&q);

  // Run the given program.
  let client = ExecutionClient::new();
  let (mut output, _) = client
    .execute(ELF, stdin, None, None, None)
    .expect("execution failed");

  // Read the output.
  let r = output.read::<MyPointUnaligned>();
  println!("r: {:?}", r);

  println!("successful execution")
}
