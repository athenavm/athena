use std::mem::MaybeUninit;

use athena_interface::Context;

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
