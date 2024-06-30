use crate::runtime::{Syscall, SyscallContext};
use athena_interface::HostInterface;

pub struct SyscallHalt;

impl SyscallHalt {
  pub const fn new() -> Self {
    Self
  }
}

impl<T> Syscall<T> for SyscallHalt
where
  T: HostInterface,
{
  fn execute(&self, ctx: &mut SyscallContext<T>, exit_code: u32, _: u32) -> Option<u32> {
    ctx.set_next_pc(0);
    ctx.set_exit_code(exit_code);
    None
  }
}
