#[cfg(test)]
pub mod tests {
  pub const IO_ELF: &[u8] = include_bytes!("../../../examples/io/program/elf/io-program");

  pub const TEST_PANIC_ELF: &[u8] = include_bytes!("../../../tests/panic/elf/panic-test");
}
