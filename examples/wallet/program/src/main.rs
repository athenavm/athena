//! The Spacemesh standard wallet template.
#![no_main]
extern crate alloc;

use athena_interface::Address;
use athena_vm_declare::{callable, template};
use athena_vm_sdk::{call, spawn, Pubkey, SendArguments, VerifiableTemplate, WalletProgram};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use parity_scale_codec::{Decode, Encode};
#[derive(Debug, Encode, Decode)]
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
  fn spawn(owner: Pubkey) -> Address {
    let wallet = Wallet::new(owner);
    let serialized = wallet.encode();
    spawn(serialized)
  }

  #[callable]
  fn send(&self, send_arguments: SendArguments) {
    // Send coins
    // Note: error checking happens inside the host
    println!(
      "Sending {} coins to {:?}",
      send_arguments.amount, send_arguments.recipient
    );
    call(send_arguments.recipient, None, None, send_arguments.amount);
  }

  fn proxy(&self, _destination: Address, _args: &[u8]) {
    unimplemented!();
  }

  #[callable]
  fn deploy(&self, code: alloc::vec::Vec<u8>) -> Address {
    athena_vm_sdk::deploy(code)
  }
}

impl VerifiableTemplate for Wallet {
  fn verify(&self, tx: &[u8], signature: &[u8; 64]) -> bool {
    // Check that the transaction is signed by the owner
    let public_key = VerifyingKey::from_bytes(&self.owner.0).unwrap();
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
    let owner = Pubkey([0u8; 32]);
    let wallet = Wallet::new(owner);

    Wallet::spawn(owner);
    let send_arguments = SendArguments {
      recipient: [0u8; 24],
      amount: 0,
    };
    wallet.send(send_arguments);
  }
}
