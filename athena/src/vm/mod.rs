pub mod memory;
pub mod interpreter;

pub struct VM {
    pub memory: memory::Memory,
    pub registers: [u32; 16], // 16 general-purpose registers for RV32E (embedded) ISA
    // Other components like program counter, etc.
}

impl VM {
    pub fn new(memory_size: usize) -> Self {
        VM {
            memory: memory::Memory::new(memory_size),
            registers: [0; 16],
            // Initialize other components
        }
    }

    pub fn execute(&mut self) {
        // Execution logic
    }
}
