#[cfg(target_os = "zkvm")]
use core::arch::asm;

/// Read from host storage at the given address and key.
///
/// The output is stored in the `key` pointer.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn read_storage(key: *mut u32, address: *const u32) {
  #[cfg(target_os = "zkvm")]
  unsafe {
    asm!(
        "ecall",
        in("t0") crate::syscalls::HOST_READ,
        in("a0") key,
        in("a1") address,
    )
  }

  #[cfg(not(target_os = "zkvm"))]
  unreachable!()
}

/// Write to host storage at the given address and key.
///
/// The result status code is stored in the `key` pointer.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn write_storage(key: *mut u32, address: *const u32, value: *const u32) {
  #[cfg(target_os = "zkvm")]
  unsafe {
    asm!(
        "ecall",
        in("t0") crate::syscalls::HOST_WRITE,
        in("a0") key,
        in("a1") address,
        in("a2") value,
    )
  }

  #[cfg(not(target_os = "zkvm"))]
  unreachable!()
}
