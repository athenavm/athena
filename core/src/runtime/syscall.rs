use std::collections::HashMap;
use std::sync::Arc;

use athena_interface::StatusCode;
use strum_macros::EnumIter;

use crate::runtime::{Register, Runtime};
use crate::syscall::precompiles::ed25519::SyscallEd25519Verify;
use crate::syscall::{
  SyscallHalt, SyscallHintLen, SyscallHintRead, SyscallHostCall, SyscallHostContext,
  SyscallHostDeploy, SyscallHostGetBalance, SyscallHostRead, SyscallHostSpawn, SyscallHostWrite,
  SyscallWrite,
};

/// A system call is invoked by the the `ecall` instruction with a specific value in register t0.
/// The syscall number is a 32-bit integer, with the following layout (in little-endian format)
/// - The first byte is the syscall id.
/// - The second byte is 0/1 depending on whether the syscall has a separate table. This is used
///
/// In the CPU table to determine whether to lookup the syscall using the syscall interaction.
/// - The third byte is the number of additional cycles the syscall uses.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
#[allow(non_camel_case_types)]
pub enum SyscallCode {
  /// Halts the program.
  HALT = 0x00_00_00_00,

  /// Write to the output buffer.
  WRITE = 0x00_00_00_02,

  // Precompiles
  PRECOMPILE_ED25519_VERIFY = 0x00_64_00_20,

  /// Host functions
  HOST_READ = 0x00_00_00_A0,
  HOST_WRITE = 0x00_00_00_A1,
  HOST_CALL = 0x00_00_00_A2,
  HOST_GETBALANCE = 0x00_00_00_A3,
  HOST_SPAWN = 0x00_00_00_A4,
  HOST_DEPLOY = 0x00_00_00_A5,
  HOST_CONTEXT = 0x00_00_00_A6,

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
      0x00_64_00_20 => SyscallCode::PRECOMPILE_ED25519_VERIFY,
      0x00_00_00_A0 => SyscallCode::HOST_READ,
      0x00_00_00_A1 => SyscallCode::HOST_WRITE,
      0x00_00_00_A2 => SyscallCode::HOST_CALL,
      0x00_00_00_A3 => SyscallCode::HOST_GETBALANCE,
      0x00_00_00_A4 => SyscallCode::HOST_SPAWN,
      0x00_00_00_A5 => SyscallCode::HOST_DEPLOY,
      0x00_00_00_A6 => SyscallCode::HOST_CONTEXT,
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

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
  Result(Option<u32>),
  Exit(u32),
}

pub(crate) type SyscallResult = Result<Outcome, StatusCode>;

#[mockall::automock]
pub trait Syscall: Send + Sync {
  /// Execute the syscall and return the result.
  ///  `arg1` and `arg2` are the first two arguments to the syscall. These are the
  /// values in registers X10 and X11, respectively. The implementations might read more
  /// arguments from registers X12..X15.
  #[mockall::concretize]
  fn execute(&self, ctx: &mut SyscallContext, arg1: u32, arg2: u32) -> SyscallResult;

  /// The number of extra cycles that the syscall takes to execute. Unless this syscall is complex
  /// and requires many cycles, this should be zero.
  fn num_extra_cycles(&self) -> u32 {
    0
  }
}

/// A runtime for syscalls that is protected so that developers cannot arbitrarily modify the runtime.
pub struct SyscallContext<'a, 'h> {
  pub(crate) rt: &'a mut Runtime<'h>,
}

impl<'a, 'h> SyscallContext<'a, 'h> {
  pub fn new(runtime: &'a mut Runtime<'h>) -> Self {
    Self { rt: runtime }
  }

  pub fn mw(&mut self, addr: u32, value: u32) {
    assert_eq!(addr % 4, 0);
    self.rt.mw(addr, value);
  }

  pub fn mw_slice(&mut self, addr: u32, values: &[u32]) {
    assert_eq!(addr % 4, 0);
    for i in 0..values.len() {
      self.mw(addr + i as u32 * 4, values[i]);
    }
  }

  pub fn register(&self, register: Register) -> u32 {
    self.rt.register(register)
  }

  pub fn byte(&self, addr: u32) -> u8 {
    self.rt.byte(addr)
  }

  pub fn word(&self, addr: u32) -> u32 {
    assert_eq!(addr % 4, 0);
    self.rt.word(addr)
  }

  pub fn dword(&self, addr: u32) -> u64 {
    assert_eq!(addr % 4, 0);
    self.word(addr) as u64 | ((self.word(addr + 4) as u64) << 32)
  }

  pub fn slice(&self, addr: u32, len: usize) -> Vec<u32> {
    assert_eq!(addr % 4, 0);
    let mut values = Vec::new();
    for i in 0..len {
      values.push(self.word(addr + i as u32 * 4));
    }
    values
  }

  /// Read `len` bytes from `addr`.
  ///
  /// Supports unaligned read:
  /// - `addr` doesn't need to be aligned to 4B,
  /// - `len` doesn't need to be a multiple of 4.
  #[tracing::instrument(skip(self))]
  pub fn bytes(&self, mut addr: u32, len: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut bytes_to_read = len;

    // handle case when addr is not aligned to 4B
    let addr_offset = addr % 4;
    if addr_offset != 0 {
      tracing::debug!(addr, len, addr_offset, "addr not aligned");
      let word = self.word(addr - addr_offset).to_le_bytes();
      bytes.extend_from_slice(&word[addr_offset as usize..]);
      addr += bytes.len() as u32;
      bytes_to_read = bytes_to_read.saturating_sub(bytes.len());
    }

    for addr in (addr..addr + bytes_to_read as u32).step_by(4) {
      bytes.extend_from_slice(&self.word(addr).to_le_bytes());
    }
    bytes.truncate(len); // handle case when len is not a multiple of 4

    tracing::debug!(result = hex::encode(&bytes), "read bytes");
    bytes
  }

  pub fn array<const N: usize>(&self, addr: u32) -> [u8; N] {
    self.bytes(addr, N).try_into().unwrap()
  }
}

pub fn default_syscall_map() -> HashMap<SyscallCode, Arc<dyn Syscall>> {
  let mut syscall_map = HashMap::<SyscallCode, Arc<dyn Syscall>>::default();
  syscall_map.insert(SyscallCode::HALT, Arc::new(SyscallHalt {}));
  syscall_map.insert(SyscallCode::WRITE, Arc::new(SyscallWrite {}));

  syscall_map.insert(
    SyscallCode::PRECOMPILE_ED25519_VERIFY,
    Arc::new(SyscallEd25519Verify {}),
  );

  syscall_map.insert(SyscallCode::HOST_READ, Arc::new(SyscallHostRead {}));
  syscall_map.insert(SyscallCode::HOST_WRITE, Arc::new(SyscallHostWrite {}));
  syscall_map.insert(SyscallCode::HOST_CALL, Arc::new(SyscallHostCall {}));
  syscall_map.insert(
    SyscallCode::HOST_GETBALANCE,
    Arc::new(SyscallHostGetBalance {}),
  );
  syscall_map.insert(SyscallCode::HOST_SPAWN, Arc::new(SyscallHostSpawn {}));
  syscall_map.insert(SyscallCode::HOST_DEPLOY, Arc::new(SyscallHostDeploy {}));
  syscall_map.insert(SyscallCode::HINT_LEN, Arc::new(SyscallHintLen {}));
  syscall_map.insert(SyscallCode::HINT_READ, Arc::new(SyscallHintRead {}));
  syscall_map.insert(SyscallCode::HOST_CONTEXT, Arc::new(SyscallHostContext {}));

  syscall_map
}

#[cfg(test)]
mod tests {
  use crate::{
    runtime::{Program, Runtime},
    utils::AthenaCoreOpts,
  };

  use super::{default_syscall_map, SyscallCode, SyscallContext};
  use strum::IntoEnumIterator;

  #[test]
  fn test_syscalls_in_default_map() {
    let default_syscall_map = default_syscall_map();
    for code in SyscallCode::iter() {
      default_syscall_map.get(&code).unwrap();
    }
  }

  #[test]
  fn test_syscall_num_cycles_encoding() {
    for (syscall_code, syscall_impl) in default_syscall_map().iter() {
      let encoded_num_cycles = syscall_code.num_cycles();
      assert_eq!(syscall_impl.num_extra_cycles(), encoded_num_cycles);
    }
  }

  #[test]
  fn test_encoding_roundtrip() {
    for (syscall_code, _) in default_syscall_map().iter() {
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

        SyscallCode::PRECOMPILE_ED25519_VERIFY => {
          assert_eq!(code as u32, athena_vm::syscalls::PRECOMPILE_ED25519_VERIFY)
        }

        SyscallCode::HOST_READ => assert_eq!(code as u32, athena_vm::syscalls::HOST_READ),
        SyscallCode::HOST_WRITE => assert_eq!(code as u32, athena_vm::syscalls::HOST_WRITE),
        SyscallCode::HOST_CALL => assert_eq!(code as u32, athena_vm::syscalls::HOST_CALL),
        SyscallCode::HOST_GETBALANCE => {
          assert_eq!(code as u32, athena_vm::syscalls::HOST_GETBALANCE)
        }
        SyscallCode::HOST_SPAWN => assert_eq!(code as u32, athena_vm::syscalls::HOST_SPAWN),
        SyscallCode::HINT_LEN => assert_eq!(code as u32, athena_vm::syscalls::HINT_LEN),
        SyscallCode::HINT_READ => assert_eq!(code as u32, athena_vm::syscalls::HINT_READ),
        SyscallCode::HOST_DEPLOY => assert_eq!(code as u32, athena_vm::syscalls::HOST_DEPLOY),
        SyscallCode::HOST_CONTEXT => assert_eq!(code as u32, athena_vm::syscalls::HOST_CONTEXT),
      }
    }
  }

  #[test]
  fn reading_bytes_from_memory() {
    let mut rt = Runtime::new(Program::default(), None, AthenaCoreOpts::default(), None);
    // initialize memory
    let mut memory = Vec::<u8>::new();
    for (i, addr) in (0x100..0x200).step_by(4).enumerate() {
      let value = (9876 * i) as u32;
      rt.mw(addr, value);
      memory.extend_from_slice(&value.to_le_bytes());
    }

    let ctx = SyscallContext::new(&mut rt);
    let read = ctx.bytes(0x100, 0x100);
    assert_eq!(read, memory);

    // address not aligned
    let read = ctx.bytes(0x100 + 21, 0x100 - 21);
    assert_eq!(read, memory[21..]);

    // length not a multiple of 4
    let read = ctx.bytes(0x100, 27);
    assert_eq!(read, memory[..27]);

    // address not aligned and length not a multiple of 4
    let read = ctx.bytes(0x103, 59);
    assert_eq!(read, memory[3..3 + 59]);

    let read = ctx.bytes(0x1, 2);
    assert_eq!(read, memory[1..1 + 2]);
  }
}
