mod elf;
mod instruction;

use anyhow::Context;
use core::panic;
pub use elf::*;
use instruction::transpile;
use std::{collections::BTreeMap, fs::File, io::Read};

use crate::runtime::Program;

use athena_interface::MethodSelector;

impl Program {
  /// Disassemble an ELF to a program that be executed by the VM.
  pub fn from(input: &[u8]) -> anyhow::Result<Self> {
    // Check the magic number
    anyhow::ensure!(input.len() >= 4);
    if &input[0..4] == b"\x7fELF" {
      // Decode the bytes as an ELF.
      let elf = Elf::decode(input).context("decoding ELF")?;

      // Transpile the RV32IM instructions.
      let instructions = transpile(&elf.instructions)?;

      // Construct the selector table from the symbol table.
      let mut selector_table = BTreeMap::new();
      for (symbol, address) in &elf.symbol_table {
        let selector = MethodSelector::from(symbol.as_str());
        tracing::info!(
          "adding selector table entry for symbol {}: 0x{} at address 0x{:x}",
          symbol,
          selector,
          address,
        );
        selector_table.insert(selector, *address);
      }

      // Return the program.
      Ok(Program {
        instructions,
        symbol_table: elf.symbol_table,
        selector_table,
        pc_start: elf.pc_start,
        pc_base: elf.pc_base,
        memory_image: elf.memory_image,
      })
    } else if &input[0..4] == b"\x7fATH" {
      assert_eq!(input.len() % 4, 0, "malformed input");

      // convert the bytes into a vec of 32-bit words
      let mut instructions = Vec::new();
      for i in (4..input.len()).step_by(4) {
        let word = u32::from_le_bytes([input[i], input[i + 1], input[i + 2], input[i + 3]]);
        instructions.push(word);
      }

      // short-circuit for Athena binaries
      let instructions = transpile(&instructions)?;

      // Return the program.
      Ok(Program::new(instructions, 0, 0))
    } else {
      panic!("unknown executable format");
    }
  }

  /// Disassemble a RV32IM ELF to a program that be executed by the VM from a file path.
  pub fn from_elf(path: &str) -> anyhow::Result<Self> {
    let mut elf_code = Vec::new();
    File::open(path)
      .expect("failed to open input file")
      .read_to_end(&mut elf_code)
      .expect("failed to read from input file");
    Program::from(&elf_code)
  }
}
