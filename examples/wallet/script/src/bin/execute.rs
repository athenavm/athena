//! Test harness for the Wallet program.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --package wallet-script --bin execute --release
//! ```

use std::error::Error;

use athena_interface::{
  Address, AthenaContext, Encode, HostDynamicContext, HostInterface, HostStaticContext,
  MethodSelector, MockHost, ADDRESS_ALICE, ADDRESS_CHARLIE,
};
use athena_sdk::{AthenaStdin, ExecutionClient};
use athena_vm_sdk::{wallet::SpendArguments, Pubkey};
use clap::Parser;

/// The ELF (executable and linkable format) file for the Athena RISC-V VM.
///
/// This file is generated by running `cargo athena build` inside the `program` directory.
pub const ELF: &[u8] = include_bytes!("../../../program/elf/wallet-template");

/// The arguments for the run command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct RunArgs {
  #[arg(
    long,
    default_value = "000dfb23b0979b4b000000000000000000000000000000000000000000000000",
    value_parser(parse_owner)
  )]
  owner: Pubkey,
}

fn parse_owner(data: &str) -> Result<Pubkey, hex::FromHexError> {
  let mut key = Pubkey::default();
  hex::decode_to_slice(data, &mut key.0 as &mut [u8])?;
  Ok(key)
}

fn spawn(host: &mut MockHost, owner: &Pubkey) -> Result<Address, Box<dyn Error>> {
  let mut stdin = AthenaStdin::new();
  stdin.write_vec(owner.encode());

  let method_selector = MethodSelector::from("athexp_spawn");

  let client = ExecutionClient::new();
  let (mut result, _) =
    client.execute_function(ELF, &method_selector, stdin, Some(host), None, None)?;

  Ok(Address::from(result.read::<[u8; 24]>()))
}

fn main() {
  tracing_subscriber::fmt::init();

  let args = RunArgs::parse();

  let mut host = MockHost::new_with_context(
    HostStaticContext::new(ADDRESS_ALICE, 0, ADDRESS_ALICE),
    HostDynamicContext::new(Address::default(), ADDRESS_ALICE),
  );
  host.set_balance(&ADDRESS_ALICE, 10000);
  let address = spawn(&mut host, &args.owner).expect("spawning wallet program");
  println!(
    "spawned a wallet program at {address} for {}",
    hex::encode(args.owner.0),
  );

  // send some coins
  let address_bob = Address::from([0xBB; 24]);
  let context = AthenaContext::new(ADDRESS_ALICE, address_bob, 0);

  let mut stdin = AthenaStdin::new();
  let wallet = host
    .get_program(&address)
    .expect("getting wallet program instance");

  let args = SpendArguments {
    recipient: ADDRESS_CHARLIE,
    amount: 120,
  };

  stdin.write_vec(wallet.clone());
  stdin.write_vec(args.encode());

  let alice_balance = host.get_balance(&ADDRESS_ALICE);
  assert!(alice_balance >= 120);
  println!(
    "sending {} coins {} -> {}",
    args.amount,
    context.address(),
    args.recipient,
  );
  // calculate method selector
  let method_selector = MethodSelector::from("athexp_spend");
  let max_gas = 25000;
  let (_, gas_left) = ExecutionClient::new()
    .execute_function(
      ELF,
      &method_selector,
      stdin,
      Some(&mut host),
      Some(max_gas),
      Some(context.clone()),
    )
    .expect("sending coins");

  let new_alice_balance = host.get_balance(&ADDRESS_ALICE);
  let charlie_balance = host.get_balance(&ADDRESS_CHARLIE);
  println!(
    "sent coins at gas cost {}, balances: alice: {}, charlie: {}",
    max_gas - gas_left.unwrap_or_default(),
    new_alice_balance,
    charlie_balance
  );
  assert!(gas_left.is_some());
  assert_eq!(charlie_balance, 120);
  assert_eq!(new_alice_balance, alice_balance - 120);
}

#[cfg(test)]
mod tests {
  use athena_interface::{
    payload::ExecutionPayloadBuilder, payload::Payload, Address, AthenaMessage, AthenaRevision,
    Balance, Encode, HostDynamicContext, HostStaticContext, MessageKind, MethodSelector, MockHost,
    StatusCode, VmInterface, ADDRESS_ALICE,
  };
  use athena_runner::AthenaVm;
  use athena_sdk::{AthenaStdin, ExecutionClient};
  use athena_vm_sdk::Pubkey;
  use ed25519_dalek::ed25519::signature::Signer;
  use ed25519_dalek::SigningKey;
  use rand::rngs::OsRng;

  fn setup_logger() {
    let _ = tracing_subscriber::fmt()
      .with_test_writer()
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .try_init();
  }

  #[test]
  fn deploy_template() {
    setup_logger();

    let mut host = MockHost::new_with_context(
      HostStaticContext::new(ADDRESS_ALICE, 0, ADDRESS_ALICE),
      HostDynamicContext::new(Address::default(), ADDRESS_ALICE),
    );
    let address = super::spawn(&mut host, &Pubkey::default()).unwrap();

    // deploy other contract
    let code = b"some really bad code".to_vec();
    let mut stdin = AthenaStdin::new();
    let wallet_state = host.get_program(&address).unwrap();
    stdin.write_vec(wallet_state.clone());
    stdin.write_vec(code.encode());

    let selector = MethodSelector::from("athexp_deploy");
    let result = ExecutionClient::new().execute_function(
      super::ELF,
      &selector,
      stdin.clone(),
      Some(&mut host),
      Some(25000000),
      None,
    );
    let (mut result, gas_cost) = result.unwrap();
    assert!(gas_cost.is_some());

    let address = result.read::<[u8; 24]>().into();
    let template = host.template(&address);
    assert_eq!(*template.unwrap(), code);
  }

  #[test]
  fn verifying_direct() {
    setup_logger();

    let mut host = MockHost::new_with_context(
      HostStaticContext::new(ADDRESS_ALICE, 0, ADDRESS_ALICE),
      HostDynamicContext::new(Address::default(), ADDRESS_ALICE),
    );

    let signing_key = SigningKey::generate(&mut OsRng);
    let owner = Pubkey(signing_key.verifying_key().to_bytes());
    let address = super::spawn(&mut host, &owner).unwrap();
    let wallet_state = host.get_program(&address).unwrap().clone();

    let tx = b"some really bad tx";

    // First try with invalid signature
    {
      let mut stdin = AthenaStdin::new();
      stdin.write_vec(wallet_state.clone());
      stdin.write_vec(tx.as_slice().encode());
      stdin.write_vec([0; 64].encode());

      let result = ExecutionClient::new().execute_function(
        super::ELF,
        &MethodSelector::from("athexp_verify"),
        stdin.clone(),
        Some(&mut host),
        Some(20000),
        None,
      );
      let (mut result, _) = result.unwrap();
      let valid = result.read::<bool>();
      assert!(!valid);
    }

    // Now with valid signature
    let signature = signing_key.sign(tx);

    let mut stdin = AthenaStdin::new();
    stdin.write_vec(wallet_state.clone());
    stdin.write_vec(tx.as_slice().encode());
    stdin.write_vec(signature.to_bytes().encode());

    let result = ExecutionClient::new().execute_function(
      super::ELF,
      &MethodSelector::from("athexp_verify"),
      stdin.clone(),
      Some(&mut host),
      Some(25000000),
      None,
    );
    let (mut result, _) = result.unwrap();
    let valid = result.read::<bool>();
    assert!(valid);
  }

  #[test]
  fn verifying_tx() {
    setup_logger();

    let mut host = MockHost::new_with_context(
      HostStaticContext::new(ADDRESS_ALICE, 0, ADDRESS_ALICE),
      HostDynamicContext::new(Address::default(), ADDRESS_ALICE),
    );

    let signing_key = SigningKey::generate(&mut OsRng);
    let owner = Pubkey(signing_key.verifying_key().to_bytes());
    let address = super::spawn(&mut host, &owner).unwrap();
    let wallet_state = host.get_program(&address).unwrap().clone();

    let tx = b"some really bad tx";

    let vm = AthenaVm::new();
    let mut host = MockHost::new_with_vm(&vm);

    // First try with invalid signature
    {
      // Construct the payload
      let verify_args = (tx.to_vec(), [0; 64]).encode();
      let payload = Payload::new(Some(MethodSelector::from("athexp_verify")), verify_args);
      let payload = ExecutionPayloadBuilder::new()
        .with_payload(payload)
        .with_state(wallet_state.clone())
        .build();
      let result = vm.execute(
        &mut host,
        AthenaRevision::AthenaFrontier,
        AthenaMessage::new(
          MessageKind::Call,
          0,
          25000000,
          Address::default(),
          Address::default(),
          Some(payload.into()),
          Balance::default(),
          vec![],
        ),
        super::ELF,
      );

      // the call should succeed, and it should return false
      assert_eq!(result.status_code, StatusCode::Success);

      let result = result.output.unwrap();
      assert_eq!(result.len(), 1);
      assert_eq!(result[0], 0);
    }

    // Now with valid signature
    let signature = signing_key.sign(tx);

    // Construct the payload
    let verify_args = (tx.to_vec(), signature.to_bytes()).encode();
    let payload = Payload::new(Some(MethodSelector::from("athexp_verify")), verify_args);
    let payload = ExecutionPayloadBuilder::new()
      .with_payload(payload)
      .with_state(wallet_state)
      .build();
    let result = vm.execute(
      &mut host,
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        25000000,
        Address::default(),
        Address::default(),
        Some(payload.into()),
        Balance::default(),
        vec![],
      ),
      super::ELF,
    );

    // the call should succeed, and it should return true
    assert_eq!(result.status_code, StatusCode::Success);

    let result = result.output.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 1);
  }

  #[test]
  fn max_spend() {
    setup_logger();

    let mut host = MockHost::new_with_context(
      HostStaticContext::new(ADDRESS_ALICE, 0, ADDRESS_ALICE),
      HostDynamicContext::new(Address::default(), ADDRESS_ALICE),
    );

    let address = super::spawn(&mut host, &Pubkey::default()).unwrap();
    let wallet_state = host.get_program(&address).unwrap();
    let recipient = Address::default();
    let amount = 100;

    let mut stdin = AthenaStdin::new();
    stdin.write_vec(wallet_state.clone());
    stdin.write_vec(athena_vm_sdk::wallet::encode_spend_inner(
      &recipient, amount,
    ));

    let selector = MethodSelector::from("athexp_max_spend");
    let result = ExecutionClient::new().execute_function(
      super::ELF,
      &selector,
      stdin,
      Some(&mut host),
      Some(25000000),
      None,
    );
    let (mut result, gas_cost) = result.unwrap();
    assert!(gas_cost.is_some());
    assert_eq!(result.read::<u64>(), amount);
  }
}
