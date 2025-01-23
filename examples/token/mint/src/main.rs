#![no_main]

use athena_interface::{Address, Encode, MethodSelector};
use athena_vm::entrypoint;
use athena_vm_declare::{callable, template};
use athena_vm_sdk::{wallet::SpendArguments, Pubkey};
use parity_scale_codec::Decode;

/// 1000 Smidge == 1 tokens
const EXCHANGE_RATE: u64 = 1000;
const SUPPLY_KEY: [u32; 8] = [0; 8];

#[derive(Debug, Decode, Encode)]
struct Mint {
  owner: Pubkey,
  max_supply: u64,
}

athena_vm::entrypoint!();

#[template]
impl Mint {
  #[callable]
  fn spawn(mint: Mint) -> Address {
    let mut max_supply = [0u32; 8];
    max_supply[0] = mint.max_supply as u32;
    max_supply[1] = (mint.max_supply >> 32) as u32;
    athena_vm::syscalls::write_storage(&SUPPLY_KEY, &max_supply);
    athena_vm_sdk::spawn(mint.encode())
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
    let amount = ctx.received * EXCHANGE_RATE;
    if amount == 0 {
      return;
    }

    // 2. Check the remaining supply,
    //    calculate how much is bought,
    //    and update the remaining supply
    let mut supply_words = athena_vm::syscalls::read_storage(&SUPPLY_KEY);
    let mut supply = (supply_words[0] as u64) + ((supply_words[1] as u64) << 32);
    let bought = std::cmp::min(supply, amount);
    supply -= bought;
    supply_words[0] = supply as u32;
    supply_words[1] = (supply >> 32) as u32;
    athena_vm::syscalls::write_storage(&SUPPLY_KEY, &supply_words);

    // 3. Call receive() on the wallet.
    // It will increase its balance
    athena_vm_sdk::call(
      recipient,
      Some(wallet::ReceiveArguments { amount }.encode()),
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
