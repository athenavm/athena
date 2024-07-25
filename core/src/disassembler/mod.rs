mod elf;
mod instruction;

pub use elf::*;
pub use instruction::*;

use core::panic;
use std::{collections::BTreeMap, fs::File, io::Read};

use crate::runtime::{Instruction, Program};

impl Program {
  /// Create a new program.
  pub const fn new(instructions: Vec<Instruction>, pc_start: u32, pc_base: u32) -> Self {
    Self {
      instructions,
      pc_start,
      pc_base,
      memory_image: BTreeMap::new(),
    }
  }

  /// Disassemble a RV32IM ELF to a program that be executed by the VM.
  pub fn from(input: &[u8]) -> Self {
    // Check the magic number
    if input.len() < 4 {
      panic!("malformed input");
    } else if &input[0..4] == b"\x7fELF" {
      // Decode the bytes as an ELF.
      let elf = Elf::decode(input);

      // Transpile the RV32IM instructions.
      let instructions = transpile(&elf.instructions);

      // Return the program.
      Program {
        instructions,
        pc_start: elf.pc_start,
        pc_base: elf.pc_base,
        memory_image: elf.memory_image,
      }
    } else if &input[0..4] == b"\x7fATH" {
      assert_eq!(input.len() % 4, 0, "malformed input");

      // convert the bytes into a vec of 32-bit words
      let mut instructions = Vec::new();
      for i in (4..input.len()).step_by(4) {
        let word = u32::from_le_bytes([input[i], input[i + 1], input[i + 2], input[i + 3]]);
        instructions.push(word);
      }

      // short-circuit for Athena binaries
      let instructions = transpile(&instructions);

      // Return the program.
      Program::new(instructions, 0, 0)
    } else {
      panic!("unknown executable format");
    }
  }

  /// Disassemble a RV32IM ELF to a program that be executed by the VM from a file path.
  pub fn from_elf(path: &str) -> Self {
    let mut elf_code = Vec::new();
    File::open(path)
      .expect("failed to open input file")
      .read_to_end(&mut elf_code)
      .expect("failed to read from input file");
    Program::from(&elf_code)
  }
}
