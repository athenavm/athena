use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::runtime::{Outcome, Register, Syscall, SyscallContext, SyscallResult};

/// Verify signature
///
/// Inputs:
///  - a0 (arg1): pointer to signed message
///  - a1 (arg2): size of the signed message in bytes
///  - a2 (x12): pointer to 32B public key
///  - a2 (x13): pointer to 64B signature
pub(crate) struct SyscallEd25519Verify;

impl Syscall for SyscallEd25519Verify {
  fn execute(&self, ctx: &mut SyscallContext, msg_ptr: u32, msg_size: u32) -> SyscallResult {
    let pubkey = ctx.array(ctx.rt.register(Register::X12));
    let pubkey = VerifyingKey::from_bytes(&pubkey).unwrap();

    let signature = ctx.array(ctx.rt.register(Register::X13));
    let signature = Signature::from_bytes(&signature);

    let msg = ctx.bytes(msg_ptr, msg_size as usize);

    tracing::debug!(?msg, ?pubkey, ?signature, "verifying ED25519",);
    if pubkey.verify(&msg, &signature).is_ok() {
      tracing::debug!("signature is valid");
      return Ok(Outcome::Result(Some(1)));
    }
    tracing::debug!("signature is invalid");
    Ok(Outcome::Result(Some(0)))
  }

  fn num_extra_cycles(&self) -> u32 {
    // TODO: decide the cost, see https://github.com/athenavm/athena/issues/176
    100
  }
}
