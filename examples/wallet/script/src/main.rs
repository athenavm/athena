//! Test harness for the Wallet program.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --package wallet-script --bin execute --release
//! ```
mod host;
use host::*;

use std::error::Error;

use athena_interface::{Address, AthenaContext, Encode, MethodSelector};
use athena_sdk::{host::HostInterface, AthenaStdin, ExecutionClient};
use athena_vm_sdk::{wallet::SpendArguments, Pubkey};

/// The ELF (executable and linkable format) file for the Athena RISC-V VM.
///
/// This file is generated by running `cargo athena build` inside the `program` directory.
pub const ELF: &[u8] = include_bytes!("../../program/elf/wallet-template");
pub const ADDRESS_ALICE: Address = Address([1u8; 24]);
pub const ADDRESS_CHARLIE: Address = Address([3u8; 24]);

fn spawn(host: &mut MockHost, owner: &Pubkey) -> Result<Address, Box<dyn Error>> {
  let mut stdin = AthenaStdin::new();
  stdin.write_vec(owner.encode());

  let method_selector = MethodSelector::from("athexp_spawn");

  let max = 10_000;
  let client = ExecutionClient::new();
  let (mut result, gas) =
    client.execute_function(ELF, &method_selector, stdin, Some(host), Some(max), None)?;
  dbg!(max - gas.unwrap());
  Ok(Address::from(result.read::<[u8; 24]>()))
}

fn main() {
  tracing_subscriber::fmt::init();

  let owner = Pubkey::default();
  let mut host = MockHost::new(
    HostStaticContext::new(ADDRESS_ALICE, 0, ADDRESS_ALICE),
    HostDynamicContext::new(Address::default(), ADDRESS_ALICE),
  );
  host.set_balance(&ADDRESS_ALICE, 10000);
  let address = spawn(&mut host, &owner).expect("spawning wallet program");
  println!(
    "spawned a wallet program at {address} for {}",
    hex::encode(owner.0),
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

  stdin.write_slice(wallet);
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
    payload::ExecutionPayloadBuilder, payload::Payload, Address, AthenaMessage, Balance, Encode,
    MessageKind, MethodSelector, StatusCode,
  };
  use athena_runner::vm::AthenaRevision;
  use athena_runner::AthenaVm;
  use athena_sdk::host::MockHostInterface;
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

    let code = b"some really bad code".to_vec();
    let mut stdin = AthenaStdin::new();
    let owner = Pubkey::default();
    stdin.write_slice(&owner.0);
    stdin.write_vec(code.encode());

    let mut host = MockHostInterface::new();
    host.expect_deploy().returning(move |got| {
      assert_eq!(got, code);
      Ok(Address::from([0x66; 24]))
    });

    let selector = MethodSelector::from("athexp_deploy");
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

    let address = result.read::<[u8; 24]>();
    assert_eq!([0x66; 24], address);
  }

  #[test]
  fn verifying_direct() {
    setup_logger();

    let signing_key = SigningKey::generate(&mut OsRng);
    let owner = Pubkey(signing_key.verifying_key().to_bytes());
    let tx = b"some really bad tx".to_vec();

    // First try with invalid signature
    {
      let mut stdin = AthenaStdin::new();
      stdin.write_slice(&owner.0);
      stdin.write_vec(tx.encode());
      stdin.write_slice(&[0; 64]);

      let result = ExecutionClient::new().execute_function(
        super::ELF,
        &MethodSelector::from("athexp_verify"),
        stdin,
        Some(&mut MockHostInterface::new()),
        Some(20000),
        None,
      );
      let (mut result, _) = result.unwrap();
      let valid = result.read::<bool>();
      assert!(!valid);
    }

    // Now with valid signature
    let signature = signing_key.sign(&tx);

    let mut stdin = AthenaStdin::new();
    stdin.write_slice(&owner.0);
    stdin.write_vec(tx.encode());
    stdin.write_slice(&signature.to_bytes());

    let result = ExecutionClient::new().execute_function(
      super::ELF,
      &MethodSelector::from("athexp_verify"),
      stdin,
      Some(&mut MockHostInterface::new()),
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

    let signing_key = SigningKey::generate(&mut OsRng);
    let owner = Pubkey(signing_key.verifying_key().to_bytes());
    let tx = b"some really bad tx".to_vec();

    let vm = AthenaVm::new();
    let mut host = MockHostInterface::new();

    // First try with invalid signature
    {
      // Construct the payload
      let verify_args = (tx.clone(), [0; 64]).encode();
      let payload = Payload::new(Some(MethodSelector::from("athexp_verify")), verify_args);
      let payload = ExecutionPayloadBuilder::new()
        .with_payload(payload)
        .with_state(owner.0.as_slice())
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
    let signature = signing_key.sign(&tx);

    // Construct the payload
    let verify_args = (tx, signature.to_bytes()).encode();
    let payload = Payload::new(Some(MethodSelector::from("athexp_verify")), verify_args);
    let payload = ExecutionPayloadBuilder::new()
      .with_payload(payload)
      .with_state(owner.0.as_slice())
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

    let recipient = Address::default();
    let amount = 100;

    let mut stdin = AthenaStdin::new();
    stdin.write_slice(Pubkey::default().0.as_slice());
    stdin.write_vec(athena_vm_sdk::wallet::encode_spend_inner(
      &recipient, amount,
    ));

    let selector = MethodSelector::from("athexp_max_spend");
    let result = ExecutionClient::new().execute_function(
      super::ELF,
      &selector,
      stdin,
      Some(&mut MockHostInterface::new()),
      Some(25000000),
      None,
    );
    let (mut result, gas_cost) = result.unwrap();
    assert!(gas_cost.is_some());
    assert_eq!(result.read::<u64>(), amount);
  }
}
