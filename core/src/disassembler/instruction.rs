use anyhow::anyhow;
use rrs_lib::instruction_formats::{
  BType, IType, ITypeCSR, ITypeShamt, JType, RType, SType, UType,
};
use rrs_lib::{process_instruction, InstructionProcessor};

use crate::instruction::Instruction;
use crate::runtime::Register;

pub(crate) struct InstructionTranspilerNew;

impl InstructionProcessor for InstructionTranspilerNew {
  type InstructionResult = anyhow::Result<Instruction>;

  fn process_add(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Add(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_sub(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Sub(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_sll(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Sll(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_slt(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Slt(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_sltu(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Sltu(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_xor(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Xor(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_srl(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Srl(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_sra(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Sra(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_or(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Or(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_and(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::And(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_addi(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Addi(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_slli(&mut self, dec_insn: ITypeShamt) -> Self::InstructionResult {
    Ok(Instruction::Slli(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.shamt as u32,
    ))
  }

  fn process_slti(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Slti(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_sltui(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Sltiu(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_xori(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Xori(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_srli(&mut self, dec_insn: ITypeShamt) -> Self::InstructionResult {
    Ok(Instruction::Srli(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.shamt as u32,
    ))
  }

  fn process_srai(&mut self, dec_insn: ITypeShamt) -> Self::InstructionResult {
    Ok(Instruction::Srai(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.shamt as u32,
    ))
  }

  fn process_ori(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Ori(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_andi(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Andi(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_lui(&mut self, dec_insn: UType) -> Self::InstructionResult {
    Ok(Instruction::Lui(
      Register::try_from(dec_insn.rd)?,
      dec_insn.imm as u32,
    ))
  }

  fn process_auipc(&mut self, dec_insn: UType) -> Self::InstructionResult {
    Ok(Instruction::Auipc(
      Register::try_from(dec_insn.rd)?,
      dec_insn.imm,
    ))
  }

  fn process_beq(&mut self, dec_insn: BType) -> Self::InstructionResult {
    Ok(Instruction::Beq(
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
      dec_insn.imm,
    ))
  }

  fn process_bne(&mut self, dec_insn: BType) -> Self::InstructionResult {
    Ok(Instruction::Bne(
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
      dec_insn.imm,
    ))
  }

  fn process_blt(&mut self, dec_insn: BType) -> Self::InstructionResult {
    Ok(Instruction::Blt(
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
      dec_insn.imm,
    ))
  }

  fn process_bltu(&mut self, dec_insn: BType) -> Self::InstructionResult {
    Ok(Instruction::Bltu(
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
      dec_insn.imm,
    ))
  }

  fn process_bge(&mut self, dec_insn: BType) -> Self::InstructionResult {
    Ok(Instruction::Bge(
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
      dec_insn.imm,
    ))
  }

  fn process_bgeu(&mut self, dec_insn: BType) -> Self::InstructionResult {
    Ok(Instruction::Bgeu(
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
      dec_insn.imm,
    ))
  }

  fn process_lb(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Lb(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_lbu(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Lbu(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_lh(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Lh(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_lhu(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Lhu(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_lw(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Lw(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_sb(&mut self, dec_insn: SType) -> Self::InstructionResult {
    Ok(Instruction::Sb(
      Register::try_from(dec_insn.rs2)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_sh(&mut self, dec_insn: SType) -> Self::InstructionResult {
    Ok(Instruction::Sh(
      Register::try_from(dec_insn.rs2)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_sw(&mut self, dec_insn: SType) -> Self::InstructionResult {
    Ok(Instruction::Sw(
      Register::try_from(dec_insn.rs2)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_jal(&mut self, dec_insn: JType) -> Self::InstructionResult {
    Ok(Instruction::Jal(
      Register::try_from(dec_insn.rd)?,
      dec_insn.imm,
    ))
  }

  fn process_jalr(&mut self, dec_insn: IType) -> Self::InstructionResult {
    Ok(Instruction::Jalr(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      dec_insn.imm,
    ))
  }

  fn process_mul(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Mul(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_mulh(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Mulh(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_mulhu(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Mulhu(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_mulhsu(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Mulhsu(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_div(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Div(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_divu(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Divu(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_rem(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Rem(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_remu(&mut self, dec_insn: RType) -> Self::InstructionResult {
    Ok(Instruction::Remu(
      Register::try_from(dec_insn.rd)?,
      Register::try_from(dec_insn.rs1)?,
      Register::try_from(dec_insn.rs2)?,
    ))
  }

  fn process_ecall(&mut self) -> Self::InstructionResult {
    Ok(Instruction::Ecall)
  }

  fn process_ebreak(&mut self) -> Self::InstructionResult {
    Ok(Instruction::Ebreak)
  }

  fn process_csrrc(&mut self, _: ITypeCSR) -> Self::InstructionResult {
    Ok(Instruction::NotImplemented { opcode: "csrrc" })
  }

  fn process_csrrci(&mut self, _: ITypeCSR) -> Self::InstructionResult {
    Ok(Instruction::NotImplemented { opcode: "csrrci" })
  }

  fn process_csrrs(&mut self, _: ITypeCSR) -> Self::InstructionResult {
    Ok(Instruction::NotImplemented { opcode: "csrrci" })
  }

  fn process_csrrsi(&mut self, _: ITypeCSR) -> Self::InstructionResult {
    Ok(Instruction::NotImplemented { opcode: "csrrci" })
  }

  fn process_csrrw(&mut self, _: ITypeCSR) -> Self::InstructionResult {
    Ok(Instruction::NotImplemented { opcode: "csrrw" })
  }

  fn process_csrrwi(&mut self, _: ITypeCSR) -> Self::InstructionResult {
    Ok(Instruction::NotImplemented { opcode: "csrrwi" })
  }

  fn process_fence(&mut self, _: IType) -> Self::InstructionResult {
    Ok(Instruction::NotImplemented { opcode: "fence" })
  }

  fn process_mret(&mut self) -> Self::InstructionResult {
    Ok(Instruction::NotImplemented { opcode: "mret" })
  }

  fn process_wfi(&mut self) -> Self::InstructionResult {
    Ok(Instruction::NotImplemented { opcode: "wfi" })
  }
}

/// Transpile the instructions from the 32-bit encoded instructions.
pub(crate) fn transpile(instructions_u32: &[u32]) -> anyhow::Result<Vec<Instruction>> {
  let mut transpiler = InstructionTranspilerNew;
  let instrs = instructions_u32
    .iter()
    .map(|i| {
      process_instruction(&mut transpiler, *i)
        .ok_or(anyhow!("unknown opcode for instruction {:x}", *i))
    })
    .collect::<anyhow::Result<Vec<anyhow::Result<_>>>>()?;
  instrs.into_iter().collect()
}

#[cfg(test)]
mod tests {
  use crate::disassembler::transpile;

  #[test]
  fn transpiling_invalid_instruction_fails() {
    assert!(transpile(&[0x12345678]).is_err());
  }
}
