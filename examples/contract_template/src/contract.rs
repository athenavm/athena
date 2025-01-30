//! A basic template to kickstart smart-contract development
//!
//! The contract implements two basic necessary methods:
//!  - spawn(immutable_state) used to spawn an instance of the contract (its account)
//!  - verify(tx, signature/witness) used to verify a TX before it's executed

use athena_interface::Address;

// Import proc macro which will generate boilerplate code:
//  - exported `extern "C"` functions for the `#[callable]` methods
//    along with registering them in `.note.athena_export` section so
//    the VM's disassembler can find them
//  - reading the method arguments from the IO
//  - writing the returned value to the IO
//
//  Method argument can be anything that implements `athena_vm::program::IntoArgument`.
//  NOTE: `IntoArgument` is automatically implemented for types that implement
//  `parity_scale_codec::Decode`.
//
//  Method return type can be anything that implements `athena_vm::program::IntoResult`.
//  NOTE: `IntoResult` is automatically implemented for types that implement
//  `parity_scale_codec::Encode`.
use athena_vm_declare::{callable, template};
use athena_vm_sdk::{spawn, Pubkey};
use parity_scale_codec::{Decode, Encode};

#[derive(Decode, Encode)]
struct Contract {
  owner: Pubkey,
}

#[template]
impl Contract {
  /// The spawn method is required and is used to spawn contract account using the
  /// SPAWN system call. This is defacto the constructor and it's supposed to pass
  /// the encoded (to bytes) intance of the contract to the SPAWN syscall. It is later
  /// passed to every method call as self.
  /// The VM host calculates the address of the account, persists the account
  /// and returns its address.
  ///
  /// NOTE: Pubkey and Address can be used here because they implement `Decode` and `Encode`
  /// respectively.
  #[callable]
  fn spawn(pubkey: Pubkey) -> Address {
    let instance = Contract { owner: pubkey };
    spawn(&instance.encode())
  }

  /// The verify method is required and is called before executing every method of the contract.
  /// It typically verifies the signature of the TX using the public key persisted in the
  /// account's immutable state.
  ///
  /// NOTE: `verify()` is not called for method called with the CALL syscall.
  #[callable]
  fn verify(&self, tx: Vec<u8>, signature: [u8; 64]) -> bool {
    athena_vm_sdk::precompiles::ed25519::verify(&tx, &self.owner.0, &signature)
  }
}
