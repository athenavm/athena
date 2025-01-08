pub mod gdbstub;
pub mod hooks;
mod io;
mod opcode;
mod program;
mod register;
mod state;
mod syscall;

use anyhow::anyhow;
use athena_interface::MethodSelector;
pub use opcode::*;
pub use program::*;
pub use register::*;
pub use state::*;
pub use syscall::*;

use std::collections::hash_map::Entry;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::sync::Arc;

use thiserror::Error;

use crate::host::HostInterface;
use crate::instruction::Instruction;
use crate::utils::AthenaCoreOpts;

use athena_interface::{AthenaContext, StatusCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum Base {
  RV32E,
}

const fn align(addr: u32) -> u32 {
  addr - addr % 4
}

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

  breakpoints: BTreeSet<u32>,

  hook_registry: hooks::HookRegistry,
}

#[non_exhaustive]
#[derive(Error, Debug, PartialEq)]
pub enum ExecutionError {
  #[error("execution failed with exit code {0}")]
  HaltWithNonZeroExitCode(u32),
  #[error("syscall failed with status code {0}")]
  SyscallFailed(StatusCode),
  #[error("invalid memory access {2} at address {1} for instruction {0:?}")]
  InvalidMemoryAccess(Instruction, u32, MemoryErr),
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
  #[error("parsing code failed ({0})")]
  ParsingCodeFailed(String),
  #[error("failed to fetch instruction at PC: {pc}")]
  InstructionFetchFailed { pc: u32 },
}

#[non_exhaustive]
#[derive(Error, Debug, PartialEq, Eq)]
pub enum MemoryErr {
  #[error("unaligned memory access")]
  Unaligned,
  #[error("memory access out of bounds")]
  OutOfBounds,
}

fn check_memory_access(addr: u32) -> Result<(), MemoryErr> {
  if addr % 4 != 0 {
    return Err(MemoryErr::Unaligned);
  }
  if addr < 40 {
    return Err(MemoryErr::OutOfBounds);
  }
  Ok(())
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
      io_buf: HashMap::new(),
      trace_buf,
      unconstrained: false,
      max_gas: opts.max_gas(),
      syscall_map,
      max_syscall_cycles,
      breakpoints: BTreeSet::new(),
      hook_registry: Default::default(),
    }
  }

  fn registers(&self) -> &[u32] {
    self.state.regs.all()
  }

  /// Get the current value of a register.
  pub fn register(&self, register: Register) -> u32 {
    self.rr(register)
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
    let word = self.word(align(addr));
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

  // read a u32 from memory, addr must be aligned to 4.
  fn read_memory_u32(&mut self, addr: u32) -> Result<u32, MemoryErr> {
    if addr % 4 != 0 {
      return Err(MemoryErr::Unaligned);
    }
    Ok(self.mr(addr))
  }

  // read a u16 from memory, addr must be aligned to 2.
  fn read_memory_u16(&mut self, addr: u32) -> Result<u16, MemoryErr> {
    match addr % 4 {
      0 => Ok(self.mr(align(addr)) as u16),
      2 => Ok((self.mr(align(addr)) >> 16) as u16),
      _ => Err(MemoryErr::Unaligned),
    }
  }

  /// Write to memory.
  pub fn mw_cpu(&mut self, addr: u32, value: u32) -> Result<(), MemoryErr> {
    check_memory_access(addr)?;

    self.mw(addr, value);
    Ok(())
  }

  /// Read from a register.
  pub fn rr(&self, register: Register) -> u32 {
    self.state.regs.read(register)
  }

  /// Write to a register.
  pub fn rw(&mut self, register: Register, value: u32) {
    self.state.regs.write(register, value)
  }

  /// Execute the instruction and return the new PC
  fn execute_instruction(&mut self, instruction: Instruction) -> Result<u32, ExecutionError> {
    self.trace_execution(instruction);

    let mut next_pc = self.state.pc.wrapping_add(4);

    match instruction {
      // Load upper immediate
      Instruction::Lui(rd, imm) => {
        self.rw(rd, imm << 12);
      }

      // Add upper immediate to PC
      Instruction::Auipc(rd, imm) => {
        let value = self.state.pc.wrapping_add_signed(imm << 12);
        self.rw(rd, value);
      }

      // Jump and link
      Instruction::Jal(rd, imm) => {
        let value = self.state.pc + 4;
        self.rw(rd, value);
        next_pc = self.state.pc.wrapping_add_signed(imm);
      }

      // Jump and link register
      Instruction::Jalr(rd, rs1, imm) => {
        let value = self.state.pc + 4;
        next_pc = self.rr(rs1).wrapping_add_signed(imm);
        self.rw(rd, value);
      }

      // Load instructions
      Instruction::Lb(rd, rs1, imm) => {
        let addr = self.rr(rs1).wrapping_add_signed(imm);
        let memory_read_value = self.mr(align(addr));
        let value = memory_read_value.to_le_bytes()[(addr % 4) as usize] as i8;
        self.rw(rd, value as i32 as u32);
      }

      Instruction::Lh(rd, rs1, imm) => {
        let addr = self.rr(rs1).wrapping_add_signed(imm);
        let value = self
          .read_memory_u16(addr)
          .map_err(|e| ExecutionError::InvalidMemoryAccess(instruction, addr, e))?
          as i16;
        self.rw(rd, value as i32 as u32);
      }

      Instruction::Lw(rd, rs1, imm) => {
        let addr = self.rr(rs1).wrapping_add_signed(imm);
        let value = self
          .read_memory_u32(addr)
          .map_err(|e| ExecutionError::InvalidMemoryAccess(instruction, addr, e))?;
        self.rw(rd, value);
      }

      Instruction::Lbu(rd, rs1, imm) => {
        let addr = self.rr(rs1).wrapping_add_signed(imm);
        let value = self.byte(addr);
        self.rw(rd, value as u32);
      }

      Instruction::Lhu(rd, rs1, imm) => {
        let addr = self.rr(rs1).wrapping_add_signed(imm);
        let value = self
          .read_memory_u16(addr)
          .map_err(|e| ExecutionError::InvalidMemoryAccess(instruction, addr, e))?;
        self.rw(rd, value as u32);
      }

      // Immediate arithmetic
      Instruction::Addi(rd, rs1, imm) => {
        self.rw(rd, self.rr(rs1).wrapping_add_signed(imm));
      }
      // SLTI (set less than immediate) places the value 1 in register rd
      // if register rs1 is less than the sign-extended immediate when both are treated as signed numbers,
      // else 0 is written to rd.
      Instruction::Slti(rd, rs1, imm) => {
        self.rw(rd, if (self.rr(rs1) as i32) < imm { 1 } else { 0 });
      }

      // SLTIU is compares the values as unsigned numbers
      // (i.e., the immediate is first sign-extended to XLEN bits then treated as an unsigned number).
      // Note, SLTIU rd, rs1, 1 sets rd to 1 if rs1 equals zero, otherwise sets rd to 0.
      Instruction::Sltiu(rd, rs1, imm) => {
        self.rw(rd, if self.rr(rs1) < imm as u32 { 1 } else { 0 });
      }

      Instruction::Xori(rd, rs1, imm) => {
        self.rw(rd, self.rr(rs1) ^ imm as u32);
      }

      Instruction::Ori(rd, rs1, imm) => {
        self.rw(rd, self.rr(rs1) | imm as u32);
      }

      Instruction::Andi(rd, rs1, imm) => {
        self.rw(rd, self.rr(rs1) & imm as u32);
      }

      Instruction::Slli(rd, rs1, imm) => {
        self.rw(rd, self.rr(rs1).wrapping_shl(imm));
      }

      Instruction::Srli(rd, rs1, imm) => {
        self.rw(rd, self.rr(rs1).wrapping_shr(imm));
      }

      Instruction::Srai(rd, rs1, imm) => {
        self.rw(rd, ((self.rr(rs1) as i32).wrapping_shr(imm)) as u32);
      }

      // Store instructions
      Instruction::Sb(rs2, rs1, imm) => {
        let addr = self.rr(rs1).wrapping_add_signed(imm);
        let value = self.rr(rs2) & 0xFF;
        let memory_read_value = self.mr(align(addr));
        let value = match addr % 4 {
          0 => value + (memory_read_value & 0xFFFFFF00),
          1 => (value << 8) + (memory_read_value & 0xFFFF00FF),
          2 => (value << 16) + (memory_read_value & 0xFF00FFFF),
          3 => (value << 24) + (memory_read_value & 0x00FFFFFF),
          _ => unreachable!(),
        };
        let addr = align(addr);
        self
          .mw_cpu(addr, value)
          .map_err(|e| ExecutionError::InvalidMemoryAccess(instruction, addr, e))?;
      }

      Instruction::Sh(rs2, rs1, imm) => {
        let addr = self.rr(rs1).wrapping_add_signed(imm);
        let value = self.rr(rs2) & 0xFFFF;
        let memory_read_value = self.mr(align(addr));
        let value = match addr % 4 {
          0 => value + (memory_read_value & 0xFFFF0000),
          2 => (value << 16) + (memory_read_value & 0x0000FFFF),
          _ => {
            return Err(ExecutionError::InvalidMemoryAccess(
              instruction,
              addr,
              MemoryErr::Unaligned,
            ))
          }
        };
        let addr = align(addr);
        self
          .mw_cpu(addr, value)
          .map_err(|e| ExecutionError::InvalidMemoryAccess(instruction, addr, e))?;
      }

      Instruction::Sw(rs2, rs1, imm) => {
        let addr = self.rr(rs1).wrapping_add_signed(imm);
        let value = self.rr(rs2);
        self
          .mw_cpu(addr, value)
          .map_err(|e| ExecutionError::InvalidMemoryAccess(instruction, addr, e))?;
      }

      // Register arithmetic
      Instruction::Add(rd, rs1, rs2) => {
        self.rw(rd, self.rr(rs1).wrapping_add(self.rr(rs2)));
      }

      Instruction::Sub(rd, rs1, rs2) => {
        self.rw(rd, self.rr(rs1).wrapping_sub(self.rr(rs2)));
      }

      Instruction::Sll(rd, rs1, rs2) => {
        self.rw(rd, self.rr(rs1).wrapping_shl(self.rr(rs2)));
      }

      Instruction::Slt(rd, rs1, rs2) => {
        self.rw(
          rd,
          if (self.rr(rs1) as i32) < (self.rr(rs2) as i32) {
            1
          } else {
            0
          },
        );
      }

      Instruction::Sltu(rd, rs1, rs2) => {
        self.rw(rd, if self.rr(rs1) < self.rr(rs2) { 1 } else { 0 });
      }

      Instruction::Xor(rd, rs1, rs2) => {
        self.rw(rd, self.rr(rs1) ^ self.rr(rs2));
      }

      Instruction::Srl(rd, rs1, rs2) => {
        self.rw(rd, self.rr(rs1).wrapping_shr(self.rr(rs2)));
      }

      Instruction::Sra(rd, rs1, rs2) => {
        self.rw(
          rd,
          ((self.rr(rs1) as i32).wrapping_shr(self.rr(rs2))) as u32,
        );
      }

      Instruction::Or(rd, rs1, rs2) => {
        self.rw(rd, self.rr(rs1) | self.rr(rs2));
      }

      Instruction::And(rd, rs1, rs2) => {
        self.rw(rd, self.rr(rs1) & self.rr(rs2));
      }

      // Branch instructions
      Instruction::Beq(rs1, rs2, imm) => {
        if self.rr(rs1) == self.rr(rs2) {
          next_pc = self.state.pc.wrapping_add_signed(imm);
        }
      }

      Instruction::Bne(rs1, rs2, imm) => {
        if self.rr(rs1) != self.rr(rs2) {
          next_pc = self.state.pc.wrapping_add_signed(imm);
        }
      }

      Instruction::Blt(rs1, rs2, imm) => {
        if (self.rr(rs1) as i32) < (self.rr(rs2) as i32) {
          next_pc = self.state.pc.wrapping_add_signed(imm);
        }
      }

      Instruction::Bge(rs1, rs2, imm) => {
        if (self.rr(rs1) as i32) >= (self.rr(rs2) as i32) {
          next_pc = self.state.pc.wrapping_add_signed(imm);
        }
      }

      Instruction::Bltu(rs1, rs2, imm) => {
        if self.rr(rs1) < self.rr(rs2) {
          next_pc = self.state.pc.wrapping_add_signed(imm);
        }
      }

      Instruction::Bgeu(rs1, rs2, imm) => {
        if self.rr(rs1) >= self.rr(rs2) {
          next_pc = self.state.pc.wrapping_add_signed(imm);
        }
      }

      // Multiply instructions
      Instruction::Mul(rd, rs1, rs2) => {
        self.rw(rd, self.rr(rs1).wrapping_mul(self.rr(rs2)));
      }

      Instruction::Mulh(rd, rs1, rs2) => {
        let value =
          (((self.rr(rs1) as i32) as i64).wrapping_mul((self.rr(rs2) as i32) as i64) >> 32) as u32;
        self.rw(rd, value);
      }
      Instruction::Mulhsu(rd, rs1, rs2) => {
        let value = (((self.rr(rs1) as i32) as i64).wrapping_mul(self.rr(rs2) as i64) >> 32) as u32;
        self.rw(rd, value);
      }
      Instruction::Mulhu(rd, rs1, rs2) => {
        let value = ((self.rr(rs1) as u64).wrapping_mul(self.rr(rs2) as u64) >> 32) as u32;
        self.rw(rd, value);
      }
      Instruction::Div(rd, rs1, rs2) => {
        let rhs = self.rr(rs2);
        let value = if rhs == 0 {
          u32::MAX
        } else {
          (self.rr(rs1) as i32).wrapping_div(rhs as i32) as u32
        };
        self.rw(rd, value);
      }
      Instruction::Divu(rd, rs1, rs2) => {
        let rhs = self.rr(rs2);
        let value = if rhs == 0 {
          u32::MAX
        } else {
          self.rr(rs1).wrapping_div(rhs)
        };
        self.rw(rd, value);
      }
      Instruction::Rem(rd, rs1, rs2) => {
        let (lhs, rhs) = (self.rr(rs1) as i32, self.rr(rs2) as i32);
        let value = if rhs == 0 { lhs } else { lhs.wrapping_rem(rhs) };
        self.rw(rd, value as u32);
      }
      Instruction::Remu(rd, rs1, rs2) => {
        let (lhs, rhs) = (self.rr(rs1), self.rr(rs2));
        let value = if rhs == 0 { lhs } else { lhs.wrapping_rem(rhs) };
        self.rw(rd, value);
      }
      Instruction::Ecall => {
        let t0 = Register::X5;
        let syscall_id = self.register(t0);
        let c = self.rr(Register::X11);
        let b = self.rr(Register::X10);
        let syscall = SyscallCode::from_u32(syscall_id);

        let syscall_impl = self
          .get_syscall(syscall)
          .ok_or(ExecutionError::UnsupportedSyscall(syscall_id))?
          .clone();
        let mut ctx = SyscallContext::new(self);

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

        self.state.gas += syscall_impl.num_extra_cycles();
      }
      Instruction::Ebreak => {
        return Err(ExecutionError::Breakpoint());
      }
      Instruction::NotImplemented { opcode } => {
        tracing::error!(opcode, "found unimplemented opcode");
        return Err(ExecutionError::Unimplemented());
      }
    }
    self.state.pc = next_pc;
    self.state.gas += 4;
    self.state.global_clk += 1;

    Ok(next_pc)
  }

  #[inline]
  fn execute_cycle(&mut self) -> Result<Option<Event>, ExecutionError> {
    let instruction = self
      .program
      .instruction(self.state.pc)
      .ok_or(ExecutionError::InstructionFetchFailed { pc: self.state.pc })?;

    self.execute_instruction(instruction)?;

    if self.breakpoints.contains(&self.state.pc) {
      return Ok(Some(Event::Break));
    }

    // We're allowed to spend all of our gas, but no more.
    // Gas checking is "lazy" here: it happens _after_ the instruction is executed.
    if let Some(gas_left) = self.gas_left() {
      if !self.unconstrained && gas_left < 0 {
        tracing::debug!("out of gas");
        return Err(ExecutionError::OutOfGas());
      }
    }
    if self.state.pc == 0 {
      tracing::debug!("HALT: zero PC (possibly returned from a function execution");
      return Ok(Some(Event::Halted));
    }

    let relative_pc = self.state.pc.wrapping_sub(self.program.pc_base);
    let max_pc = self.program.instructions.len() as u32 * 4;
    if relative_pc >= max_pc {
      tracing::warn!(relative_pc, max_pc, "HALT: out of instructions");
      return Ok(Some(Event::Halted));
    }
    Ok(None)
  }

  pub(crate) fn gas_left(&self) -> Option<i64> {
    // gas left can be negative, if we spent too much on the last instruction
    self
      .max_gas
      .map(|max_gas| max_gas as i64 - self.state.gas as i64)
  }

  pub fn initialize(&mut self) {
    self.state.gas = 0;

    // TODO: do we want to load all of the memory when executing a particular function?
    tracing::info!("loading memory image");
    for (addr, value) in self.program.memory_image.iter() {
      self.state.memory.insert(*addr, *value);
    }
  }

  pub(crate) fn jump_to_symbol(&mut self, symbol_name: &str) -> Result<(), ExecutionError> {
    // Make sure the symbol exists, and set the program counter
    let offset = match self.program.symbol_table.get(symbol_name) {
      Some(offset) => *offset,
      None => return Err(ExecutionError::UnknownSymbol()),
    };
    self.state.pc = offset;
    Ok(())
  }

  /// Execute an exported function. Does the same work as execute().
  pub fn execute_function_by_name(
    &mut self,
    symbol_name: &str,
  ) -> Result<Option<u32>, ExecutionError> {
    self.jump_to_symbol(symbol_name)?;
    // Hand over to execute
    self.execute()
  }

  /// Execute an exported function using method selector. Does the same work as execute().
  pub fn execute_function_by_selector(
    &mut self,
    selector: &MethodSelector,
  ) -> Result<Option<u32>, ExecutionError> {
    // Make sure the selector exists, and set the program counter
    let offset = match self.program.selector_table.get(selector) {
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

    tracing::info!("starting execution");
    // Loop until program finishes execution or until an error occurs, whichever comes first
    loop {
      if Some(Event::Halted) == self.execute_cycle()? {
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
      self.state.gas,
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

  /// Register a new hook.
  /// Will fail if a hook is already registered on a given FD
  /// or a FD is <= 4.
  pub fn register_hook(&mut self, fd: u32, hook: Box<dyn hooks::Hook>) -> anyhow::Result<()> {
    self.hook_registry.register(fd, hook)
  }

  pub(crate) fn execute_hook(&self, fd: u32, data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let hook = self
      .hook_registry
      .get(fd)
      .ok_or_else(|| anyhow!("no hook registered for FD {fd}"))?;
    hook.execute(hooks::HookEnv { runtime: self }, data)
  }

  fn trace_execution(&mut self, instruction: Instruction) {
    // Write the current program counter to the trace buffer for the cycle tracer.
    if let Some(ref mut buf) = self.trace_buf {
      if !self.unconstrained {
        buf.write_all(&u32::to_be_bytes(self.state.pc)).unwrap();
      }
    }

    tracing::trace!(
      clk = self.state.gas,
      global_clk = self.state.global_clk,
      pc = format_args!("0x{:x}", self.state.pc),
      instruction = ?instruction,
      registers = ?self.state.regs,
    );
  }
}

#[cfg(test)]
pub mod tests {
  use crate::{
    host::MockHostInterface, instruction::Instruction, runtime::ExecutionError, utils::with_max_gas,
  };
  use athena_interface::{Address, AthenaContext, ExecutionResult, StatusCode, ADDRESS_LENGTH};

  use crate::{
    runtime::Register,
    utils::{tests::TEST_PANIC_ELF, AthenaCoreOpts},
  };

  use super::{
    hooks::{self, MockHook},
    syscall::SyscallCode,
  };
  use super::{Program, Runtime};

  pub(crate) fn setup_logger() {
    let _ = tracing_subscriber::fmt()
      .with_test_writer()
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .try_init();
  }

  #[test]
  fn test_panic() {
    let program = Program::from(TEST_PANIC_ELF).unwrap();
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap_err();
  }

  #[test]
  fn test_gas() {
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::Add(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);
    let ctx = AthenaContext::new(Address::default(), Address::default(), 0);

    // failure
    let mut runtime = Runtime::new(
      program.clone(),
      None,
      AthenaCoreOpts::default().with_options(vec![with_max_gas(1)]),
      Some(ctx.clone()),
    );
    assert!(matches!(runtime.execute(), Err(ExecutionError::OutOfGas())));

    // success
    let mut runtime = Runtime::new(
      program.clone(),
      None,
      AthenaCoreOpts::default().with_options(vec![with_max_gas(12)]),
      Some(ctx.clone()),
    );
    let gas_left = runtime.execute().unwrap();
    assert_eq!(gas_left, Some(0));

    // success
    let mut runtime = Runtime::new(
      program.clone(),
      None,
      AthenaCoreOpts::default().with_options(vec![with_max_gas(13)]),
      Some(ctx.clone()),
    );
    let gas_left = runtime.execute().unwrap();
    assert_eq!(gas_left, Some(1));
  }

  #[test]
  fn test_call_send() {
    setup_logger();

    // recipient address
    let address = Address::from([0xCA; ADDRESS_LENGTH]);

    let amount_to_send = 1000;

    // arbitrary memory locations
    let memloc: u32 = 0x12345678;
    let memloc2 = memloc.wrapping_add(address.as_ref().len() as u32);

    let mut instructions = vec![];

    // write address to memory
    for (i, word) in address.as_ref().chunks_exact(4).enumerate() {
      instructions.push(Instruction::Addi(
        Register::X14,
        Register::X0,
        u32::from_le_bytes(word.try_into().unwrap()) as i32,
      ));
      instructions.push(Instruction::Sw(
        Register::X14,
        Register::X0,
        (memloc + i as u32 * 4) as i32,
      ));
    }

    // write value to memory
    instructions.push(Instruction::Addi(
      Register::X14,
      Register::X0,
      amount_to_send as i32,
    ));
    instructions.push(Instruction::Sw(Register::X14, Register::X0, memloc2 as i32));

    instructions.push(Instruction::Addi(Register::X14, Register::X0, 0));
    instructions.push(Instruction::Sw(
      Register::X14,
      Register::X0,
      (memloc2 + 4) as i32,
    ));

    // X10 is arg1 (ptr to address)
    instructions.push(Instruction::Addi(
      Register::X10,
      Register::X0,
      memloc as i32,
    ));

    // X11 is arg2 (ptr to input)
    // zero pointer
    instructions.push(Instruction::Addi(Register::X11, Register::X0, 0));

    // X12 is arg3 (input len)
    // no input
    instructions.push(Instruction::Addi(Register::X12, Register::X0, 0));

    // X13 is arg4 (value ptr)
    instructions.push(Instruction::Addi(
      Register::X13,
      Register::X0,
      memloc2 as i32,
    ));

    // X5 is syscall ID
    instructions.push(Instruction::Addi(
      Register::X5,
      Register::X0,
      SyscallCode::HOST_CALL as i32,
    ));

    instructions.push(Instruction::Ecall);
    let program = Program::new(instructions, 0, 0);

    let mut host = MockHostInterface::new();
    host
      .expect_call()
      .once()
      .returning(|_| ExecutionResult::new(StatusCode::Success, 1000, None));
    let ctx = AthenaContext::new(Address::default(), Address::default(), 0);
    let opts = AthenaCoreOpts::default().with_options(vec![with_max_gas(100000)]);

    let mut runtime = Runtime::new(program, Some(&mut host), opts, Some(ctx));
    let result = runtime.execute().unwrap();
    assert!(result.unwrap() < 1000);
  }

  #[test]
  fn test_bad_call() {
    setup_logger();

    let instructions = vec![
      // X10 is arg1 (ptr to address)
      // memory location to store an address
      Instruction::Addi(Register::X10, Register::X0, 0x12345678),
      // store arbitrary address here
      Instruction::Sw(Register::X10, Register::X0, 0x12345678),
      // X11 is arg2 (ptr to input)
      // zero pointer
      Instruction::Addi(Register::X11, Register::X0, 0),
      // X12 is arg3 (input len)
      // no input
      Instruction::Addi(Register::X12, Register::X0, 0),
      // X5 is syscall ID
      Instruction::Addi(Register::X5, Register::X0, SyscallCode::HOST_CALL as i32),
      Instruction::Ecall,
    ];
    let program = Program::new(instructions, 0, 0);

    let ctx = AthenaContext::new(Address::default(), Address::default(), 0);
    let opts = AthenaCoreOpts::default().with_options(vec![with_max_gas(100000)]);
    let mut host = MockHostInterface::new();
    host
      .expect_call()
      .returning(|_| ExecutionResult::failed(1000));

    let mut runtime = Runtime::new(program, Some(&mut host), opts, Some(ctx));
    assert!(matches!(
      runtime.execute().unwrap_err(),
      ExecutionError::SyscallFailed(StatusCode::Failure)
    ));
  }

  #[test]
  fn syscall_get_balance() {
    setup_logger();

    // arbitrary memory location
    let memloc = 0x12345678;

    let instructions = vec![
      // X10 is arg1 (ptr to address)
      // store result here
      Instruction::Addi(Register::X10, Register::X0, memloc as i32),
      // X5 is syscall ID
      Instruction::Addi(
        Register::X5,
        Register::X0,
        SyscallCode::HOST_GETBALANCE as i32,
      ),
      Instruction::Ecall,
    ];
    let program = Program::new(instructions, 0, 0);

    let ctx = AthenaContext::new(Address::default(), Address::default(), 0);
    let opts = AthenaCoreOpts::default().with_options(vec![with_max_gas(100000)]);
    let mut host = MockHostInterface::new();
    host.expect_get_balance().return_const(1111u64);

    let mut runtime = Runtime::new(program, Some(&mut host), opts, Some(ctx));
    let gas_left = runtime.execute().expect("execution failed");
    assert!(gas_left.is_some());

    // check result: we expect the u64 value to be split into two 32-bit words
    let value_low = runtime.mr(memloc);
    let value_high = runtime.mr(memloc + 4);
    assert_eq!(u64::from(value_high) << 32 | u64::from(value_low), 1111u64);
  }

  #[test]
  fn test_syscall_fail() {
    let instructions = vec![
      Instruction::Addi(Register::X5, Register::X0, SyscallCode::WRITE as i32),
      Instruction::Ecall,
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
  fn test_lui() {
    let instructions = vec![Instruction::Lui(Register::X15, 123)];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 123 << 12);
  }

  #[test]
  fn test_auipc() {
    // Test adding different immediate values to PC
    // PC starts at 0, so we can easily calculate expected results

    let instructions = vec![
      // Add small positive immediate
      Instruction::Auipc(Register::X5, 1), // 1 << 12 = 4096
      // Add larger positive immediate
      Instruction::Auipc(Register::X6, 0x7FF), // 0x7FF << 12 = 0x7FF000
      // Add maximum positive immediate
      Instruction::Auipc(Register::X7, 0xFFFFF), // 0xFFFFF << 12 = 0xFFFFF000
    ];

    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();

    // First AUIPC: PC = 0, imm = 1
    // Expected: 0 + (1 << 12) = 4096
    assert_eq!(runtime.register(Register::X5), 4096);

    // Second AUIPC: PC = 4, imm = 0x7FF
    // Expected: 4 + (0x7FF << 12) = 0x7FF004
    assert_eq!(runtime.register(Register::X6), 0x7FF004);

    // Third AUIPC: PC = 8, imm = 0xFFFFF
    // Expected: 8 + (0xFFFFF << 12) = 0xFFFFF008
    assert_eq!(runtime.register(Register::X7), 0xFFFFF008);
  }

  #[test]
  fn test_add() {
    // main:
    //     addi x29, x0, 5
    //     addi x30, x0, 37
    //     add x31, x30, x29
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::Add(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 42);
  }

  #[test]
  fn test_sub() {
    //     addi x13, x0, 5
    //     addi x14, x0, 37
    //     sub x15, x14, x13
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::Sub(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 32);
  }

  #[test]
  fn test_xor() {
    //     addi x13, x0, 5
    //     addi x14, x0, 37
    //     xor x15, x14, x13
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::Xor(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 32);
  }

  #[test]
  fn test_or() {
    //     addi x13, x0, 5
    //     addi x14, x0, 37
    //     or x15, x14 x13
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::Or(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);

    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 37);
  }

  #[test]
  fn test_and() {
    //     addi x13, x0, 5
    //     addi x14, x0, 37
    //     and x15, x14, x13
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::And(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 5);
  }

  #[test]
  fn test_sll() {
    //     addi x13, x0, 5
    //     addi x14, x0, 37
    //     sll x15, x14, x13
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::Sll(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 1184);
  }

  #[test]
  fn test_srl() {
    //     addi x13, x0, 5
    //     addi x14, x0, 37
    //     srl x15, x14, x13
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::Srl(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 1);
  }

  #[test]
  fn test_sra() {
    //     addi x13, x0, 5
    //     addi x14, x0, 37
    //     sra x15, x14, x13
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::Sra(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 1);
  }

  #[test]
  fn test_slt() {
    //     addi x13, x0, 5
    //     addi x14, x0, 37
    //     slt x15, x14, x13
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::Slt(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 0);
  }

  #[test]
  fn test_sltu() {
    //     addi x13, x0, 5
    //     addi x14, x0, 37
    //     sltu x15, x14, x13
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X0, 37),
      Instruction::Sltu(Register::X15, Register::X14, Register::X13),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 0);
  }

  #[test]
  fn test_addi() {
    //     addi x13, x0, 5
    //     addi x14, x13, 37
    //     addi x15, x14, 42
    let instructions = vec![
      crate::instruction::Instruction::Addi(Register::X13, Register::X0, 5),
      crate::instruction::Instruction::Addi(Register::X14, Register::X13, 37),
      crate::instruction::Instruction::Addi(Register::X15, Register::X14, 42),
    ];
    let program = Program::new(instructions, 0, 0);

    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 84);
  }

  #[test]
  fn test_addi_negative() {
    //     addi x13, x0, 5
    //     addi x14, x13, -1
    //     addi x15, x14, 4
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Addi(Register::X14, Register::X13, -1),
      Instruction::Addi(Register::X15, Register::X14, 4),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 5 - 1 + 4);
  }

  #[test]
  fn test_xori() {
    //     addi x13, x0, 5
    //     xori x14, x13, 37
    //     xori x15, x14, 42
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Xori(Register::X14, Register::X13, 37),
      Instruction::Xori(Register::X15, Register::X14, 42),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 10);
  }

  #[test]
  fn test_ori() {
    //     addi x13, x0, 5
    //     ori x14, x13, 37
    //     ori x15, x14, 42
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Ori(Register::X14, Register::X13, 37),
      Instruction::Ori(Register::X15, Register::X14, 42),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 47);
  }

  #[test]
  fn test_andi() {
    //     addi x13, x0, 5
    //     andi x14, x13, 37
    //     andi x15, x14, 42
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Andi(Register::X14, Register::X13, 37),
      Instruction::Andi(Register::X15, Register::X13, 42),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 0);
  }

  #[test]
  fn test_slli() {
    //     addi x13, x0, 5
    //     slli x15, x13, 4
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 5),
      Instruction::Slli(Register::X15, Register::X13, 4),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 80);
  }

  #[test]
  fn test_srli() {
    //    addi x13, x0, 42
    //    srli x15, x13, 4
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 42),
      Instruction::Srli(Register::X15, Register::X13, 4),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 2);
  }

  #[test]
  fn test_srai() {
    //   addi x13, x0, 42
    //   srai x15, x13, 4
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 42),
      Instruction::Srai(Register::X15, Register::X13, 4),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 2);
  }

  #[test]
  fn test_slti() {
    //   addi x13, x0, 42
    //   slti x15, x13, 37
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 42),
      Instruction::Slti(Register::X15, Register::X13, 37),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 0);
  }

  #[test]
  fn test_sltiu() {
    //   addi x13, x0, 42
    //   sltiu x15, x13, 37
    let instructions = vec![
      Instruction::Addi(Register::X13, Register::X0, 42),
      Instruction::Sltiu(Register::X15, Register::X13, 37),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.register(Register::X15), 0);
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
      Instruction::Addi(Register::X11, Register::X11, 100),
      Instruction::Jalr(Register::X5, Register::X11, 8),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.registers()[Register::X5 as usize], 8);
    assert_eq!(runtime.registers()[Register::X11 as usize], 100);
    assert_eq!(runtime.state.pc, 108);
  }

  fn simple_op_test(
    op: fn(Register, Register, Register) -> Instruction,
    expected: u32,
    a: u32,
    b: u32,
  ) {
    let instructions = vec![
      Instruction::Addi(Register::X10, Register::X0, a as i32),
      Instruction::Addi(Register::X11, Register::X0, b as i32),
      op(Register::X12, Register::X10, Register::X11),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    assert_eq!(runtime.registers()[Register::X12 as usize], expected);
  }

  #[test]
  fn multiplication_tests() {
    // Mulhu tests
    simple_op_test(Instruction::Mulhu, 0x00000000, 0x00000000, 0x00000000);
    simple_op_test(Instruction::Mulhu, 0x00000000, 0x00000001, 0x00000001);
    simple_op_test(Instruction::Mulhu, 0x00000000, 0x00000003, 0x00000007);
    simple_op_test(Instruction::Mulhu, 0x00000000, 0x00000000, 0xffff8000);
    simple_op_test(Instruction::Mulhu, 0x00000000, 0x80000000, 0x00000000);
    simple_op_test(Instruction::Mulhu, 0x7fffc000, 0x80000000, 0xffff8000);
    simple_op_test(Instruction::Mulhu, 0x0001fefe, 0xaaaaaaab, 0x0002fe7d);
    simple_op_test(Instruction::Mulhu, 0x0001fefe, 0x0002fe7d, 0xaaaaaaab);
    simple_op_test(Instruction::Mulhu, 0xfe010000, 0xff000000, 0xff000000);
    simple_op_test(Instruction::Mulhu, 0xfffffffe, 0xffffffff, 0xffffffff);
    simple_op_test(Instruction::Mulhu, 0x00000000, 0xffffffff, 0x00000001);
    simple_op_test(Instruction::Mulhu, 0x00000000, 0x00000001, 0xffffffff);

    // Mulhsu tests
    simple_op_test(Instruction::Mulhsu, 0x00000000, 0x00000000, 0x00000000);
    simple_op_test(Instruction::Mulhsu, 0x00000000, 0x00000001, 0x00000001);
    simple_op_test(Instruction::Mulhsu, 0x00000000, 0x00000003, 0x00000007);
    simple_op_test(Instruction::Mulhsu, 0x00000000, 0x00000000, 0xffff8000);
    simple_op_test(Instruction::Mulhsu, 0x00000000, 0x80000000, 0x00000000);
    simple_op_test(Instruction::Mulhsu, 0x80004000, 0x80000000, 0xffff8000);
    simple_op_test(Instruction::Mulhsu, 0xffff0081, 0xaaaaaaab, 0x0002fe7d);
    simple_op_test(Instruction::Mulhsu, 0x0001fefe, 0x0002fe7d, 0xaaaaaaab);
    simple_op_test(Instruction::Mulhsu, 0xff010000, 0xff000000, 0xff000000);
    simple_op_test(Instruction::Mulhsu, 0xffffffff, 0xffffffff, 0xffffffff);
    simple_op_test(Instruction::Mulhsu, 0xffffffff, 0xffffffff, 0x00000001);
    simple_op_test(Instruction::Mulhsu, 0x00000000, 0x00000001, 0xffffffff);

    // Mulh tests
    simple_op_test(Instruction::Mulh, 0x00000000, 0x00000000, 0x00000000);
    simple_op_test(Instruction::Mulh, 0x00000000, 0x00000001, 0x00000001);
    simple_op_test(Instruction::Mulh, 0x00000000, 0x00000003, 0x00000007);
    simple_op_test(Instruction::Mulh, 0x00000000, 0x00000000, 0xffff8000);
    simple_op_test(Instruction::Mulh, 0x00000000, 0x80000000, 0x00000000);
    simple_op_test(Instruction::Mulh, 0x00000000, 0x80000000, 0x00000000);
    simple_op_test(Instruction::Mulh, 0xffff0081, 0xaaaaaaab, 0x0002fe7d);
    simple_op_test(Instruction::Mulh, 0xffff0081, 0x0002fe7d, 0xaaaaaaab);
    simple_op_test(Instruction::Mulh, 0x00010000, 0xff000000, 0xff000000);
    simple_op_test(Instruction::Mulh, 0x00000000, 0xffffffff, 0xffffffff);
    simple_op_test(Instruction::Mulh, 0xffffffff, 0xffffffff, 0x00000001);
    simple_op_test(Instruction::Mulh, 0xffffffff, 0x00000001, 0xffffffff);

    // Mul tests
    simple_op_test(Instruction::Mul, 0x00001200, 0x00007e00, 0xb6db6db7);
    simple_op_test(Instruction::Mul, 0x00001240, 0x00007fc0, 0xb6db6db7);
    simple_op_test(Instruction::Mul, 0x00000000, 0x00000000, 0x00000000);
    simple_op_test(Instruction::Mul, 0x00000001, 0x00000001, 0x00000001);
    simple_op_test(Instruction::Mul, 0x00000015, 0x00000003, 0x00000007);
    simple_op_test(Instruction::Mul, 0x00000000, 0x00000000, 0xffff8000);
    simple_op_test(Instruction::Mul, 0x00000000, 0x80000000, 0x00000000);
    simple_op_test(Instruction::Mul, 0x00000000, 0x80000000, 0xffff8000);
    simple_op_test(Instruction::Mul, 0x0000ff7f, 0xaaaaaaab, 0x0002fe7d);
    simple_op_test(Instruction::Mul, 0x0000ff7f, 0x0002fe7d, 0xaaaaaaab);
    simple_op_test(Instruction::Mul, 0x00000000, 0xff000000, 0xff000000);
    simple_op_test(Instruction::Mul, 0x00000001, 0xffffffff, 0xffffffff);
    simple_op_test(Instruction::Mul, 0xffffffff, 0xffffffff, 0x00000001);
    simple_op_test(Instruction::Mul, 0xffffffff, 0x00000001, 0xffffffff);
  }

  fn neg(a: u32) -> u32 {
    u32::MAX - a + 1
  }

  #[test]
  fn division_tests() {
    simple_op_test(Instruction::Divu, 3, 20, 6);
    simple_op_test(Instruction::Divu, 715827879, u32::MAX - 20 + 1, 6);
    simple_op_test(Instruction::Divu, 0, 20, u32::MAX - 6 + 1);
    simple_op_test(Instruction::Divu, 0, u32::MAX - 20 + 1, u32::MAX - 6 + 1);

    simple_op_test(Instruction::Divu, 1 << 31, 1 << 31, 1);
    simple_op_test(Instruction::Divu, 0, 1 << 31, u32::MAX - 1 + 1);

    simple_op_test(Instruction::Divu, u32::MAX, 1 << 31, 0);
    simple_op_test(Instruction::Divu, u32::MAX, 1, 0);
    simple_op_test(Instruction::Divu, u32::MAX, 0, 0);

    simple_op_test(Instruction::Div, 3, 18, 6);
    simple_op_test(Instruction::Div, neg(6), neg(24), 4);
    simple_op_test(Instruction::Div, neg(2), 16, neg(8));
    simple_op_test(Instruction::Div, neg(1), 0, 0);

    // Overflow cases
    simple_op_test(Instruction::Div, 1 << 31, 1 << 31, neg(1));
    simple_op_test(Instruction::Rem, 0, 1 << 31, neg(1));
  }

  #[test]
  fn remainder_tests() {
    simple_op_test(Instruction::Rem, 7, 16, 9);
    simple_op_test(Instruction::Rem, neg(4), neg(22), 6);
    simple_op_test(Instruction::Rem, 1, 25, neg(3));
    simple_op_test(Instruction::Rem, neg(2), neg(22), neg(4));
    simple_op_test(Instruction::Rem, 0, 873, 1);
    simple_op_test(Instruction::Rem, 0, 873, neg(1));
    simple_op_test(Instruction::Rem, 5, 5, 0);
    simple_op_test(Instruction::Rem, neg(5), neg(5), 0);
    simple_op_test(Instruction::Rem, 0, 0, 0);

    simple_op_test(Instruction::Remu, 4, 18, 7);
    simple_op_test(Instruction::Remu, 6, neg(20), 11);
    simple_op_test(Instruction::Remu, 23, 23, neg(6));
    simple_op_test(Instruction::Remu, neg(21), neg(21), neg(11));
    simple_op_test(Instruction::Remu, 5, 5, 0);
    simple_op_test(Instruction::Remu, neg(1), neg(1), 0);
    simple_op_test(Instruction::Remu, 0, 0, 0);
  }

  #[test]
  fn shift_tests() {
    // Test logical left shifts (SLL)
    simple_op_test(Instruction::Sll, 0x00000002, 0x00000001, 1);
    simple_op_test(Instruction::Sll, 0x00000080, 0x00000001, 7);
    simple_op_test(Instruction::Sll, 0x00004000, 0x00000001, 14);
    simple_op_test(Instruction::Sll, 0x80000000, 0x00000001, 31);
    simple_op_test(Instruction::Sll, 0xfffffffe, 0xffffffff, 1);
    simple_op_test(Instruction::Sll, 0xffffff80, 0xffffffff, 7);
    simple_op_test(Instruction::Sll, 0x42424242, 0x21212121, 1);
    simple_op_test(Instruction::Sll, 0x90909080, 0x21212121, 7);

    // Test logical right shifts (SRL)
    simple_op_test(Instruction::Srl, 0x7fffc000, 0xffff8000, 1);
    simple_op_test(Instruction::Srl, 0x01ffff00, 0xffff8000, 7);
    simple_op_test(Instruction::Srl, 0x0003fffe, 0xffff8000, 14);
    simple_op_test(Instruction::Srl, 0x7fffffff, 0xffffffff, 1);
    simple_op_test(Instruction::Srl, 0x01ffffff, 0xffffffff, 7);
    simple_op_test(Instruction::Srl, 0x00000001, 0xffffffff, 31);
    simple_op_test(Instruction::Srl, 0x10909090, 0x21212121, 1);
    simple_op_test(Instruction::Srl, 0x00424242, 0x21212121, 7);

    // Test arithmetic right shifts (SRA)
    simple_op_test(Instruction::Sra, 0xc0000000, 0x80000000, 1);
    simple_op_test(Instruction::Sra, 0xff000000, 0x80000000, 7);
    simple_op_test(Instruction::Sra, 0xfffe0000, 0x80000000, 14);
    simple_op_test(Instruction::Sra, 0xffffffff, 0x80000001, 31);
    simple_op_test(Instruction::Sra, 0x3fffffff, 0x7fffffff, 1);
    simple_op_test(Instruction::Sra, 0x00ffffff, 0x7fffffff, 7);
    simple_op_test(Instruction::Sra, 0xc0c0c0c0, 0x81818181, 1);
    simple_op_test(Instruction::Sra, 0xff030303, 0x81818181, 7);
  }

  #[test]
  fn test_simple_memory_program_run() {
    let instructions = vec![
      Instruction::Addi(Register::X15, Register::X0, 0x12348765),
      // SW and LW
      Instruction::Sw(Register::X15, Register::X0, 0x27654320),
      Instruction::Lw(Register::X14, Register::X0, 0x27654320),
      // LBU
      Instruction::Lbu(Register::X13, Register::X0, 0x27654320),
      Instruction::Lbu(Register::X12, Register::X0, 0x27654321),
      Instruction::Lbu(Register::X11, Register::X0, 0x27654322),
      Instruction::Lbu(Register::X10, Register::X0, 0x27654323),
      // LB
      Instruction::Lb(Register::X9, Register::X0, 0x27654320),
      Instruction::Lb(Register::X8, Register::X0, 0x27654321),
      // LHU
      Instruction::Lhu(Register::X7, Register::X0, 0x27654320),
      Instruction::Lhu(Register::X6, Register::X0, 0x27654322),
      // LH
      Instruction::Lh(Register::X5, Register::X0, 0x27654320),
      Instruction::Lh(Register::X4, Register::X0, 0x27654322),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();

    // Assert SW & LW case
    assert_eq!(runtime.register(Register::X14), 0x12348765);

    // Assert LBU cases
    assert_eq!(runtime.register(Register::X13), 0x65);
    assert_eq!(runtime.register(Register::X12), 0x87);
    assert_eq!(runtime.register(Register::X11), 0x34);
    assert_eq!(runtime.register(Register::X10), 0x12);

    // Assert LB cases
    assert_eq!(runtime.register(Register::X9), 0x65);
    assert_eq!(runtime.register(Register::X8), 0xffffff87);

    // Assert LHU cases
    assert_eq!(runtime.register(Register::X7), 0x8765);
    assert_eq!(runtime.register(Register::X6), 0x1234);

    // Assert LH cases
    assert_eq!(runtime.register(Register::X5), 0xffff8765);
    assert_eq!(runtime.register(Register::X4), 0x1234);

    // SB
    let instructions = vec![
      Instruction::Addi(Register::X15, Register::X0, 0x12348765),
      Instruction::Addi(Register::X3, Register::X0, 0x38276525),
      // Save the value 0x12348765 into address 0x43627530
      Instruction::Sw(Register::X15, Register::X0, 0x43627530),
      Instruction::Sb(Register::X3, Register::X0, 0x43627530),
      Instruction::Lw(Register::X14, Register::X0, 0x43627530),
      Instruction::Sb(Register::X3, Register::X0, 0x43627531),
      Instruction::Lw(Register::X13, Register::X0, 0x43627530),
      Instruction::Sb(Register::X3, Register::X0, 0x43627532),
      Instruction::Lw(Register::X12, Register::X0, 0x43627530),
      Instruction::Sb(Register::X3, Register::X0, 0x43627533),
      Instruction::Lw(Register::X11, Register::X0, 0x43627530),
      // SH
      // Save the value 0x12348765 into address 0x43627530
      Instruction::Sw(Register::X15, Register::X0, 0x43627530),
      Instruction::Sh(Register::X3, Register::X0, 0x43627530),
      Instruction::Lw(Register::X10, Register::X0, 0x43627530),
      Instruction::Sh(Register::X3, Register::X0, 0x43627532),
      Instruction::Lw(Register::X9, Register::X0, 0x43627530),
    ];
    let program = Program::new(instructions, 0, 0);
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    runtime.execute().unwrap();
    // Assert SB cases
    assert_eq!(runtime.register(Register::X14), 0x12348725);
    assert_eq!(runtime.register(Register::X13), 0x12342525);
    assert_eq!(runtime.register(Register::X12), 0x12252525);
    assert_eq!(runtime.register(Register::X11), 0x25252525);

    // Assert SH cases
    assert_eq!(runtime.register(Register::X10), 0x12346525);
    assert_eq!(runtime.register(Register::X9), 0x65256525);
  }

  #[test]
  fn registering_hook() {
    let mut runtime = Runtime::new(
      Program::new(vec![], 0, 0),
      None,
      AthenaCoreOpts::default(),
      None,
    );

    // can't register on FD <= 4
    for fd in 0..=4 {
      runtime
        .register_hook(fd, Box::new(hooks::MockHook::new()))
        .unwrap_err();
    }

    // register on 5
    runtime
      .register_hook(5, Box::new(hooks::MockHook::new()))
      .unwrap();

    // can't register another hook on occupied FD
    runtime
      .register_hook(5, Box::new(hooks::MockHook::new()))
      .unwrap_err();
  }

  #[test]
  fn executing_hook() {
    let mut runtime = Runtime::new(
      Program::new(vec![], 0, 0),
      None,
      AthenaCoreOpts::default(),
      None,
    );

    let mut hook = Box::new(MockHook::new());
    hook.expect_execute().returning(|_, data| {
      assert_eq!(&[1, 2, 3, 4], data);
      Ok(vec![5, 6, 7])
    });
    runtime.register_hook(5, hook).unwrap();

    let result = runtime.execute_hook(5, &[1, 2, 3, 4]).unwrap();
    assert_eq!(vec![5, 6, 7], result);
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event {
  DoneStep,
  Halted,
  Break,
}
