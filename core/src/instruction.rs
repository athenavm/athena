use crate::runtime::Register;

/// Instructions set for RV32IM/RV32EM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
  /// RV32IM/RV32EM base instructions
  // U-type
  Lui(Register, u32), // Load Upper Immediate
  Auipc(Register, u32), // Add Upper Immediate to PC

  // J-type
  Jal(Register, i32), // Jump and Link

  // I-type
  Jalr(Register, Register, i32),  // Jump and Link Register
  Lb(Register, Register, i32),    // Load Byte
  Lh(Register, Register, i32),    // Load Halfword
  Lw(Register, Register, i32),    // Load Word
  Lbu(Register, Register, i32),   // Load Byte Unsigned
  Lhu(Register, Register, i32),   // Load Halfword Unsigned
  Addi(Register, Register, i32),  // Add Immediate
  Slti(Register, Register, i32),  // Set Less Than Immediate
  Sltiu(Register, Register, i32), // Set Less Than Immediate Unsigned
  Xori(Register, Register, i32),  // Xor Immediate
  Ori(Register, Register, i32),   // Or Immediate
  Andi(Register, Register, i32),  // And Immediate
  Slli(Register, Register, u32),  // Shift Left Logical Immediate
  Srli(Register, Register, u32),  // Shift Right Logical Immediate
  Srai(Register, Register, u32),  // Shift Right Arithmetic Immediate

  // S-type
  Sb(Register, Register, i32), // Store Byte
  Sh(Register, Register, i32), // Store Halfword
  Sw(Register, Register, i32), // Store Word

  // R-type
  Add(Register, Register, Register),  // Add
  Sub(Register, Register, Register),  // Subtract
  Sll(Register, Register, Register),  // Shift Left Logical
  Slt(Register, Register, Register),  // Set Less Than
  Sltu(Register, Register, Register), // Set Less Than Unsigned
  Xor(Register, Register, Register),  // Xor
  Srl(Register, Register, Register),  // Shift Right Logical
  Sra(Register, Register, Register),  // Shift Right Arithmetic
  Or(Register, Register, Register),   // Or
  And(Register, Register, Register),  // And

  // B-type
  Beq(Register, Register, i32),  // Branch Equal
  Bne(Register, Register, i32),  // Branch Not Equal
  Blt(Register, Register, i32),  // Branch Less Than
  Bge(Register, Register, i32),  // Branch Greater Equal
  Bltu(Register, Register, i32), // Branch Less Than Unsigned
  Bgeu(Register, Register, i32), // Branch Greater Equal Unsigned

  // RV32M Standard Extension for Integer Multiplication and Division
  Mul(Register, Register, Register),    // Multiply
  Mulh(Register, Register, Register),   // Multiply High Signed Signed
  Mulhsu(Register, Register, Register), // Multiply High Signed Unsigned
  Mulhu(Register, Register, Register),  // Multiply High Unsigned Unsigned
  Div(Register, Register, Register),    // Divide Signed
  Divu(Register, Register, Register),   // Divide Unsigned
  Rem(Register, Register, Register),    // Remainder Signed
  Remu(Register, Register, Register),   // Remainder Unsigned

  // System Instructions
  Ecall,  // Environment Call
  Ebreak, // Environment Break

  NotImplemented {
    opcode: &'static str,
  },
}
