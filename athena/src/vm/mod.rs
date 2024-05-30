pub mod memory;
pub mod interpreter;

use interpreter::Instruction;
use memory::Memory;

pub struct VM {
    pub memory: Memory,
    pub registers: [u32; 16], // 16 general-purpose registers
    pub pc: usize, // Program counter
}

impl VM {
    pub fn new(memory_size: usize) -> Self {
        VM {
            memory: Memory::new(memory_size),
            registers: [0; 16],
            pc: 0,
        }
    }

    pub fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD { rd, rs1, rs2 } => {
                self.registers[rd] = self.registers[rs1].wrapping_add(self.registers[rs2]);
            }
            Instruction::SUB { rd, rs1, rs2 } => {
                self.registers[rd] = self.registers[rs1].wrapping_sub(self.registers[rs2]);
            }
            Instruction::LW { rd, imm, rs1 } => {
                let addr = (self.registers[rs1] as i32).wrapping_add(imm) as usize;
                self.registers[rd] = self.memory.load(addr) as u32;
            }
            Instruction::SW { rs2, imm, rs1 } => {
                let addr = (self.registers[rs1] as i32).wrapping_add(imm) as usize;
                self.memory.store(addr, self.registers[rs2] as u8);
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            let instruction = self.fetch_instruction();
            let decoded = Instruction::decode(instruction);
            self.execute_instruction(decoded);
            self.pc += 4; // Assuming each instruction is 4 bytes
        }
    }

    fn fetch_instruction(&self) -> u32 {
        let mut bytes = [0u8; 4];
        for i in 0..4 {
            bytes[i] = self.memory.load(self.pc + i);
        }
        u32::from_le_bytes(bytes)
    }
}
