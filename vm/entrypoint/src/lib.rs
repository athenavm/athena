pub mod heap;
pub mod helpers;
pub mod syscalls;
pub mod host {
  pub use athena_hostfunctions::*;
}
pub mod io {
  pub use athena_precompiles::io::*;
}
pub mod precompiles {
  pub use athena_precompiles::*;
}
pub mod types {
  pub use athena_interface::*;
}

extern crate alloc;

#[macro_export]
macro_rules! entrypoint {
  ($path:path) => {
    const VM_ENTRY: fn() = $path;

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

  #[cfg(not(feature = "interface"))]
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

  #[cfg(feature = "rv32e")]
  core::arch::global_asm!(include_str!("memset-rv32e.s"));
  #[cfg(feature = "rv32e")]
  core::arch::global_asm!(include_str!("memcpy-rv32e.s"));
  #[cfg(not(feature = "rv32e"))]
  core::arch::global_asm!(include_str!("memset.s"));
  #[cfg(not(feature = "rv32e"))]
  core::arch::global_asm!(include_str!("memcpy.s"));

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
