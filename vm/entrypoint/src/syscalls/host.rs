use athena_interface::{Address, StorageStatus};

/// Call a function in a foreign program.
///
/// `address` is the callee address, `input_ptr` is a bytearray to be passed to the
/// callee function, and `input_len` is the number of bytes to read from the input bytearray.
/// `amount` is the number of coins to transfer to the callee.
/// For now there is no return value and no return status code. The caller can assume
/// that, if this function returns, the call was successful.
///
/// See https://github.com/athenavm/athena/issues/5 for more information.
#[allow(unused_variables)]
pub fn call(
  address: &Address,
  input: &[u8],
  output: *mut u32,
  output_len: usize,
  amount: u64,
) -> usize {
  #[cfg(target_os = "zkvm")]
  unsafe {
    let size: usize;
    core::arch::asm!(
        "ecall",
        in("t0") crate::syscalls::HOST_CALL,
        in("a0") address,
        in("a1") input.as_ptr(),
        in("a2") input.len(),
        in("a3") output,
        in("a4") output_len,
        in("a5") &amount,
        lateout("t0") size,
    );
    return size;
  }

  #[cfg(not(target_os = "zkvm"))]
  unreachable!()
}

/// Read from host storage at the given address and key.
#[allow(unused_variables)]
pub fn read_storage(key: &[u32; 8]) -> [u32; 8] {
  #[cfg(target_os = "zkvm")]
  unsafe {
    let mut result = *key;
    core::arch::asm!(
        "ecall",
        in("t0") crate::syscalls::HOST_READ,
        in("a0") result.as_mut_ptr(),
    );
    return result;
  }

  #[cfg(not(target_os = "zkvm"))]
  unreachable!()
}

/// Write to host storage at the given address and key.
#[allow(unused_variables)]
pub fn write_storage(key: &[u32; 8], value: &[u32; 8]) -> StorageStatus {
  #[cfg(target_os = "zkvm")]
  unsafe {
    let status: u32;
    core::arch::asm!(
        "ecall",
        in("t0") crate::syscalls::HOST_WRITE,
        in("a0") key.as_ptr(),
        in("a1") value.as_ptr(),
        lateout("t0") status,
    );
    return status.try_into().unwrap();
  }

  #[cfg(not(target_os = "zkvm"))]
  unreachable!()
}

/// Get the current account balance
pub fn get_balance() -> u64 {
  #[cfg(target_os = "zkvm")]
  unsafe {
    let mut balance = std::mem::MaybeUninit::<u64>::uninit();
    core::arch::asm!(
        "ecall",
        in("t0") crate::syscalls::HOST_GETBALANCE,
        in("a0") balance.as_mut_ptr(),
    );
    balance.assume_init()
  }

  #[cfg(not(target_os = "zkvm"))]
  unreachable!()
}

/// Spawn a new instance of a template.
///
/// It either succeeds or reverts.
/// In case of success, returns the address of the spawned program.
/// The host calculates the new program address based on the template,
/// the state blob, and the principal nonce. The template and nonce are
/// available in its context and don't need to be passed here. The
/// owner of the new program is implicit in the blob, which is just a
/// serialized version of the instantiated template (struct) state.
///
/// The blob is a pointer to a serialized version of the instantiated template (struct) state.
/// The len is the number of **bytes** to read from the blob.
///
/// The address of spawned program is obtained via sharing a
/// variable located on the stack. The host must write the address,
/// initializing the variable.
#[allow(unused_variables)]
pub fn spawn(blob: &[u8]) -> athena_interface::Address {
  #[cfg(target_os = "zkvm")]
  {
    let mut result = std::mem::MaybeUninit::<athena_interface::Address>::uninit();

    unsafe {
      core::arch::asm!(
          "ecall",
          in("t0") crate::syscalls::HOST_SPAWN,
          in("a0") blob.as_ptr(),
          in("a1") blob.len(),
          in("a2") result.as_mut_ptr(),
      )
    }

    // SAFETY: the host initialized the data in the `result` variable
    // by writing to the memory address pointed to in the `a2` register.
    // In the case the host failed it would not return here.
    unsafe { result.assume_init() }
  }

  #[cfg(not(target_os = "zkvm"))]
  unreachable!()
}

/// Deploy a new template.
///
/// Returns the newly-deployed template address, which is calculated as the hash of the template code
///
/// The blob contains the template code.
///
/// The address of the deployed template is obtained via sharing a
/// variable located on the stack. The host must write the address,
/// initializing the variable.
#[allow(unused_variables)]
pub fn deploy(blob: &[u8]) -> athena_interface::Address {
  #[cfg(target_os = "zkvm")]
  {
    let mut result = std::mem::MaybeUninit::<athena_interface::Address>::uninit();

    unsafe {
      core::arch::asm!(
          "ecall",
          in("t0") crate::syscalls::HOST_DEPLOY,
          in("a0") blob.as_ptr(),
          in("a1") blob.len(),
          in("a2") result.as_mut_ptr(),
      )
    }

    unsafe { result.assume_init() }
  }

  #[cfg(not(target_os = "zkvm"))]
  unreachable!()
}
