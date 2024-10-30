//! The Spacemesh standard wallet template.
#![no_main]
#![no_std]
extern crate alloc;

use athena_interface::Address;
use athena_vm::entrypoint;
use athena_vm_declare::{callable, template};
use athena_vm_sdk::{call, spawn, Pubkey, SpendArguments, VerifiableTemplate, WalletProgram};
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

athena_vm::entrypoint!();

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
  fn spend(&self, args: SpendArguments) {
    // Send coins
    // Note: error checking happens inside the host
    call(args.recipient, None, None, args.amount);
  }

  fn proxy(&self, _destination: Address, _args: &[u8]) {
    unimplemented!();
  }

  #[callable]
  fn deploy(&self, code: alloc::vec::Vec<u8>) -> Address {
    athena_vm_sdk::deploy(code)
  }

  #[callable]
  fn max_spend(&self, args: SpendArguments) -> u64 {
    args.amount
  }
}

#[template]
impl VerifiableTemplate for Wallet {
  #[callable]
  fn verify(&self, tx: alloc::vec::Vec<u8>, signature: [u8; 64]) -> bool {
    athena_vm_sdk::precompiles::ed25519::verify(&tx, &self.owner.0, &signature)
  }
}
