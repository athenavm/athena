use std::error::Error;

use athena_interface::{
  payload::ExecutionPayload, Address, AthenaContext, AthenaMessage, AthenaRevision, Balance,
  Bytes32, ExecutionResult, HostInterface, StorageStatus, VmInterface,
};
use athena_runner::AthenaVm;
use athena_sdk::{AthenaStdin, ExecutionClient};

struct Host {
  code: Vec<u8>,
}

impl HostInterface for Host {
  fn get_storage(&self, _: &Address, _: &Bytes32) -> Bytes32 {
    unimplemented!()
  }

  fn set_storage(&mut self, _: &Address, _: &Bytes32, _: &Bytes32) -> StorageStatus {
    unimplemented!()
  }

  fn get_balance(&self, _: &Address) -> Balance {
    todo!()
  }

  fn call(&mut self, msg: AthenaMessage) -> ExecutionResult {
    let msg = AthenaMessage {
      input_data: Some(ExecutionPayload::encode_with_encoded_payload(
        [],
        msg.input_data.unwrap(),
      )),
      ..msg
    };

    AthenaVm::new().execute(
      self,
      AthenaRevision::AthenaFrontier,
      msg,
      &self.code.clone(),
    )
  }

  fn spawn(&mut self, _: Vec<u8>) -> Address {
    unimplemented!()
  }

  fn deploy(&mut self, _: Vec<u8>) -> Result<Address, Box<dyn Error>> {
    unimplemented!()
  }
}

fn setup_logger() {
  let _ = tracing_subscriber::fmt()
    .with_test_writer()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .try_init();
}

#[test]
fn recursive_calling() {
  setup_logger();
  let elf = include_bytes!("../elf/recursive-call-test");
  let mut stdin = AthenaStdin::new();
  let mut host = Host { code: elf.to_vec() };
  let template_address = Address::from([0x77; 24]);
  stdin.write(&(template_address.as_ref(), 6u32));

  let context = AthenaContext::new(template_address, template_address, 0);
  let result =
    ExecutionClient::new().execute(elf, stdin, Some(&mut host), Some(100000000), Some(context));

  let (mut output, _) = result.unwrap();

  let result = output.read::<u32>();
  // fibonacci(6) is 8
  assert_eq!(result, 8);
}

#[test]
fn gas_limiting() {
  setup_logger();
  let elf = include_bytes!("../elf/recursive-call-test");
  let mut stdin = AthenaStdin::new();
  let mut host = Host { code: elf.to_vec() };
  let template_address = Address::from([0x77; 24]);
  stdin.write(&(template_address.as_ref(), 6u32));

  let context = AthenaContext::new(template_address, template_address, 0);
  let result =
    ExecutionClient::new().execute(elf, stdin, Some(&mut host), Some(100), Some(context));
  assert!(result.is_err());
}
