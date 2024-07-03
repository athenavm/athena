use std::collections::HashMap;
use std::sync::Arc;

use strum_macros::EnumIter;

use crate::runtime::{Register, Runtime};
use crate::syscall::{
  SyscallHalt, SyscallHintLen, SyscallHintRead, SyscallHostRead, SyscallHostWrite, SyscallWrite,
};
use crate::{runtime::MemoryReadRecord, runtime::MemoryWriteRecord};

use athena_interface::HostInterface;

/// A system call is invoked by the the `ecall` instruction with a specific value in register t0.
/// The syscall number is a 32-bit integer, with the following layout (in little-endian format)
/// - The first byte is the syscall id.
/// - The second byte is 0/1 depending on whether the syscall has a separate table. This is used
/// in the CPU table to determine whether to lookup the syscall using the syscall interaction.
/// - The third byte is the number of additional cycles the syscall uses.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
#[allow(non_camel_case_types)]
pub enum SyscallCode {
  /// Halts the program.
  HALT = 0x00_00_00_00,

  /// Write to the output buffer.
  WRITE = 0x00_00_00_02,

  /// Host functions
  HOST_READ = 0x00_00_00_A0,
  HOST_WRITE = 0x00_00_00_A1,
  HOST_CALL = 0x00_00_00_A2,

  /// Executes the `HINT_LEN` precompile.
  HINT_LEN = 0x00_00_00_F0,

  /// Executes the `HINT_READ` precompile.
  HINT_READ = 0x00_00_00_F1,
}

impl SyscallCode {
  /// Create a syscall from a u32.
  pub fn from_u32(value: u32) -> Self {
    match value {
      0x00_00_00_00 => SyscallCode::HALT,
      0x00_00_00_02 => SyscallCode::WRITE,
      0x00_00_00_A0 => SyscallCode::HOST_READ,
      0x00_00_00_A1 => SyscallCode::HOST_WRITE,
      0x00_00_00_A2 => SyscallCode::HOST_CALL,
      0x00_00_00_F0 => SyscallCode::HINT_LEN,
      0x00_00_00_F1 => SyscallCode::HINT_READ,
      _ => panic!("invalid syscall number: {}", value),
    }
  }

  pub fn syscall_id(&self) -> u32 {
    (*self as u32).to_le_bytes()[0].into()
  }

  pub fn should_send(&self) -> u32 {
    (*self as u32).to_le_bytes()[1].into()
  }

  pub fn num_cycles(&self) -> u32 {
    (*self as u32).to_le_bytes()[2].into()
  }
}

pub trait Syscall<T: HostInterface>: Send + Sync {
  /// Execute the syscall and return the resulting value of register a0. `arg1` and `arg2` are the
  /// values in registers X10 and X11, respectively. While not a hard requirement, the convention
  /// is that the return value is only for system calls such as `HALT`. Most precompiles use `arg1`
  /// and `arg2` to denote the addresses of the input data, and write the result to the memory at
  /// `arg1`.
  fn execute(&self, ctx: &mut SyscallContext<T>, arg1: u32, arg2: u32) -> Option<u32>;

  /// The number of extra cycles that the syscall takes to execute. Unless this syscall is complex
  /// and requires many cycles, this should be zero.
  fn num_extra_cycles(&self) -> u32 {
    0
  }
}

/// A runtime for syscalls that is protected so that developers cannot arbitrarily modify the runtime.
pub struct SyscallContext<'a, T: HostInterface> {
  pub clk: u32,

  pub(crate) next_pc: u32,
  /// This is the exit_code used for the HALT syscall
  pub(crate) exit_code: u32,
  pub(crate) rt: &'a mut Runtime<T>,
}

impl<'a, T> SyscallContext<'a, T>
where
  T: HostInterface,
{
  pub fn new(runtime: &'a mut Runtime<T>) -> Self {
    let clk = runtime.state.clk;
    Self {
      clk,
      next_pc: runtime.state.pc.wrapping_add(4),
      exit_code: 0,
      rt: runtime,
    }
  }

  pub fn mr(&mut self, addr: u32) -> (MemoryReadRecord, u32) {
    let record = self.rt.mr(addr, self.clk);
    (record, record.value)
  }

  pub fn mr_slice(&mut self, addr: u32, len: usize) -> (Vec<MemoryReadRecord>, Vec<u32>) {
    let mut records = Vec::new();
    let mut values = Vec::new();
    for i in 0..len {
      let (record, value) = self.mr(addr + i as u32 * 4);
      records.push(record);
      values.push(value);
    }
    (records, values)
  }

  pub fn mw(&mut self, addr: u32, value: u32) -> MemoryWriteRecord {
    self.rt.mw(addr, value, self.clk)
  }

  pub fn mw_slice(&mut self, addr: u32, values: &[u32]) -> Vec<MemoryWriteRecord> {
    let mut records = Vec::new();
    for i in 0..values.len() {
      let record = self.mw(addr + i as u32 * 4, values[i]);
      records.push(record);
    }
    records
  }

  /// Get the current value of a register, but doesn't use a memory record.
  /// This is generally unconstrained, so you must be careful using it.
  pub fn register_unsafe(&self, register: Register) -> u32 {
    self.rt.register(register)
  }

  pub fn byte_unsafe(&self, addr: u32) -> u8 {
    self.rt.byte(addr)
  }

  pub fn word_unsafe(&self, addr: u32) -> u32 {
    self.rt.word(addr)
  }

  pub fn slice_unsafe(&self, addr: u32, len: usize) -> Vec<u32> {
    let mut values = Vec::new();
    for i in 0..len {
      values.push(self.rt.word(addr + i as u32 * 4));
    }
    values
  }

  pub fn set_next_pc(&mut self, next_pc: u32) {
    self.next_pc = next_pc;
  }

  pub fn set_exit_code(&mut self, exit_code: u32) {
    self.exit_code = exit_code;
  }
}

pub fn default_syscall_map<T: HostInterface>() -> HashMap<SyscallCode, Arc<dyn Syscall<T>>> {
  let mut syscall_map = HashMap::<SyscallCode, Arc<dyn Syscall<T>>>::default();
  syscall_map.insert(SyscallCode::HALT, Arc::new(SyscallHalt {}));
  syscall_map.insert(SyscallCode::WRITE, Arc::new(SyscallWrite::new()));
  syscall_map.insert(SyscallCode::HOST_READ, Arc::new(SyscallHostRead::new()));
  syscall_map.insert(SyscallCode::HOST_WRITE, Arc::new(SyscallHostWrite::new()));
  syscall_map.insert(SyscallCode::HOST_CALL, Arc::new(SyscallHostCall::new()));
  syscall_map.insert(SyscallCode::HINT_LEN, Arc::new(SyscallHintLen::new()));
  syscall_map.insert(SyscallCode::HINT_READ, Arc::new(SyscallHintRead::new()));

  syscall_map
}

#[cfg(test)]
mod tests {
  use super::{default_syscall_map, SyscallCode};
  use athena_interface::MockHost;
  use strum::IntoEnumIterator;

  #[test]
  fn test_syscalls_in_default_map() {
    let default_syscall_map = default_syscall_map::<MockHost>();
    for code in SyscallCode::iter() {
      default_syscall_map.get(&code).unwrap();
    }
  }

  #[test]
  fn test_syscall_num_cycles_encoding() {
    for (syscall_code, syscall_impl) in default_syscall_map::<MockHost>().iter() {
      let encoded_num_cycles = syscall_code.num_cycles();
      assert_eq!(syscall_impl.num_extra_cycles(), encoded_num_cycles);
    }
  }

  #[test]
  fn test_encoding_roundtrip() {
    for (syscall_code, _) in default_syscall_map::<MockHost>().iter() {
      assert_eq!(SyscallCode::from_u32(*syscall_code as u32), *syscall_code);
    }
  }

  #[test]
  /// Check that the Syscall number match the VM crate's.
  fn test_syscall_consistency_vm() {
    for code in SyscallCode::iter() {
      match code {
        SyscallCode::HALT => assert_eq!(code as u32, athena_vm::syscalls::HALT),
        SyscallCode::WRITE => assert_eq!(code as u32, athena_vm::syscalls::WRITE),
        SyscallCode::HOST_READ => assert_eq!(code as u32, athena_vm::syscalls::HOST_READ),
        SyscallCode::HOST_WRITE => assert_eq!(code as u32, athena_vm::syscalls::HOST_WRITE),
        SyscallCode::HOST_CALL => assert_eq!(code as u32, athena_vm::syscalls::HOST_CALL),
        SyscallCode::HINT_LEN => assert_eq!(code as u32, athena_vm::syscalls::HINT_LEN),
        SyscallCode::HINT_READ => assert_eq!(code as u32, athena_vm::syscalls::HINT_READ),
      }
    }
  }
}
