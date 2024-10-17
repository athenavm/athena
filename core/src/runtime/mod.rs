mod instruction;
mod io;
mod opcode;
mod program;
mod register;
mod state;
mod syscall;
#[macro_use]
mod utils;

pub use instruction::*;
pub use opcode::*;
pub use program::*;
pub use register::*;
pub use state::*;
pub use syscall::*;
pub use utils::*;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::sync::Arc;

use thiserror::Error;

use crate::utils::AthenaCoreOpts;

use athena_interface::{AthenaContext, HostInterface, StatusCode};

/// An implementation of a runtime for the Athena RISC-V VM.
///
/// The runtime is responsible for executing a user program.
///
/// For more information on the RV32IM instruction set, see the following:
/// https://www.cs.sfu.ca/~ashriram/Courses/CS295/assets/notebooks/RISCV/RISCV_CARD.pdf
pub struct Runtime<'host> {
  /// The program.
  pub program: Arc<Program>,

  /// Runtime context.
  pub context: Option<AthenaContext>,

  /// The state of the execution.
  pub state: ExecutionState,

  /// The host interface for host calls.
  pub host: Option<&'host mut dyn HostInterface>,

  /// A counter for the number of cycles that have been executed in certain functions.
  pub cycle_tracker: HashMap<String, (u64, u32)>,

  /// A buffer for stdout and stderr IO.
  pub io_buf: HashMap<u32, String>,

  /// A buffer for writing trace events to a file.
  pub trace_buf: Option<BufWriter<File>>,

  /// Whether the runtime is in constrained mode or not.
  ///
  /// In unconstrained mode, any events, clock, register, or memory changes are reset after leaving
  /// the unconstrained block. The only thing preserved is writes to the input stream.
  pub unconstrained: bool,

  /// Max gas for the runtime.
  pub max_gas: Option<u32>,

  /// The mapping between syscall codes and their implementations.
  pub syscall_map: HashMap<SyscallCode, Arc<dyn Syscall>>,

  /// The maximum number of cycles for a syscall.
  pub max_syscall_cycles: u32,
}

/// An record of a write to a memory address.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MemoryAccessPosition {
  Memory = 0,
  // Note that these AccessPositions mean that when when read/writing registers, they must be
  // read/written in the following order: C, B, A.
  C = 1,
  B = 2,
  A = 3,
}

#[derive(Error, Debug, PartialEq)]
pub enum ExecutionError {
  #[error("execution failed with exit code {0}")]
  HaltWithNonZeroExitCode(u32),
  #[error("syscall failed with status code {0}")]
  SyscallFailed(StatusCode),
  #[error("invalid memory access for opcode {0} and address {1}")]
  InvalidMemoryAccess(Opcode, u32),
  #[error("unimplemented syscall {0}")]
  UnsupportedSyscall(u32),
  #[error("out of gas")]
  OutOfGas(),
  #[error("breakpoint encountered")]
  Breakpoint(),
  #[error("got unimplemented as opcode")]
  Unimplemented(),
  #[error("symbol not found")]
  UnknownSymbol(),
}

fn assert_valid_memory_access(addr: u32, position: MemoryAccessPosition) {
  {
    match position {
      MemoryAccessPosition::Memory => {
        assert_eq!(addr % 4, 0, "addr is not aligned");
        assert!(addr > 40);
      }
      _ => {
        Register::from_u32(addr);
      }
    };
  }
}

impl<'host> Runtime<'host> {
  // Create a new runtime from a program and, optionally, a host.
  pub fn new(
    program: Program,
    host: Option<&'host mut dyn HostInterface>,
    opts: AthenaCoreOpts,
    context: Option<AthenaContext>,
  ) -> Self {
    // Create a shared reference to the program.
    let program = Arc::new(program);

    // If TRACE_FILE is set, initialize the trace buffer.
    let trace_buf = if let Ok(trace_file) = std::env::var("TRACE_FILE") {
      let file = File::create(trace_file).unwrap();
      Some(BufWriter::new(file))
    } else {
      None
    };

    // Determine the maximum number of cycles for any syscall.
    let syscall_map = default_syscall_map();
    let max_syscall_cycles = syscall_map
      .values()
      .map(|syscall| syscall.num_extra_cycles())
      .max()
      .unwrap_or(0);

    Self {
      context,
      state: ExecutionState::new(program.pc_start),
      program,
      host,
      cycle_tracker: HashMap::new(),
      io_buf: HashMap::new(),
      trace_buf,
      unconstrained: false,
      max_gas: opts.max_gas(),
      syscall_map,
      max_syscall_cycles,
    }
  }

  /// Recover runtime state from a program and existing execution state.
  pub fn recover(program: Program, state: ExecutionState, opts: AthenaCoreOpts) -> Self {
    let mut runtime = Self::new(program, None, opts, None);
    runtime.state = state;
    runtime
  }

  /// Get the current values of the registers.
  pub fn registers(&self) -> [u32; 32] {
    let mut registers = [0; 32];
    for i in 0..32 {
      let addr = Register::from_u32(i as u32) as u32;
      registers[i] = match self.state.memory.get(&addr) {
        Some(record) => *record,
        None => 0,
      };
    }
    registers
  }

  /// Get the current value of a register.
  pub fn register(&self, register: Register) -> u32 {
    let addr = register as u32;
    match self.state.memory.get(&addr) {
      Some(record) => *record,
      None => 0,
    }
  }

  /// Get the current value of a word.
  pub fn word(&self, addr: u32) -> u32 {
    match self.state.memory.get(&addr) {
      Some(record) => *record,
      None => 0,
    }
  }

  /// Get the current value of a byte.
  pub fn byte(&self, addr: u32) -> u8 {
    let word = self.word(addr - addr % 4);
    (word >> ((addr % 4) * 8)) as u8
  }

  /// Read a word from memory.
  pub fn mr(&mut self, addr: u32) -> u32 {
    // Get the memory entry.
    let entry = self.state.memory.entry(addr);

    // If it's the first time accessing this address, initialize previous values.
    match entry {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        // If addr has a specific value to be initialized with, use that, otherwise 0.
        let value = self.state.uninitialized_memory.remove(&addr).unwrap_or(0);
        *entry.insert(value)
      }
    }
  }

  /// Write a word to memory.
  pub fn mw(&mut self, addr: u32, value: u32) {
    // Get the memory record entry.
    let entry = self.state.memory.entry(addr);

    match entry {
      Entry::Occupied(mut entry) => {
        entry.insert(value);
      }
      Entry::Vacant(entry) => {
        // If addr has a specific value to be initialized with, use that, otherwise 0.
        self.state.uninitialized_memory.remove(&addr);
        entry.insert(value);
      }
    };
  }

  /// Read from memory, assuming that all addresses are aligned.
  pub fn mr_cpu(&mut self, addr: u32, position: MemoryAccessPosition) -> u32 {
    // Assert that the address is aligned.
    assert_valid_memory_access(addr, position);

    // Read the address from memory and create a memory read record.
    self.mr(addr)
  }

  /// Write to memory.
  pub fn mw_cpu(&mut self, addr: u32, value: u32, position: MemoryAccessPosition) {
    // Assert that the address is aligned.
    assert_valid_memory_access(addr, position);

    // Read the address from memory and create a memory read record.
    self.mw(addr, value);
  }

  /// Read from a register.
  pub fn rr(&mut self, register: Register, position: MemoryAccessPosition) -> u32 {
    self.mr_cpu(register as u32, position)
  }

  /// Write to a register.
  pub fn rw(&mut self, register: Register, value: u32) {
    // The only time we are writing to a register is when it is in operand A.
    // Register %x0 should always be 0. See 2.6 Load and Store Instruction on
    // P.18 of the RISC-V spec. We always write 0 to %x0.
    if register == Register::X0 {
      self.mw_cpu(register as u32, 0, MemoryAccessPosition::A);
    } else {
      self.mw_cpu(register as u32, value, MemoryAccessPosition::A)
    }
  }

  /// Fetch the destination register and input operand values for an ALU instruction.
  fn alu_rr(&mut self, instruction: Instruction) -> (Register, u32, u32) {
    if !instruction.imm_c {
      let (rd, rs1, rs2) = instruction.r_type();
      let c = self.rr(rs2, MemoryAccessPosition::C);
      let b = self.rr(rs1, MemoryAccessPosition::B);
      (rd, b, c)
    } else if !instruction.imm_b && instruction.imm_c {
      let (rd, rs1, imm) = instruction.i_type();
      let (rd, b, c) = (rd, self.rr(rs1, MemoryAccessPosition::B), imm);
      (rd, b, c)
    } else {
      assert!(instruction.imm_b && instruction.imm_c);
      let (rd, b, c) = (
        Register::from_u32(instruction.op_a),
        instruction.op_b,
        instruction.op_c,
      );
      (rd, b, c)
    }
  }

  /// Set the destination register with the result and emit an ALU event.
  fn alu_rw(&mut self, _instruction: Instruction, rd: Register, a: u32, _b: u32, _c: u32) {
    self.rw(rd, a);
  }

  /// Fetch the input operand values for a load instruction.
  fn load_rr(&mut self, instruction: Instruction) -> (Register, u32, u32, u32, u32) {
    let (rd, rs1, imm) = instruction.i_type();
    let (b, c) = (self.rr(rs1, MemoryAccessPosition::B), imm);
    let addr = b.wrapping_add(c);
    let memory_value = self.mr_cpu(align(addr), MemoryAccessPosition::Memory);
    (rd, b, c, addr, memory_value)
  }

  /// Fetch the input operand values for a store instruction.
  fn store_rr(&mut self, instruction: Instruction) -> (u32, u32, u32, u32, u32) {
    let (rs1, rs2, imm) = instruction.s_type();
    let c = imm;
    let b = self.rr(rs2, MemoryAccessPosition::B);
    let a = self.rr(rs1, MemoryAccessPosition::A);
    let addr = b.wrapping_add(c);
    let memory_value = self.word(align(addr));
    (a, b, c, addr, memory_value)
  }

  /// Fetch the input operand values for a branch instruction.
  fn branch_rr(&mut self, instruction: Instruction) -> (u32, u32, u32) {
    let (rs1, rs2, imm) = instruction.b_type();
    let c = imm;
    let b = self.rr(rs2, MemoryAccessPosition::B);
    let a = self.rr(rs1, MemoryAccessPosition::A);
    (a, b, c)
  }

  /// Fetch the instruction at the current program counter.
  fn fetch(&self) -> Instruction {
    let idx = ((self.state.pc - self.program.pc_base) / 4) as usize;
    self.program.instructions[idx]
  }

  /// Execute the given instruction over the current state of the runtime.
  fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), ExecutionError> {
    let mut next_pc = self.state.pc.wrapping_add(4);

    let rd: Register;
    let (a, b, c): (u32, u32, u32);
    let (addr, memory_read_value): (u32, u32);

    match instruction.opcode {
      // Arithmetic instructions.
      Opcode::ADD => {
        (rd, b, c) = self.alu_rr(instruction);
        a = b.wrapping_add(c);
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::SUB => {
        (rd, b, c) = self.alu_rr(instruction);
        a = b.wrapping_sub(c);
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::XOR => {
        (rd, b, c) = self.alu_rr(instruction);
        a = b ^ c;
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::OR => {
        (rd, b, c) = self.alu_rr(instruction);
        a = b | c;
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::AND => {
        (rd, b, c) = self.alu_rr(instruction);
        a = b & c;
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::SLL => {
        (rd, b, c) = self.alu_rr(instruction);
        a = b.wrapping_shl(c);
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::SRL => {
        (rd, b, c) = self.alu_rr(instruction);
        a = b.wrapping_shr(c);
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::SRA => {
        (rd, b, c) = self.alu_rr(instruction);
        a = (b as i32).wrapping_shr(c) as u32;
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::SLT => {
        (rd, b, c) = self.alu_rr(instruction);
        a = if (b as i32) < (c as i32) { 1 } else { 0 };
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::SLTU => {
        (rd, b, c) = self.alu_rr(instruction);
        a = if b < c { 1 } else { 0 };
        self.alu_rw(instruction, rd, a, b, c);
      }

      // Load instructions.
      Opcode::LB => {
        (rd, _, _, addr, memory_read_value) = self.load_rr(instruction);
        let value = (memory_read_value).to_le_bytes()[(addr % 4) as usize];
        a = ((value as i8) as i32) as u32;
        self.rw(rd, a);
      }
      Opcode::LH => {
        (rd, _, _, addr, memory_read_value) = self.load_rr(instruction);
        if addr % 2 != 0 {
          return Err(ExecutionError::InvalidMemoryAccess(Opcode::LH, addr));
        }
        let value = match (addr >> 1) % 2 {
          0 => memory_read_value & 0x0000FFFF,
          1 => (memory_read_value & 0xFFFF0000) >> 16,
          _ => unreachable!(),
        };
        a = ((value as i16) as i32) as u32;
        self.rw(rd, a);
      }
      Opcode::LW => {
        (rd, _, _, addr, memory_read_value) = self.load_rr(instruction);
        if addr % 4 != 0 {
          return Err(ExecutionError::InvalidMemoryAccess(Opcode::LW, addr));
        }
        a = memory_read_value;
        self.rw(rd, a);
      }
      Opcode::LBU => {
        (rd, _, _, addr, memory_read_value) = self.load_rr(instruction);
        let value = (memory_read_value).to_le_bytes()[(addr % 4) as usize];
        a = value as u32;
        self.rw(rd, a);
      }
      Opcode::LHU => {
        (rd, _, _, addr, memory_read_value) = self.load_rr(instruction);
        if addr % 2 != 0 {
          return Err(ExecutionError::InvalidMemoryAccess(Opcode::LHU, addr));
        }
        let value = match (addr >> 1) % 2 {
          0 => memory_read_value & 0x0000FFFF,
          1 => (memory_read_value & 0xFFFF0000) >> 16,
          _ => unreachable!(),
        };
        a = (value as u16) as u32;
        self.rw(rd, a);
      }

      // Store instructions.
      Opcode::SB => {
        (a, _, _, addr, memory_read_value) = self.store_rr(instruction);
        let value = match addr % 4 {
          0 => (a & 0x000000FF) + (memory_read_value & 0xFFFFFF00),
          1 => ((a & 0x000000FF) << 8) + (memory_read_value & 0xFFFF00FF),
          2 => ((a & 0x000000FF) << 16) + (memory_read_value & 0xFF00FFFF),
          3 => ((a & 0x000000FF) << 24) + (memory_read_value & 0x00FFFFFF),
          _ => unreachable!(),
        };
        self.mw_cpu(align(addr), value, MemoryAccessPosition::Memory);
      }
      Opcode::SH => {
        (a, _, _, addr, memory_read_value) = self.store_rr(instruction);
        if addr % 2 != 0 {
          return Err(ExecutionError::InvalidMemoryAccess(Opcode::SH, addr));
        }
        let value = match (addr >> 1) % 2 {
          0 => (a & 0x0000FFFF) + (memory_read_value & 0xFFFF0000),
          1 => ((a & 0x0000FFFF) << 16) + (memory_read_value & 0x0000FFFF),
          _ => unreachable!(),
        };
        self.mw_cpu(align(addr), value, MemoryAccessPosition::Memory);
      }
      Opcode::SW => {
        (a, _, _, addr, _) = self.store_rr(instruction);
        if addr % 4 != 0 {
          return Err(ExecutionError::InvalidMemoryAccess(Opcode::SW, addr));
        }
        let value = a;
        self.mw_cpu(align(addr), value, MemoryAccessPosition::Memory);
      }

      // B-type instructions.
      Opcode::BEQ => {
        (a, b, c) = self.branch_rr(instruction);
        if a == b {
          next_pc = self.state.pc.wrapping_add(c);
        }
      }
      Opcode::BNE => {
        (a, b, c) = self.branch_rr(instruction);
        if a != b {
          next_pc = self.state.pc.wrapping_add(c);
        }
      }
      Opcode::BLT => {
        (a, b, c) = self.branch_rr(instruction);
        if (a as i32) < (b as i32) {
          next_pc = self.state.pc.wrapping_add(c);
        }
      }
      Opcode::BGE => {
        (a, b, c) = self.branch_rr(instruction);
        if (a as i32) >= (b as i32) {
          next_pc = self.state.pc.wrapping_add(c);
        }
      }
      Opcode::BLTU => {
        (a, b, c) = self.branch_rr(instruction);
        if a < b {
          next_pc = self.state.pc.wrapping_add(c);
        }
      }
      Opcode::BGEU => {
        (a, b, c) = self.branch_rr(instruction);
        if a >= b {
          next_pc = self.state.pc.wrapping_add(c);
        }
      }

      // Jump instructions.
      Opcode::JAL => {
        let (rd, imm) = instruction.j_type();
        a = self.state.pc + 4;
        self.rw(rd, a);
        next_pc = self.state.pc.wrapping_add(imm);
      }
      Opcode::JALR => {
        let (rd, rs1, imm) = instruction.i_type();
        (b, c) = (self.rr(rs1, MemoryAccessPosition::B), imm);
        a = self.state.pc + 4;
        self.rw(rd, a);
        next_pc = b.wrapping_add(c);
      }

      // Upper immediate instructions.
      Opcode::AUIPC => {
        let (rd, imm) = instruction.u_type();
        a = self.state.pc.wrapping_add(imm);
        self.rw(rd, a);
      }

      // System instructions.
      Opcode::ECALL => {
        // We peek at register x5 to get the syscall id. The reason we don't `self.rr` this
        // register is that we write to it later.
        let t0 = Register::X5;
        let syscall_id = self.register(t0);
        c = self.rr(Register::X11, MemoryAccessPosition::C);
        b = self.rr(Register::X10, MemoryAccessPosition::B);
        let syscall = SyscallCode::from_u32(syscall_id);

        let syscall_impl = self
          .get_syscall(syscall)
          .ok_or(ExecutionError::UnsupportedSyscall(syscall_id))?
          .clone();
        let mut ctx = SyscallContext::new(self);

        // Executing a syscall optionally returns a value to write to the t0 register.
        // If it returns None, we just keep the syscall_id in t0.
        let result = syscall_impl
          .execute(&mut ctx, b, c)
          .map_err(ExecutionError::SyscallFailed)?;

        match result {
          Outcome::Result(value) => {
            if let Some(value) = value {
              self.rw(t0, value);
            }
            next_pc = self.state.pc.wrapping_add(4);
          }
          Outcome::Exit(0) => {
            next_pc = 0;
          }
          Outcome::Exit(code) => {
            return Err(ExecutionError::HaltWithNonZeroExitCode(code));
          }
        };

        // Allow the syscall impl to modify state.clk/pc (exit unconstrained does this)
        self.state.clk += syscall_impl.num_extra_cycles();
      }
      Opcode::EBREAK => {
        return Err(ExecutionError::Breakpoint());
      }

      // Multiply instructions.
      Opcode::MUL => {
        (rd, b, c) = self.alu_rr(instruction);
        a = b.wrapping_mul(c);
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::MULH => {
        (rd, b, c) = self.alu_rr(instruction);
        a = (((b as i32) as i64).wrapping_mul((c as i32) as i64) >> 32) as u32;
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::MULHU => {
        (rd, b, c) = self.alu_rr(instruction);
        a = ((b as u64).wrapping_mul(c as u64) >> 32) as u32;
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::MULHSU => {
        (rd, b, c) = self.alu_rr(instruction);
        a = (((b as i32) as i64).wrapping_mul(c as i64) >> 32) as u32;
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::DIV => {
        (rd, b, c) = self.alu_rr(instruction);
        if c == 0 {
          a = u32::MAX;
        } else {
          a = (b as i32).wrapping_div(c as i32) as u32;
        }
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::DIVU => {
        (rd, b, c) = self.alu_rr(instruction);
        if c == 0 {
          a = u32::MAX;
        } else {
          a = b.wrapping_div(c);
        }
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::REM => {
        (rd, b, c) = self.alu_rr(instruction);
        if c == 0 {
          a = b;
        } else {
          a = (b as i32).wrapping_rem(c as i32) as u32;
        }
        self.alu_rw(instruction, rd, a, b, c);
      }
      Opcode::REMU => {
        (rd, b, c) = self.alu_rr(instruction);
        if c == 0 {
          a = b;
        } else {
          a = b.wrapping_rem(c);
        }
        self.alu_rw(instruction, rd, a, b, c);
      }

      // See https://github.com/riscv-non-isa/riscv-asm-manual/blob/master/riscv-asm.md#instruction-aliases
      Opcode::UNIMP => {
        return Err(ExecutionError::Unimplemented());
      }
    }

    // Update the program counter.
    self.state.pc = next_pc;

    // Update the clk to the next cycle.
    self.state.clk += 4;

    Ok(())
  }

  /// Executes one cycle of the program, returning whether the program has finished.
  #[inline]
  fn execute_cycle(&mut self) -> Result<bool, ExecutionError> {
    // Fetch the instruction at the current program counter.
    let instruction = self.fetch();

    tracing::debug!(instruction = ?instruction, "executing cycle");

    // Log the current state of the runtime.
    self.log(&instruction);

    // Execute the instruction.
    self.execute_instruction(instruction)?;

    // Increment the clock.
    self.state.global_clk += 1;

    // We're allowed to spend all of our gas, but no more.
    // Gas checking is "lazy" here: it happens _after_ the instruction is executed.
    if let Some(gas_left) = self.gas_left() {
      if !self.unconstrained && gas_left < 0 {
        return Err(ExecutionError::OutOfGas());
      }
    }

    Ok(
      self.state.pc.wrapping_sub(self.program.pc_base)
        >= (self.program.instructions.len() * 4) as u32,
    )
  }

  pub(crate) fn gas_left(&self) -> Option<i64> {
    // gas left can be negative, if we spent too much on the last instruction
    self
      .max_gas
      .map(|max_gas| max_gas as i64 - self.state.clk as i64)
  }

  fn initialize(&mut self) {
    self.state.clk = 0;

    // TODO: do we want to load all of the memory when executing a particular function?
    tracing::info!("loading memory image");
    for (addr, value) in self.program.memory_image.iter() {
      self.state.memory.insert(*addr, *value);
    }

    tracing::info!("starting execution");
  }

  /// Execute an exported function. Does the same work as execute().
  pub fn execute_function(&mut self, symbol_name: &str) -> Result<Option<u32>, ExecutionError> {
    // Make sure the symbol exists, and set the program counter
    let offset = match self.program.symbol_table.get(symbol_name) {
      Some(offset) => *offset,
      None => return Err(ExecutionError::UnknownSymbol()),
    };
    self.state.pc = offset;

    // Hand over to execute
    self.execute()
  }

  /// Execute the program, returning remaining gas. Execution will either complete or produce an error.
  pub fn execute(&mut self) -> Result<Option<u32>, ExecutionError> {
    // If it's the first cycle, initialize the program.
    if self.state.global_clk == 0 {
      tracing::info!("initializing");
      self.initialize();
    }

    // Loop until program finishes execution or until an error occurs, whichever comes first
    loop {
      if self.execute_cycle()? {
        break;
      }
    }
    tracing::info!("execution finished");

    self.postprocess();

    // Calculate remaining gas. If we spent too much gas, an error would already have been thrown and
    // we would never reach this code, hence the assertion.
    Ok(
      self
        .gas_left()
        .map(|gas_left| u32::try_from(gas_left).expect("Gas conversion error")),
    )
  }

  fn postprocess(&mut self) {
    tracing::info!(
      "finished execution clk = {} global_clk = {} pc = 0x{:x?}",
      self.state.clk,
      self.state.global_clk,
      self.state.pc
    );
    // Flush remaining stdout/stderr
    for (fd, buf) in self.io_buf.iter() {
      if !buf.is_empty() {
        match fd {
          1 => {
            println!("stdout: {}", buf);
          }
          2 => {
            println!("stderr: {}", buf);
          }
          _ => {}
        }
      }
    }

    // Flush trace buf
    if let Some(ref mut buf) = self.trace_buf {
      buf.flush().unwrap();
    }
  }

  fn get_syscall(&mut self, code: SyscallCode) -> Option<&Arc<dyn Syscall>> {
    self.syscall_map.get(&code)
  }
}

#[cfg(test)]
pub mod tests {

  use crate::{
    io::AthenaStdin,
    runtime::{ExecutionError, MemoryAccessPosition},
    utils::with_max_gas,
  };
  use athena_interface::{
    calculate_address, Address, AthenaContext, HostDynamicContext, HostInterface,
    HostStaticContext, MockHost, StatusCode, ADDRESS_ALICE, ADDRESS_BOB, ADDRESS_CHARLIE,
    ADDRESS_LENGTH, SOME_COINS,
  };
  use athena_vm::helpers::address_to_32bit_words;
  use athena_vm_sdk::SpendArguments;
  use parity_scale_codec::Encode;

  use crate::{
    runtime::Register,
    utils::{
      tests::{TEST_FIBONACCI_ELF, TEST_HOST, TEST_PANIC_ELF, WALLET_ELF},
      AthenaCoreOpts,
    },
  };

  use super::syscall::SyscallCode;
  use super::{Instruction, Opcode, Program, Runtime};

  pub fn simple_program() -> Program {
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::ADD, 31, 30, 29, false, false),
    ];
    Program::new(instructions, 0, 0)
  }

  pub fn fibonacci_program() -> Program {
    Program::from(TEST_FIBONACCI_ELF)
  }

  pub fn panic_program() -> Program {
    Program::from(TEST_PANIC_ELF)
  }

  pub fn host_program() -> Program {
    Program::from(TEST_HOST)
  }

  pub fn wallet_program() -> Program {
    Program::from(WALLET_ELF)
  }

  pub(crate) fn setup_logger() {
    let _ = tracing_subscriber::fmt()
      .with_test_writer()
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .try_init();
  }

  #[test]
  fn test_simple_program_run() {
    let program = simple_program();
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 42);
  }

  #[test]
  #[should_panic]
  fn test_panic() {
    let program = panic_program();
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
  }

  #[test]
  fn test_wallet() {
    let program = wallet_program();

    // set up host context
    let owner_pubkey = [1u8; 32];
    let principal = ADDRESS_ALICE;
    let template = ADDRESS_BOB;
    let callee = ADDRESS_CHARLIE;
    // arbitrary template address
    let wallet_template = [9u8; ADDRESS_LENGTH];
    let nonce = 0;
    let static_context = HostStaticContext::new(principal, nonce, Address::default());
    let dynamic_context = HostDynamicContext::new(template, callee);
    let mut host = MockHost::new_with_context(static_context, dynamic_context);

    // check balances: only alice has coins
    let amount_to_send = 1000;
    assert_eq!(host.get_balance(&ADDRESS_ALICE), SOME_COINS);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 0);

    // set up send arguments
    let send_args = SpendArguments {
      amount: amount_to_send,
      recipient: ADDRESS_CHARLIE,
    };

    // alice initiates the call. initially she calls the wallet template directly.
    let ctx = AthenaContext::new(wallet_template, ADDRESS_ALICE, 0);
    let opts = AthenaCoreOpts::default().with_options(vec![with_max_gas(100000)]);
    let mut runtime = Runtime::new(program.clone(), Some(&mut host), opts, Some(ctx));
    let mut stdin = AthenaStdin::new();
    stdin.write(&owner_pubkey);
    runtime.write_vecs(&stdin.buffer);

    // make sure the program loaded correctly
    // riscv32-unknown-linux-gnu-objdump -d -j .text elf/wallet-template | grep athexp
    assert_eq!(
      runtime
        .program
        .as_ref()
        .symbol_table
        .get("athexp_spawn")
        .unwrap(),
      &2106680
    );
    assert_eq!(
      runtime
        .program
        .as_ref()
        .symbol_table
        .get("athexp_spend")
        .unwrap(),
      &2106732
    );

    // now attempt to execute each function in turn
    // first, the spawn
    runtime.execute_function("athexp_spawn").unwrap();
    drop(runtime);

    // get newly-created wallet address
    let spawn_result = host.get_spawn_result().unwrap().clone();
    assert_eq!(principal, spawn_result.principal);
    assert_eq!(template, spawn_result.template);
    assert_eq!(nonce, spawn_result.nonce);
    assert_eq!(
      calculate_address(&template, &spawn_result.blob, &principal, nonce),
      spawn_result.address
    );
    assert_eq!(
      host.get_program(&spawn_result.address).unwrap(),
      &spawn_result.blob
    );

    // check balances again
    // note: Alice hasn't paid for gas since we're not charging for gas yet
    assert_eq!(host.get_balance(&ADDRESS_ALICE), SOME_COINS);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 0);
    assert_eq!(host.get_balance(&spawn_result.address), 0);

    // write the input: serialized state blob, then serialized send args
    let mut stdin = AthenaStdin::new();
    let ctx = AthenaContext::new(wallet_template, ADDRESS_ALICE, 0);
    let mut runtime = Runtime::new(program.clone(), Some(&mut host), opts, Some(ctx));
    stdin.write_vec(spawn_result.blob.clone());
    stdin.write_vec(send_args.encode());
    runtime.write_vecs(&stdin.buffer);

    // now attempt the send
    let res = runtime.execute_function("athexp_spend");
    match res {
      Ok(_) => panic!("expected execution error"),
      Err(e) => match e {
        ExecutionError::SyscallFailed(status) => {
          assert_eq!(status, StatusCode::InsufficientBalance);
        }
        _ => panic!("expected SyscallFailed error"),
      },
    }
    drop(runtime);

    // transfer some coins to the new wallet
    let address = spawn_result.address;
    host.transfer_balance(&ADDRESS_ALICE, &address, amount_to_send);
    assert_eq!(
      host.get_balance(&ADDRESS_ALICE),
      SOME_COINS - amount_to_send
    );
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 0);
    assert_eq!(host.get_balance(&address), amount_to_send);

    // set up the runtime again
    let mut stdin = AthenaStdin::new();
    let ctx = AthenaContext::new(spawn_result.address, ADDRESS_ALICE, 0);
    let mut runtime = Runtime::new(program.clone(), Some(&mut host), opts, Some(ctx));
    stdin.write_vec(spawn_result.blob);
    stdin.write_vec(send_args.encode());
    runtime.write_vecs(&stdin.buffer);

    // do the send again
    runtime.execute_function("athexp_spend").unwrap();

    // final balance check: some of alice's coins were sent to Charlie
    assert_eq!(
      host.get_balance(&ADDRESS_ALICE),
      SOME_COINS - amount_to_send
    );
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), amount_to_send);
    assert_eq!(host.get_balance(&address), 0);
  }

  #[test]
  fn test_host() {
    setup_logger();
    let program = host_program();
    let ctx = AthenaContext::new(ADDRESS_ALICE, Address::default(), 0);
    let opts = AthenaCoreOpts::default().with_options(vec![with_max_gas(100000)]);
    let mut host = MockHost::new();
    let mut runtime = Runtime::new(program, Some(&mut host), opts, Some(ctx));
    let gas_left = runtime.execute().unwrap();

    // don't bother checking exact gas value, that's checked in the following test
    assert!(gas_left.is_some());
  }

  #[test]
  fn test_gas() {
    let program = fibonacci_program();
    let ctx = AthenaContext::new(Address::default(), Address::default(), 0);

    // failure
    let mut runtime = Runtime::new(
      program.clone(),
      None,
      AthenaCoreOpts::default().with_options(vec![with_max_gas(543)]),
      Some(ctx.clone()),
    );
    assert!(matches!(runtime.execute(), Err(ExecutionError::OutOfGas())));

    // success
    let mut runtime = Runtime::new(
      program.clone(),
      None,
      AthenaCoreOpts::default().with_options(vec![with_max_gas(568)]),
      Some(ctx.clone()),
    );
    let gas_left = runtime.execute().unwrap();
    assert_eq!(gas_left, Some(0));

    // success
    let mut runtime = Runtime::new(
      program.clone(),
      None,
      AthenaCoreOpts::default().with_options(vec![with_max_gas(569)]),
      Some(ctx.clone()),
    );
    let gas_left = runtime.execute().unwrap();
    assert_eq!(gas_left, Some(1));
  }

  #[test]
  fn test_call_send() {
    setup_logger();

    // recipient address
    let address_words = address_to_32bit_words(ADDRESS_CHARLIE);

    let amount_to_send = 1000;

    // arbitrary memory locations
    let memloc: u32 = 0x12345678;
    let memloc2 = memloc.wrapping_add(address_words.len() as u32 * 4);

    let mut instructions = vec![];

    // write address to memory
    for (i, word) in (0u32..).zip(address_words.iter()) {
      instructions.push(Instruction::new(
        Opcode::ADD,
        Register::X16 as u32,
        0,
        *word,
        false,
        true,
      ));
      instructions.push(Instruction::new(
        Opcode::SW,
        Register::X16 as u32,
        0,
        memloc + i * 4,
        false,
        true,
      ));
    }

    // write value to memory
    instructions.push(Instruction::new(
      Opcode::ADD,
      Register::X16 as u32,
      0,
      amount_to_send,
      false,
      true,
    ));
    instructions.push(Instruction::new(
      Opcode::SW,
      Register::X16 as u32,
      0,
      memloc2,
      false,
      true,
    ));
    instructions.push(Instruction::new(
      Opcode::ADD,
      Register::X16 as u32,
      0,
      0,
      false,
      true,
    ));
    instructions.push(Instruction::new(
      Opcode::SW,
      Register::X16 as u32,
      0,
      memloc2 + 4,
      false,
      true,
    ));

    // X10 is arg1 (ptr to address)
    instructions.push(Instruction::new(
      Opcode::ADD,
      Register::X10 as u32,
      0,
      memloc,
      false,
      true,
    ));
    instructions.push(
      // X11 is arg2 (ptr to input)
      // zero pointer
      Instruction::new(Opcode::ADD, Register::X11 as u32, 0, 0, false, true),
    );
    instructions.push(
      // X12 is arg3 (input len)
      // no input
      Instruction::new(Opcode::ADD, Register::X12 as u32, 0, 0, false, true),
    );
    instructions.push(
      // X13 is arg4 (ptr to method name)
      // zero pointer
      Instruction::new(Opcode::ADD, Register::X13 as u32, 0, 0, false, true),
    );
    instructions.push(
      // X14 is arg5 (method name len)
      // no input
      Instruction::new(Opcode::ADD, Register::X14 as u32, 0, 0, false, true),
    );
    instructions.push(
      // X15 is arg6 (value ptr)
      Instruction::new(Opcode::ADD, Register::X15 as u32, 0, memloc2, false, true),
    );
    instructions.push(
      // X5 is syscall ID
      Instruction::new(
        Opcode::ADD,
        Register::X5 as u32,
        0,
        SyscallCode::HOST_CALL as u32,
        false,
        true,
      ),
    );
    instructions.push(Instruction::new(Opcode::ECALL, 0, 0, 0, false, false));
    let program = Program::new(instructions, 0, 0);

    let mut host = MockHost::new();
    let ctx = AthenaContext::new(ADDRESS_ALICE, Address::default(), 0);
    let opts = AthenaCoreOpts::default().with_options(vec![with_max_gas(100000)]);
    // balances before execution
    assert_eq!(host.get_balance(&ADDRESS_ALICE), SOME_COINS);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 0);

    let mut runtime = Runtime::new(program, Some(&mut host), opts, Some(ctx));
    assert!(runtime.execute().unwrap().is_some());

    // balances after execution
    assert_eq!(
      host.get_balance(&ADDRESS_ALICE),
      SOME_COINS - amount_to_send as u64
    );
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), amount_to_send as u64);
  }

  #[test]
  fn test_bad_call() {
    setup_logger();

    let instructions = vec![
      // X10 is arg1 (ptr to address)
      // memory location to store an address
      Instruction::new(
        Opcode::ADD,
        Register::X10 as u32,
        0,
        0x12345678,
        false,
        true,
      ),
      // store arbitrary address here
      Instruction::new(Opcode::SW, Register::X10 as u32, 0, 0x12345678, false, true),
      // X11 is arg2 (ptr to input)
      // zero pointer
      Instruction::new(Opcode::ADD, Register::X11 as u32, 0, 0, false, true),
      // X12 is arg3 (input len)
      // no input
      Instruction::new(Opcode::ADD, Register::X12 as u32, 0, 0, false, true),
      // X5 is syscall ID
      Instruction::new(
        Opcode::ADD,
        Register::X5 as u32,
        0,
        SyscallCode::HOST_CALL as u32,
        false,
        true,
      ),
      Instruction::new(Opcode::ECALL, 0, 0, 0, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let ctx = AthenaContext::new(Address::default(), Address::default(), 0);
    let opts = AthenaCoreOpts::default().with_options(vec![with_max_gas(100000)]);
    let mut host = MockHost::new();
    let mut runtime = Runtime::new(program, Some(&mut host), opts, Some(ctx));
    match runtime.execute() {
      Err(e) => match e {
        ExecutionError::SyscallFailed(code) => {
          assert_eq!(code, athena_interface::StatusCode::Failure);
        }
        _ => panic!("unexpected error: {:?}", e),
      },
      Ok(_) => panic!("expected error, got Ok"),
    }
  }

  #[test]
  fn test_get_balance() {
    setup_logger();

    // arbitrary memory location
    let memloc = 0x12345678;

    let instructions = vec![
      // X10 is arg1 (ptr to address)
      // store result here
      Instruction::new(Opcode::ADD, Register::X10 as u32, 0, memloc, false, true),
      // X5 is syscall ID
      Instruction::new(
        Opcode::ADD,
        Register::X5 as u32,
        0,
        SyscallCode::HOST_GETBALANCE as u32,
        false,
        true,
      ),
      Instruction::new(Opcode::ECALL, 0, 0, 0, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let ctx = AthenaContext::new(ADDRESS_ALICE, Address::default(), 0);
    let opts = AthenaCoreOpts::default().with_options(vec![with_max_gas(100000)]);
    let mut host = MockHost::new();
    let mut runtime = Runtime::new(program, Some(&mut host), opts, Some(ctx));
    let gas_left = runtime.execute().expect("execution failed");
    assert!(gas_left.is_some());

    // check result: we expect the u64 value to be split into two 32-bit words
    let value_low = runtime.mr_cpu(memloc, MemoryAccessPosition::Memory);
    let value_high = runtime.mr_cpu(memloc + 4, MemoryAccessPosition::Memory);
    assert_eq!(
      u64::from(value_high) << 32 | u64::from(value_low),
      SOME_COINS
    );
  }

  #[test]
  fn test_syscall_fail() {
    let instructions = vec![
      Instruction::new(
        Opcode::ADD,
        Register::X5 as u32,
        0,
        SyscallCode::WRITE as u32,
        false,
        true,
      ),
      Instruction::new(Opcode::ECALL, 0, 0, 0, false, false),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, Default::default(), None);

    let mut syscall = super::MockSyscall::new();
    syscall
      .expect_execute()
      .returning(|_, _, _| Err(StatusCode::Rejected));
    runtime
      .syscall_map
      .insert(SyscallCode::WRITE, std::sync::Arc::new(syscall));

    let err = runtime.execute().unwrap_err();
    assert_eq!(err, ExecutionError::SyscallFailed(StatusCode::Rejected));
  }

  #[test]
  fn test_add() {
    // main:
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     add x31, x30, x29
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::ADD, 31, 30, 29, false, false),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 42);
  }

  #[test]
  fn test_sub() {
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     sub x31, x30, x29
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::SUB, 31, 30, 29, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 32);
  }

  #[test]
  fn test_xor() {
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     xor x31, x30, x29
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::XOR, 31, 30, 29, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 32);
  }

  #[test]
  fn test_or() {
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     or x31, x30, x29
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::OR, 31, 30, 29, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);

    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 37);
  }

  #[test]
  fn test_and() {
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     and x31, x30, x29
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::AND, 31, 30, 29, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 5);
  }

  #[test]
  fn test_sll() {
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     sll x31, x30, x29
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::SLL, 31, 30, 29, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 1184);
  }

  #[test]
  fn test_srl() {
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     srl x31, x30, x29
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::SRL, 31, 30, 29, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 1);
  }

  #[test]
  fn test_sra() {
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     sra x31, x30, x29
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::SRA, 31, 30, 29, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 1);
  }

  #[test]
  fn test_slt() {
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     slt x31, x30, x29
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::SLT, 31, 30, 29, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 0);
  }

  #[test]
  fn test_sltu() {
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     sltu x31, x30, x29
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
      Instruction::new(Opcode::SLTU, 31, 30, 29, false, false),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 0);
  }

  #[test]
  fn test_addi() {
    //     addi x29, x0, 5
    //     addi x30, x29, 37
    //     addi x31, x30, 42
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 29, 37, false, true),
      Instruction::new(Opcode::ADD, 31, 30, 42, false, true),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 84);
  }

  #[test]
  fn test_addi_negative() {
    //     addi x29, x0, 5
    //     addi x30, x29, -1
    //     addi x31, x30, 4
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::ADD, 30, 29, 0xffffffff, false, true),
      Instruction::new(Opcode::ADD, 31, 30, 4, false, true),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 5 - 1 + 4);
  }

  #[test]
  fn test_xori() {
    //     addi x29, x0, 5
    //     xori x30, x29, 37
    //     xori x31, x30, 42
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::XOR, 30, 29, 37, false, true),
      Instruction::new(Opcode::XOR, 31, 30, 42, false, true),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 10);
  }

  #[test]
  fn test_ori() {
    //     addi x29, x0, 5
    //     ori x30, x29, 37
    //     ori x31, x30, 42
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::OR, 30, 29, 37, false, true),
      Instruction::new(Opcode::OR, 31, 30, 42, false, true),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 47);
  }

  #[test]
  fn test_andi() {
    //     addi x29, x0, 5
    //     andi x30, x29, 37
    //     andi x31, x30, 42
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::AND, 30, 29, 37, false, true),
      Instruction::new(Opcode::AND, 31, 30, 42, false, true),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 0);
  }

  #[test]
  fn test_slli() {
    //     addi x29, x0, 5
    //     slli x31, x29, 37
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
      Instruction::new(Opcode::SLL, 31, 29, 4, false, true),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 80);
  }

  #[test]
  fn test_srli() {
    //    addi x29, x0, 5
    //    srli x31, x29, 37
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 42, false, true),
      Instruction::new(Opcode::SRL, 31, 29, 4, false, true),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 2);
  }

  #[test]
  fn test_srai() {
    //   addi x29, x0, 5
    //   srai x31, x29, 37
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 42, false, true),
      Instruction::new(Opcode::SRA, 31, 29, 4, false, true),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 2);
  }

  #[test]
  fn test_slti() {
    //   addi x29, x0, 5
    //   slti x31, x29, 37
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 42, false, true),
      Instruction::new(Opcode::SLT, 31, 29, 37, false, true),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 0);
  }

  #[test]
  fn test_sltiu() {
    //   addi x29, x0, 5
    //   sltiu x31, x29, 37
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 42, false, true),
      Instruction::new(Opcode::SLTU, 31, 29, 37, false, true),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X31), 0);
  }

  #[test]
  fn test_jalr() {
    //   addi x11, x11, 100
    //   jalr x5, x11, 8
    //
    // `JALR rd offset(rs)` reads the value at rs, adds offset to it and uses it as the
    // destination address. It then stores the address of the next instruction in rd in case
    // we'd want to come back here.

    let instructions = vec![
      Instruction::new(Opcode::ADD, 11, 11, 100, false, true),
      Instruction::new(Opcode::JALR, 5, 11, 8, false, true),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.registers()[Register::X5 as usize], 8);
    assert_eq!(runtime.registers()[Register::X11 as usize], 100);
    assert_eq!(runtime.state.pc, 108);
  }

  fn simple_op_code_test(opcode: Opcode, expected: u32, a: u32, b: u32) {
    let instructions = vec![
      Instruction::new(Opcode::ADD, 10, 0, a, false, true),
      Instruction::new(Opcode::ADD, 11, 0, b, false, true),
      Instruction::new(opcode, 12, 10, 11, false, false),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.registers()[Register::X12 as usize], expected);
  }

  #[test]
  fn multiplication_tests() {
    simple_op_code_test(Opcode::MULHU, 0x00000000, 0x00000000, 0x00000000);
    simple_op_code_test(Opcode::MULHU, 0x00000000, 0x00000001, 0x00000001);
    simple_op_code_test(Opcode::MULHU, 0x00000000, 0x00000003, 0x00000007);
    simple_op_code_test(Opcode::MULHU, 0x00000000, 0x00000000, 0xffff8000);
    simple_op_code_test(Opcode::MULHU, 0x00000000, 0x80000000, 0x00000000);
    simple_op_code_test(Opcode::MULHU, 0x7fffc000, 0x80000000, 0xffff8000);
    simple_op_code_test(Opcode::MULHU, 0x0001fefe, 0xaaaaaaab, 0x0002fe7d);
    simple_op_code_test(Opcode::MULHU, 0x0001fefe, 0x0002fe7d, 0xaaaaaaab);
    simple_op_code_test(Opcode::MULHU, 0xfe010000, 0xff000000, 0xff000000);
    simple_op_code_test(Opcode::MULHU, 0xfffffffe, 0xffffffff, 0xffffffff);
    simple_op_code_test(Opcode::MULHU, 0x00000000, 0xffffffff, 0x00000001);
    simple_op_code_test(Opcode::MULHU, 0x00000000, 0x00000001, 0xffffffff);

    simple_op_code_test(Opcode::MULHSU, 0x00000000, 0x00000000, 0x00000000);
    simple_op_code_test(Opcode::MULHSU, 0x00000000, 0x00000001, 0x00000001);
    simple_op_code_test(Opcode::MULHSU, 0x00000000, 0x00000003, 0x00000007);
    simple_op_code_test(Opcode::MULHSU, 0x00000000, 0x00000000, 0xffff8000);
    simple_op_code_test(Opcode::MULHSU, 0x00000000, 0x80000000, 0x00000000);
    simple_op_code_test(Opcode::MULHSU, 0x80004000, 0x80000000, 0xffff8000);
    simple_op_code_test(Opcode::MULHSU, 0xffff0081, 0xaaaaaaab, 0x0002fe7d);
    simple_op_code_test(Opcode::MULHSU, 0x0001fefe, 0x0002fe7d, 0xaaaaaaab);
    simple_op_code_test(Opcode::MULHSU, 0xff010000, 0xff000000, 0xff000000);
    simple_op_code_test(Opcode::MULHSU, 0xffffffff, 0xffffffff, 0xffffffff);
    simple_op_code_test(Opcode::MULHSU, 0xffffffff, 0xffffffff, 0x00000001);
    simple_op_code_test(Opcode::MULHSU, 0x00000000, 0x00000001, 0xffffffff);

    simple_op_code_test(Opcode::MULH, 0x00000000, 0x00000000, 0x00000000);
    simple_op_code_test(Opcode::MULH, 0x00000000, 0x00000001, 0x00000001);
    simple_op_code_test(Opcode::MULH, 0x00000000, 0x00000003, 0x00000007);
    simple_op_code_test(Opcode::MULH, 0x00000000, 0x00000000, 0xffff8000);
    simple_op_code_test(Opcode::MULH, 0x00000000, 0x80000000, 0x00000000);
    simple_op_code_test(Opcode::MULH, 0x00000000, 0x80000000, 0x00000000);
    simple_op_code_test(Opcode::MULH, 0xffff0081, 0xaaaaaaab, 0x0002fe7d);
    simple_op_code_test(Opcode::MULH, 0xffff0081, 0x0002fe7d, 0xaaaaaaab);
    simple_op_code_test(Opcode::MULH, 0x00010000, 0xff000000, 0xff000000);
    simple_op_code_test(Opcode::MULH, 0x00000000, 0xffffffff, 0xffffffff);
    simple_op_code_test(Opcode::MULH, 0xffffffff, 0xffffffff, 0x00000001);
    simple_op_code_test(Opcode::MULH, 0xffffffff, 0x00000001, 0xffffffff);

    simple_op_code_test(Opcode::MUL, 0x00001200, 0x00007e00, 0xb6db6db7);
    simple_op_code_test(Opcode::MUL, 0x00001240, 0x00007fc0, 0xb6db6db7);
    simple_op_code_test(Opcode::MUL, 0x00000000, 0x00000000, 0x00000000);
    simple_op_code_test(Opcode::MUL, 0x00000001, 0x00000001, 0x00000001);
    simple_op_code_test(Opcode::MUL, 0x00000015, 0x00000003, 0x00000007);
    simple_op_code_test(Opcode::MUL, 0x00000000, 0x00000000, 0xffff8000);
    simple_op_code_test(Opcode::MUL, 0x00000000, 0x80000000, 0x00000000);
    simple_op_code_test(Opcode::MUL, 0x00000000, 0x80000000, 0xffff8000);
    simple_op_code_test(Opcode::MUL, 0x0000ff7f, 0xaaaaaaab, 0x0002fe7d);
    simple_op_code_test(Opcode::MUL, 0x0000ff7f, 0x0002fe7d, 0xaaaaaaab);
    simple_op_code_test(Opcode::MUL, 0x00000000, 0xff000000, 0xff000000);
    simple_op_code_test(Opcode::MUL, 0x00000001, 0xffffffff, 0xffffffff);
    simple_op_code_test(Opcode::MUL, 0xffffffff, 0xffffffff, 0x00000001);
    simple_op_code_test(Opcode::MUL, 0xffffffff, 0x00000001, 0xffffffff);
  }

  fn neg(a: u32) -> u32 {
    u32::MAX - a + 1
  }

  #[test]
  fn division_tests() {
    simple_op_code_test(Opcode::DIVU, 3, 20, 6);
    simple_op_code_test(Opcode::DIVU, 715827879, u32::MAX - 20 + 1, 6);
    simple_op_code_test(Opcode::DIVU, 0, 20, u32::MAX - 6 + 1);
    simple_op_code_test(Opcode::DIVU, 0, u32::MAX - 20 + 1, u32::MAX - 6 + 1);

    simple_op_code_test(Opcode::DIVU, 1 << 31, 1 << 31, 1);
    simple_op_code_test(Opcode::DIVU, 0, 1 << 31, u32::MAX - 1 + 1);

    simple_op_code_test(Opcode::DIVU, u32::MAX, 1 << 31, 0);
    simple_op_code_test(Opcode::DIVU, u32::MAX, 1, 0);
    simple_op_code_test(Opcode::DIVU, u32::MAX, 0, 0);

    simple_op_code_test(Opcode::DIV, 3, 18, 6);
    simple_op_code_test(Opcode::DIV, neg(6), neg(24), 4);
    simple_op_code_test(Opcode::DIV, neg(2), 16, neg(8));
    simple_op_code_test(Opcode::DIV, neg(1), 0, 0);

    // Overflow cases
    simple_op_code_test(Opcode::DIV, 1 << 31, 1 << 31, neg(1));
    simple_op_code_test(Opcode::REM, 0, 1 << 31, neg(1));
  }

  #[test]
  fn remainder_tests() {
    simple_op_code_test(Opcode::REM, 7, 16, 9);
    simple_op_code_test(Opcode::REM, neg(4), neg(22), 6);
    simple_op_code_test(Opcode::REM, 1, 25, neg(3));
    simple_op_code_test(Opcode::REM, neg(2), neg(22), neg(4));
    simple_op_code_test(Opcode::REM, 0, 873, 1);
    simple_op_code_test(Opcode::REM, 0, 873, neg(1));
    simple_op_code_test(Opcode::REM, 5, 5, 0);
    simple_op_code_test(Opcode::REM, neg(5), neg(5), 0);
    simple_op_code_test(Opcode::REM, 0, 0, 0);

    simple_op_code_test(Opcode::REMU, 4, 18, 7);
    simple_op_code_test(Opcode::REMU, 6, neg(20), 11);
    simple_op_code_test(Opcode::REMU, 23, 23, neg(6));
    simple_op_code_test(Opcode::REMU, neg(21), neg(21), neg(11));
    simple_op_code_test(Opcode::REMU, 5, 5, 0);
    simple_op_code_test(Opcode::REMU, neg(1), neg(1), 0);
    simple_op_code_test(Opcode::REMU, 0, 0, 0);
  }

  #[test]
  fn shift_tests() {
    simple_op_code_test(Opcode::SLL, 0x00000001, 0x00000001, 0);
    simple_op_code_test(Opcode::SLL, 0x00000002, 0x00000001, 1);
    simple_op_code_test(Opcode::SLL, 0x00000080, 0x00000001, 7);
    simple_op_code_test(Opcode::SLL, 0x00004000, 0x00000001, 14);
    simple_op_code_test(Opcode::SLL, 0x80000000, 0x00000001, 31);
    simple_op_code_test(Opcode::SLL, 0xffffffff, 0xffffffff, 0);
    simple_op_code_test(Opcode::SLL, 0xfffffffe, 0xffffffff, 1);
    simple_op_code_test(Opcode::SLL, 0xffffff80, 0xffffffff, 7);
    simple_op_code_test(Opcode::SLL, 0xffffc000, 0xffffffff, 14);
    simple_op_code_test(Opcode::SLL, 0x80000000, 0xffffffff, 31);
    simple_op_code_test(Opcode::SLL, 0x21212121, 0x21212121, 0);
    simple_op_code_test(Opcode::SLL, 0x42424242, 0x21212121, 1);
    simple_op_code_test(Opcode::SLL, 0x90909080, 0x21212121, 7);
    simple_op_code_test(Opcode::SLL, 0x48484000, 0x21212121, 14);
    simple_op_code_test(Opcode::SLL, 0x80000000, 0x21212121, 31);
    simple_op_code_test(Opcode::SLL, 0x21212121, 0x21212121, 0xffffffe0);
    simple_op_code_test(Opcode::SLL, 0x42424242, 0x21212121, 0xffffffe1);
    simple_op_code_test(Opcode::SLL, 0x90909080, 0x21212121, 0xffffffe7);
    simple_op_code_test(Opcode::SLL, 0x48484000, 0x21212121, 0xffffffee);
    simple_op_code_test(Opcode::SLL, 0x00000000, 0x21212120, 0xffffffff);

    simple_op_code_test(Opcode::SRL, 0xffff8000, 0xffff8000, 0);
    simple_op_code_test(Opcode::SRL, 0x7fffc000, 0xffff8000, 1);
    simple_op_code_test(Opcode::SRL, 0x01ffff00, 0xffff8000, 7);
    simple_op_code_test(Opcode::SRL, 0x0003fffe, 0xffff8000, 14);
    simple_op_code_test(Opcode::SRL, 0x0001ffff, 0xffff8001, 15);
    simple_op_code_test(Opcode::SRL, 0xffffffff, 0xffffffff, 0);
    simple_op_code_test(Opcode::SRL, 0x7fffffff, 0xffffffff, 1);
    simple_op_code_test(Opcode::SRL, 0x01ffffff, 0xffffffff, 7);
    simple_op_code_test(Opcode::SRL, 0x0003ffff, 0xffffffff, 14);
    simple_op_code_test(Opcode::SRL, 0x00000001, 0xffffffff, 31);
    simple_op_code_test(Opcode::SRL, 0x21212121, 0x21212121, 0);
    simple_op_code_test(Opcode::SRL, 0x10909090, 0x21212121, 1);
    simple_op_code_test(Opcode::SRL, 0x00424242, 0x21212121, 7);
    simple_op_code_test(Opcode::SRL, 0x00008484, 0x21212121, 14);
    simple_op_code_test(Opcode::SRL, 0x00000000, 0x21212121, 31);
    simple_op_code_test(Opcode::SRL, 0x21212121, 0x21212121, 0xffffffe0);
    simple_op_code_test(Opcode::SRL, 0x10909090, 0x21212121, 0xffffffe1);
    simple_op_code_test(Opcode::SRL, 0x00424242, 0x21212121, 0xffffffe7);
    simple_op_code_test(Opcode::SRL, 0x00008484, 0x21212121, 0xffffffee);
    simple_op_code_test(Opcode::SRL, 0x00000000, 0x21212121, 0xffffffff);

    simple_op_code_test(Opcode::SRA, 0x00000000, 0x00000000, 0);
    simple_op_code_test(Opcode::SRA, 0xc0000000, 0x80000000, 1);
    simple_op_code_test(Opcode::SRA, 0xff000000, 0x80000000, 7);
    simple_op_code_test(Opcode::SRA, 0xfffe0000, 0x80000000, 14);
    simple_op_code_test(Opcode::SRA, 0xffffffff, 0x80000001, 31);
    simple_op_code_test(Opcode::SRA, 0x7fffffff, 0x7fffffff, 0);
    simple_op_code_test(Opcode::SRA, 0x3fffffff, 0x7fffffff, 1);
    simple_op_code_test(Opcode::SRA, 0x00ffffff, 0x7fffffff, 7);
    simple_op_code_test(Opcode::SRA, 0x0001ffff, 0x7fffffff, 14);
    simple_op_code_test(Opcode::SRA, 0x00000000, 0x7fffffff, 31);
    simple_op_code_test(Opcode::SRA, 0x81818181, 0x81818181, 0);
    simple_op_code_test(Opcode::SRA, 0xc0c0c0c0, 0x81818181, 1);
    simple_op_code_test(Opcode::SRA, 0xff030303, 0x81818181, 7);
    simple_op_code_test(Opcode::SRA, 0xfffe0606, 0x81818181, 14);
    simple_op_code_test(Opcode::SRA, 0xffffffff, 0x81818181, 31);
  }

  pub fn simple_memory_program() -> Program {
    let instructions = vec![
      Instruction::new(Opcode::ADD, 29, 0, 0x12348765, false, true),
      // SW and LW
      Instruction::new(Opcode::SW, 29, 0, 0x27654320, false, true),
      Instruction::new(Opcode::LW, 28, 0, 0x27654320, false, true),
      // LBU
      Instruction::new(Opcode::LBU, 27, 0, 0x27654320, false, true),
      Instruction::new(Opcode::LBU, 26, 0, 0x27654321, false, true),
      Instruction::new(Opcode::LBU, 25, 0, 0x27654322, false, true),
      Instruction::new(Opcode::LBU, 24, 0, 0x27654323, false, true),
      // LB
      Instruction::new(Opcode::LB, 23, 0, 0x27654320, false, true),
      Instruction::new(Opcode::LB, 22, 0, 0x27654321, false, true),
      // LHU
      Instruction::new(Opcode::LHU, 21, 0, 0x27654320, false, true),
      Instruction::new(Opcode::LHU, 20, 0, 0x27654322, false, true),
      // LU
      Instruction::new(Opcode::LH, 19, 0, 0x27654320, false, true),
      Instruction::new(Opcode::LH, 18, 0, 0x27654322, false, true),
      // SB
      Instruction::new(Opcode::ADD, 17, 0, 0x38276525, false, true),
      // Save the value 0x12348765 into address 0x43627530
      Instruction::new(Opcode::SW, 29, 0, 0x43627530, false, true),
      Instruction::new(Opcode::SB, 17, 0, 0x43627530, false, true),
      Instruction::new(Opcode::LW, 16, 0, 0x43627530, false, true),
      Instruction::new(Opcode::SB, 17, 0, 0x43627531, false, true),
      Instruction::new(Opcode::LW, 15, 0, 0x43627530, false, true),
      Instruction::new(Opcode::SB, 17, 0, 0x43627532, false, true),
      Instruction::new(Opcode::LW, 14, 0, 0x43627530, false, true),
      Instruction::new(Opcode::SB, 17, 0, 0x43627533, false, true),
      Instruction::new(Opcode::LW, 13, 0, 0x43627530, false, true),
      // SH
      // Save the value 0x12348765 into address 0x43627530
      Instruction::new(Opcode::SW, 29, 0, 0x43627530, false, true),
      Instruction::new(Opcode::SH, 17, 0, 0x43627530, false, true),
      Instruction::new(Opcode::LW, 12, 0, 0x43627530, false, true),
      Instruction::new(Opcode::SH, 17, 0, 0x43627532, false, true),
      Instruction::new(Opcode::LW, 11, 0, 0x43627530, false, true),
    ];
    Program::new(instructions, 0, 0)
  }

  #[test]
  fn test_simple_memory_program_run() {
    let program = simple_memory_program();
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();

    // Assert SW & LW case
    assert_eq!(runtime.register(Register::X28), 0x12348765);

    // Assert LBU cases
    assert_eq!(runtime.register(Register::X27), 0x65);
    assert_eq!(runtime.register(Register::X26), 0x87);
    assert_eq!(runtime.register(Register::X25), 0x34);
    assert_eq!(runtime.register(Register::X24), 0x12);

    // Assert LB cases
    assert_eq!(runtime.register(Register::X23), 0x65);
    assert_eq!(runtime.register(Register::X22), 0xffffff87);

    // Assert LHU cases
    assert_eq!(runtime.register(Register::X21), 0x8765);
    assert_eq!(runtime.register(Register::X20), 0x1234);

    // Assert LH cases
    assert_eq!(runtime.register(Register::X19), 0xffff8765);
    assert_eq!(runtime.register(Register::X18), 0x1234);

    // Assert SB cases
    assert_eq!(runtime.register(Register::X16), 0x12348725);
    assert_eq!(runtime.register(Register::X15), 0x12342525);
    assert_eq!(runtime.register(Register::X14), 0x12252525);
    assert_eq!(runtime.register(Register::X13), 0x25252525);

    // Assert SH cases
    assert_eq!(runtime.register(Register::X12), 0x12346525);
    assert_eq!(runtime.register(Register::X11), 0x65256525);
  }
}
