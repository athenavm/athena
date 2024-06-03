#[cfg(test)]
pub mod tests {
    /// Demos.

    // pub const FIBONACCI_IO_ELF: &[u8] =
    //     include_bytes!("../../../examples/fibonacci/program/elf/riscv32im-succinct-zkvm-elf");

    // pub const IO_ELF: &[u8] =
    //     include_bytes!("../../../examples/io/program/elf/riscv32im-succinct-zkvm-elf");

    /// Tests.

    pub const FIBONACCI_ELF: &[u8] =
        include_bytes!("../../../tests/fibonacci/elf/riscv32im-succinct-zkvm-elf");

    pub const PANIC_ELF: &[u8] =
        include_bytes!("../../../tests/panic/elf/riscv32im-succinct-zkvm-elf");
}
