//! The Spacemesh standard wallet template.
#![no_main]
#![no_std]

use athena_interface::Address;
use athena_vm_declare::{callable, template};
use athena_vm_sdk::{call, spawn, Pubkey, VerifiableTemplate, WalletProgram, PUBKEY_LENGTH};
use borsh::{from_slice, to_vec};
use borsh_derive::{BorshDeserialize, BorshSerialize};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use wallet_common::SendArguments;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Wallet {
  nonce: u64,
  balance: u64,
  owner: Pubkey,
}

impl Wallet {
  fn new(owner: Pubkey) -> Self {
    Wallet {
      nonce: 0,
      balance: 0,
      owner,
    }
  }
}

#[cfg(all(
  any(target_arch = "riscv32", target_arch = "riscv64"),
  target_feature = "e"
))]
#[template]
impl WalletProgram for Wallet {
  #[callable]
  // For now, callable methods must receive a blob.
  fn spawn(owner_blob: *const u8) {
    let obj = unsafe { core::slice::from_raw_parts(owner_blob, PUBKEY_LENGTH) };
    let owner = from_slice::<Pubkey>(&obj).expect("failed to deserialize owner pubkey");
    spawn(to_vec(&Wallet::new(owner)).expect("failed to serialize wallet"));
  }

  #[callable]
  // For now, callable methods must receive a blob.
  fn send(&self, send_arguments_blob: *const u8, send_arguments_blob_len: usize) {
    let obj = unsafe { core::slice::from_raw_parts(send_arguments_blob, send_arguments_blob_len) };
    let send_arguments =
      from_slice::<SendArguments>(&obj).expect("failed to deserialize send arguments");
    // Send coins
    // Note: error checking happens inside the host
    call(send_arguments.recipient, None, send_arguments.amount);
  }

  fn proxy(&self, _destination: Address, _args: &[u8]) {
    unimplemented!();
  }

  fn deploy(&self, _code: &[u8]) {
    unimplemented!();
  }
}

impl VerifiableTemplate for Wallet {
  fn verify(&self, tx: &[u8], signature: &[u8; 64]) -> bool {
    // Check that the transaction is signed by the owner
    let public_key = VerifyingKey::from_bytes(&self.owner).unwrap();
    let signature = Signature::from_bytes(signature);
    public_key.verify(&tx, &signature).is_ok()
  }
}

#[cfg(all(
  any(target_arch = "riscv32", target_arch = "riscv64"),
  target_feature = "e",
  test
))]
mod test {
  use super::*;

  #[test]
  fn test_wallet() {
    let wallet = Wallet::new([0u8; 32]);
    let owner: Pubkey = [0u8; 32];
    Wallet::spawn(owner.as_ptr());
    let send_arguments = SendArguments {
      recipient: [0u8; 24],
      amount: 0,
    };
    let serialized = to_vec(&send_arguments).unwrap();
    wallet.send(serialized.as_ptr(), serialized.len());
    unsafe { athexp_spawn(owner.as_ptr()) };
    let serialized = to_vec(&wallet).unwrap();
    unsafe { athexp_send(serialized.as_ptr(), serialized.len()) };
  }
}
