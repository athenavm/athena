use crate::runtime::{Outcome, Syscall, SyscallContext, SyscallResult};

pub(crate) struct SyscallHalt;

impl Syscall for SyscallHalt {
  fn execute(&self, _: &mut SyscallContext, exit_code: u32, _: u32) -> SyscallResult {
    tracing::debug!("Halt syscall with exit code {exit_code}",);
    Ok(Outcome::Exit(exit_code))
  }
}
