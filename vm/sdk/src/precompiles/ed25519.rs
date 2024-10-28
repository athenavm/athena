/// Verify ed25519 signature.
///
/// Returns `true` if signature is valid and `false` otherwise.
pub fn verify(message: &[u8], pubkey: &[u8; 32], signature: &[u8; 64]) -> bool {
  athena_vm::syscalls::precompiles::ed25519::verify(message, pubkey, signature) == 1
}
