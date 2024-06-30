#[cfg(test)]
pub mod tests {
  /// Demos.

  pub const FIBONACCI_ELF: &[u8] =
    include_bytes!("../../../examples/fibonacci/program/elf/fibonacci-program");

  pub const HELLO_WORLD_ELF: &[u8] =
    include_bytes!("../../../examples/hello_world/program/elf/hello-world-program");

  pub const IO_ELF: &[u8] = include_bytes!("../../../examples/io/program/elf/io-program");

  /// Tests.

  pub const TEST_FIBONACCI_ELF: &[u8] =
    include_bytes!("../../../tests/fibonacci/elf/fibonacci-test");

  pub const TEST_HOST: &[u8] = include_bytes!("../../../tests/host/elf/host-test");

  pub const TEST_HINT_IO: &[u8] = include_bytes!("../../../tests/hint-io/elf/hint-io-test");

  pub const TEST_PANIC_ELF: &[u8] = include_bytes!("../../../tests/panic/elf/panic-test");
}
