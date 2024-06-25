// use athena_interface::{Address, Bytes32};

cfg_if::cfg_if! {
    if #[cfg(target_os = "zkvm")] {
        use core::arch::asm;
    }
}

/// Read from host storage at the given address and key.
///
/// The result is stored in the `address` pointer.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn host_read_storage(
  // result: *mut Bytes32,
  address: *mut u32,
  key: *const u32,
) -> *const u8 {
  // let result_ptr = result as *mut u32;
  #[cfg(target_os = "zkvm")]
  unsafe {
    asm!(
        "ecall",
        in("t0") crate::syscalls::HOST_READ,
        in("a0") address,
        in("a1") key,
    );
  }

  #[cfg(not(target_os = "zkvm"))]
  unreachable!()
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn host_write_storage(address: *const u8, key: *const u8, value: *const u8) -> u32 {
  #[cfg(target_os = "zkvm")]
  unsafe {
    asm!(
        "ecall",
        in("t0") crate::syscalls::HOST_WRITE,
        in("a0") address,
        in("a1") key,
        in("a2") value,
    );
  }

  #[cfg(not(target_os = "zkvm"))]
  unreachable!()
}
