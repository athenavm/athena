use athena_sdk::{utils, ExecutionClient, AthenaStdin};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {
    // Setup a tracer for logging.
    utils::setup_logger();

    // Create an input stream.
    let stdin = AthenaStdin::new();

    // Run the given program.
    let client = ExecutionClient::new();
    let _output = client.execute(ELF, stdin).unwrap();

    println!("successful execution")
}
