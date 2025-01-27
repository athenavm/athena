//! Token mint
//!
//! It allows exchanging SMH for the token,
//! which is then transfered to the chosen
//! wallet address.
//!
//! Every mint account instance represents a
//! unique token.
#![no_main]

use athena_interface::{Address, Encode, MethodSelector};
use athena_vm::entrypoint;
use athena_vm_declare::{callable, template};
use athena_vm_sdk::{wallet::SpendArguments, Pubkey};
use parity_scale_codec::Decode;

/// Storage for the number of already distributed tokens
const SUPPLY_KEY: [u32; 8] = [0; 8];
const TOKEN_IDENTIFIER_KEY: [u32; 8] = [1; 8];

#[derive(Debug, Decode, Encode)]
struct Mint {
  owner: Pubkey,
  max_supply: u64,
  /// token/smidge ratio (how many smidges per token)
  token_price: u64,
}

athena_vm::entrypoint!();

#[template]
impl Mint {
  #[callable]
  fn spawn(mint: Mint) -> Address {
    let address = athena_vm_sdk::spawn(&mint.encode());

    let mut token_identifier_words = [0u32; 8];
    for (i, c) in address.0.chunks_exact(4).enumerate() {
      token_identifier_words[i] = u32::from_le_bytes(c.try_into().unwrap());
    }
    athena_vm::syscalls::write_storage(&TOKEN_IDENTIFIER_KEY, &token_identifier_words);
    address
  }

  /// Buy exchanges SMH transfered along with the call into
  /// TOKENS sent to the `recipient` address.
  ///
  /// The `recipient` must be address of a wallet.
  #[callable]
  fn buy(&self, recipient: Address) {
    // 1. Check number of received coins
    //    and convert to tokens
    let ctx = athena_vm::syscalls::context::context();
    let amount = ctx.received / self.token_price;
    if amount == 0 {
      return;
    }

    // 2. Check the remaining supply,
    //    calculate how much is bought,
    //    and update the remaining supply
    let mut distributed_words = athena_vm::syscalls::read_storage(&SUPPLY_KEY);
    let mut distributed = (distributed_words[0] as u64) + ((distributed_words[1] as u64) << 32);
    let supply = self.max_supply.saturating_sub(distributed);
    let bought = std::cmp::min(supply, amount);
    distributed += bought;
    distributed_words[0] = distributed as u32;
    distributed_words[1] = (distributed >> 32) as u32;
    athena_vm::syscalls::write_storage(&SUPPLY_KEY, &distributed_words);

    let token_identifier_words = athena_vm::syscalls::read_storage(&TOKEN_IDENTIFIER_KEY);
    let token_identifier: [u8; 24] = bytemuck::cast_slice::<u32, u8>(&token_identifier_words)[..24]
      .try_into()
      .unwrap();

    // 3. Call receive() on the wallet.
    // It will increase its balance
    athena_vm_sdk::call(
      recipient,
      Some(
        wallet::ReceiveArguments {
          token_identifier: Address(token_identifier),
          amount,
        }
        .encode(),
      ),
      Some(MethodSelector::from("athexp_receive")),
      0,
    );

    // 4. TODO: send back SMH that wasn't spent?
  }

  #[callable]
  fn spend(&self, args: SpendArguments) {
    athena_vm_sdk::call(args.recipient, None, None, args.amount);
  }

  #[callable]
  fn verify(&self, tx: Vec<u8>, signature: [u8; 64]) -> bool {
    athena_vm_sdk::precompiles::ed25519::verify(&tx, &self.owner.0, &signature)
  }
}
