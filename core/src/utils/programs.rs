#[cfg(test)]
pub mod tests {
  /// Demos.
  pub const FIBONACCI_ELF: &[u8] =
    include_bytes!("../../../examples/fibonacci/program/elf/fibonacci-program");

  pub const HELLO_WORLD_ELF: &[u8] =
    include_bytes!("../../../examples/hello_world/program/elf/hello-world-program");

  pub const IO_ELF: &[u8] = include_bytes!("../../../examples/io/program/elf/io-program");

  pub const WALLET_ELF: &[u8] =
    include_bytes!("../../../examples/wallet/program/elf/wallet-template");

  /// Tests.
  pub const TEST_PANIC_ELF: &[u8] = include_bytes!("../../../tests/panic/elf/panic-test");
}
