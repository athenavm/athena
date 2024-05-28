pub mod memory;
pub mod interpreter;

pub struct VM {
    pub memory: memory::Memory,
    // Other components like registers, program counter, etc.
}

impl VM {
    pub fn new(size: usize) -> Self {
        VM {
            memory: memory::Memory::new(size),
            // Initialize other components
        }
    }

    pub fn execute(&mut self) {
        // Execution logic
    }
}
