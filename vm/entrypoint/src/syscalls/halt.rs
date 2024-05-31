use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "athena")] {
        use core::arch::asm;
    }
}

/// Halts the program.
#[allow(unused_variables)]
pub extern "C" fn syscall_halt(exit_code: u8) -> ! {
    #[cfg(target_os = "athena")]
    unsafe {
        asm!(
            "ecall",
            in("t0") crate::syscalls::HALT,
            in("a0") exit_code
        );
        unreachable!()
    }

    #[cfg(not(target_os = "athena"))]
    unreachable!()
}
