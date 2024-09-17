use athena_interface::StatusCode;

use crate::runtime::{Syscall, SyscallContext, SyscallResult};

pub struct SyscallHalt;

impl Syscall for SyscallHalt {
  fn execute(
    &self,
    _: &mut SyscallContext,
    exit_code: u32,
    _: u32,
  ) -> Result<SyscallResult, StatusCode> {
    log::debug!("Halt syscall with exit code {exit_code}",);
    Ok(SyscallResult::Exit(exit_code))
  }
}
