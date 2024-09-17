use crate::runtime::{Syscall, SyscallContext};

pub struct SyscallHalt;

impl Syscall for SyscallHalt {
  fn execute(&self, ctx: &mut SyscallContext, exit_code: u32, _: u32) -> Option<u32> {
    log::debug!("Halt syscall with exit code {}", exit_code);
    ctx.set_next_pc(0);
    ctx.set_exit_code(exit_code);
    None
  }
}
