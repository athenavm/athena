//! A simple script to execute a given program.

use athena_sdk::{utils, ExecutionClient, AthenaStdin};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {
    // Setup logging.
    utils::setup_logger();

    // Generate proof.
    let mut stdin = AthenaStdin::new();
    let n = 186u32;
    stdin.write(&n);
    let client = ExecutionClient::new();
    let mut output = client.execute(ELF, stdin).expect("execution failed");

    // Read output.
    let a = output.read::<u128>();
    let b = output.read::<u128>();
    println!("a: {}", a);
    println!("b: {}", b);

    println!("successfully executed the program!")
}
