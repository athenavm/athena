extern crate alloc;

#[cfg(target_os = "zkvm")]
pub mod heap;

pub mod helpers;
pub mod io;
pub mod program;
pub mod syscalls;

pub mod types {
  pub use athena_interface::*;
}

/// Define the program entrypoint.
///
/// Configures the global allocator and an optional entrypoint function.
/// The entrypoint function is called by the runtime if no other method is
/// explicitly selected.
#[macro_export]
macro_rules! entrypoint {
  () => {
    fn main() {
      panic!("No entrypoint found");
    }
    entrypoint!(main);
  };
  ($entry:path) => {
    const VM_ENTRY: fn() = $entry;

    use $crate::heap::SimpleAlloc;

    #[global_allocator]
    static HEAP: SimpleAlloc = SimpleAlloc;

    mod vm_generated_main {
      #[no_mangle]
      fn main() {
        super::VM_ENTRY()
      }
    }
  };
}

#[cfg(target_os = "zkvm")]
mod vm {
  use crate::syscalls::syscall_halt;

  use getrandom::{register_custom_getrandom, Error};

  #[no_mangle]
  unsafe extern "C" fn __start() {
    {
      extern "C" {
        fn main();
      }
      main()
    }

    syscall_halt(0);
  }

  static STACK_TOP: u32 = 0x0020_0400;

  core::arch::global_asm!(
      r#"
    .section .text._start;
    .globl _start;
    _start:
        .option push;
        .option norelax;
        la gp, __global_pointer$;
        .option pop;
        la sp, {0}
        lw sp, 0(sp)
        call __start;
    "#,
      sym STACK_TOP
  );

  fn vm_getrandom(s: &mut [u8]) -> Result<(), Error> {
    unsafe {
      crate::syscalls::sys_rand(s.as_mut_ptr(), s.len());
    }

    Ok(())
  }

  register_custom_getrandom!(vm_getrandom);
}
