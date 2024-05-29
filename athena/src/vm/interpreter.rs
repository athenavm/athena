pub enum Instruction {
  ADD { rd: usize, rs1: usize, rs2: usize },
  SUB { rd: usize, rs1: usize, rs2: usize },
  LW { rd: usize, imm: i32, rs1: usize },
  SW { rs2: usize, imm: i32, rs1: usize },
  // More instructions as needed
}

impl Instruction {
  pub fn decode(instruction: u32) -> Self {
      let opcode = instruction & 0x7f;
      match opcode {
          0x33 => { // R-type
              let rd = ((instruction >> 7) & 0x1f) as usize;
              let funct3 = (instruction >> 12) & 0x7;
              let rs1 = ((instruction >> 15) & 0x1f) as usize;
              let rs2 = ((instruction >> 20) & 0x1f) as usize;
              let funct7 = (instruction >> 25) & 0x7f;

              match (funct7, funct3) {
                  (0x00, 0x0) => Instruction::ADD { rd, rs1, rs2 },
                  (0x20, 0x0) => Instruction::SUB { rd, rs1, rs2 },
                  _ => unimplemented!(),
              }
          },
          0x03 => { // I-type for loads
              let rd = ((instruction >> 7) & 0x1f) as usize;
              let funct3 = (instruction >> 12) & 0x7;
              let rs1 = ((instruction >> 15) & 0x1f) as usize;
              let imm = ((instruction >> 20) & 0xfff) as i32;

              match funct3 {
                  0x2 => Instruction::LW { rd, imm, rs1 },
                  _ => unimplemented!(),
              }
          },
          0x23 => { // S-type for stores
              let imm4_0 = (instruction >> 7) & 0x1f;
              let funct3 = (instruction >> 12) & 0x7;
              let rs1 = ((instruction >> 15) & 0x1f) as usize;
              let rs2 = ((instruction >> 20) & 0x1f) as usize;
              let imm11_5 = (instruction >> 25) & 0x7f;
              let imm = ((imm11_5 << 5) | imm4_0) as i32;

              match funct3 {
                  0x2 => Instruction::SW { rs2, imm, rs1 },
                  _ => unimplemented!(),
              }
          },
          _ => unimplemented!(),
      }
  }
}
