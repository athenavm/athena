use std::collections::BTreeMap;

use athena_interface::MethodSelector;

use crate::instruction::Instruction;

/// A program that can be executed by the VM.
#[derive(Debug, Clone, Default)]
pub struct Program {
  /// The instructions of the program.
  pub instructions: Vec<Instruction>,

  /// Symbol table.
  /// Used to execute a method by name.
  pub symbol_table: BTreeMap<String, u32>,

  /// Method selector table.
  /// Used in the context of a transaction with a fixed-length method selector encoding.
  pub selector_table: BTreeMap<MethodSelector, u32>,

  /// The start address of the program.
  pub pc_start: u32,

  /// The base address of the program.
  pub pc_base: u32,

  /// The initial memory image, useful for global constants.
  pub memory_image: BTreeMap<u32, u32>,
}

impl Program {
  pub(crate) fn instruction(&self, pc: u32) -> Option<Instruction> {
    let idx = ((pc - self.pc_base) / 4) as usize;
    self.instructions.get(idx).copied()
  }
}
