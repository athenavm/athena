pub mod memory;
pub mod interpreter;

// use interpreter::Instruction;
// use memory::Memory;

use sp1::runtime::{Program, Runtime};
use sp1::utils::SP1CoreOpts;

pub struct VM {
  program: Program,

  // We'll need these soon
  // pub memory: Memory,
  // pub registers: [u32; 16], // 16 general-purpose registers
  // pub pc: usize, // Program counter
}

impl VM {
    pub fn new(path: &str) -> Self {
      // Outsource execution to a third party library for now
      let program = Program::from_elf(path);
      VM {
        program: program,
          // memory: Memory::new(memory_size),
          // registers: [0; 16],
          // pc: 0,
      }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
      let mut runtime = Runtime::new(self.program.clone(), SP1CoreOpts::default());
      runtime.run()?;
      Ok(())
    }
}
