use std::mem::MaybeUninit;

use athena_interface::Address;

#[repr(C)]
pub struct Context {
  pub received: u64,
  pub caller: Address,
  pub caller_template: Address,
}

pub fn context() -> Context {
  let ctx = MaybeUninit::<Context>::uninit();
  #[cfg(target_os = "zkvm")]
  unsafe {
    core::arch::asm!(
        "ecall",
        in("t0") crate::syscalls::HOST_CONTEXT,
        in("a0") ctx.as_ptr(),
    );
  }
  // SAFETY: the host initialized it.
  unsafe { ctx.assume_init() }
}
