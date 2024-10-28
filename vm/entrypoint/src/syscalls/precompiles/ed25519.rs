#[cfg(target_os = "zkvm")]
pub fn verify(message: &[u8], pubkey: &[u8; 32], signature: &[u8; 64]) -> u32 {
  unsafe {
    let valid: u32;
    std::arch::asm!(
        "ecall",
        in("t0") crate::syscalls::PRECOMPILE_ED25519_VERIFY,
        in("a0") message.as_ptr(),
        in("a1") message.len(),
        in("a2") pubkey.as_ptr(),
        in("a3") signature.as_ptr(),
        lateout("t0") valid,
    );
    valid
  }
}

#[cfg(not(target_os = "zkvm"))]
pub fn verify(_message: &[u8], _pubkey: &[u8; 32], _signature: &[u8; 64]) -> u32 {
  unimplemented!()
}
