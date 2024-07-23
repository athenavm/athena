use std::{cell::RefCell, sync::Arc};

use athena_core::runtime::ExecutionError;
use athena_interface::{
  AthenaCapability, AthenaContext, AthenaMessage, AthenaOption, AthenaRevision, ExecutionResult,
  HostInterface, HostProvider, SetOptionError, StatusCode, VmInterface,
};
use athena_sdk::{AthenaStdin, ExecutionClient};

pub struct AthenaVm {
  client: ExecutionClient,
}

impl AthenaVm {
  pub fn new() -> Self {
    AthenaVm {
      client: ExecutionClient::default(),
    }
  }
}

impl<T> VmInterface<T> for AthenaVm
where
  T: HostInterface,
{
  fn get_capabilities(&self) -> Vec<AthenaCapability> {
    vec![]
  }

  fn set_option(&self, _option: AthenaOption, _value: &str) -> Result<(), SetOptionError> {
    Err(SetOptionError::InvalidKey)
  }

  fn execute(
    &self,
    host: Arc<RefCell<HostProvider<T>>>,
    _rev: AthenaRevision,
    msg: AthenaMessage,
    // note: ignore msg.code, should only be used on deploy
    code: &[u8],
  ) -> ExecutionResult {
    // construct context object
    let context = AthenaContext::new(msg.recipient, msg.sender, msg.depth);

    let mut stdin = AthenaStdin::new();

    // input data is optional
    if let Some(input_data) = msg.input_data {
      stdin.write_vec(input_data);
    }

    match self
      .client
      .execute(&code, stdin, Some(host), Some(msg.gas), Some(context))
    {
      Ok((public_values, gas_left)) => ExecutionResult::new(
        StatusCode::Success,
        gas_left.unwrap(),
        Some(public_values.to_vec()),
        None,
      ),
      // map error to execution result
      Err(e) => match e {
        ExecutionError::OutOfGas() => ExecutionResult::new(StatusCode::OutOfGas, 0, None, None),
        ExecutionError::HostCallFailed(code) => ExecutionResult::new(code, 0, None, None),
        // general error
        _ => ExecutionResult::new(StatusCode::Failure, 0, None, None),
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{cell::RefCell, sync::Arc};

  use super::*;
  use athena_interface::{
    Address, AthenaMessage, AthenaRevision, Balance, MessageKind, MockHost, ADDRESS_ALICE,
    STORAGE_KEY, STORAGE_VALUE,
  };
  use athena_sdk::utils;

  #[test]
  #[should_panic]
  fn test_empty_code() {
    // construct a mock host
    let host = MockHost::new();
    let host_provider = HostProvider::new(host);
    let host_interface = Arc::new(RefCell::new(host_provider));

    // construct a vm
    AthenaVm::new().execute(
      host_interface,
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        1000,
        Address::default(),
        Address::default(),
        None,
        Balance::default(),
        vec![],
      ),
      &[],
    );
  }

  // Note: we run this test here, as opposed to at a lower level (inside the SDK), since recursive host calls
  // require access to an actual VM instance.
  #[test]
  fn test_recursive_call() {
    utils::setup_logger();
    let client = ExecutionClient::new();
    let elf = include_bytes!("../../tests/recursive_call/elf/recursive-call-test");
    let mut stdin = AthenaStdin::new();
    stdin.write::<u32>(&8);
    let vm = AthenaVm::new();
    let host = Arc::new(RefCell::new(HostProvider::new(MockHost::new_with_vm(&vm))));
    host.borrow_mut().deploy_code(ADDRESS_ALICE, elf);
    let ctx = AthenaContext::new(ADDRESS_ALICE, ADDRESS_ALICE, 0);
    assert_eq!(
      host.borrow().get_storage(&ADDRESS_ALICE, &STORAGE_KEY),
      STORAGE_VALUE
    );
    let (mut output, _) = client
      .execute::<MockHost>(
        elf,
        stdin,
        Some(host.clone()),
        Some(1_000_000),
        Some(ctx.clone()),
      )
      .unwrap();
    let result = output.read::<u32>();
    assert_eq!(result, 21, "got wrong fibonacci value");

    // expect storage value to also have been updated
    assert_eq!(
      host.borrow().get_storage(&ADDRESS_ALICE, &STORAGE_KEY),
      [
        21u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0
      ]
    );

    // trying to go any higher should result in an out-of-gas error
    let host = Arc::new(RefCell::new(HostProvider::new(MockHost::new_with_vm(&vm))));
    host.borrow_mut().deploy_code(ADDRESS_ALICE, elf);
    let mut stdin = AthenaStdin::new();
    stdin.write::<u32>(&9);
    assert_eq!(
      host.borrow().get_storage(&ADDRESS_ALICE, &STORAGE_KEY),
      STORAGE_VALUE
    );
    let res = client.execute::<MockHost>(
      elf,
      stdin,
      Some(host.clone()),
      Some(1_000_000),
      Some(ctx.clone()),
    );
    match res {
      Ok(_) => panic!("expected out-of-gas error"),
      Err(ExecutionError::HostCallFailed(StatusCode::OutOfGas)) => (),
      Err(_) => panic!("expected out-of-gas error"),
    }

    // expect storage value changes to have been reverted
    assert_eq!(
      host.borrow().get_storage(&ADDRESS_ALICE, &STORAGE_KEY),
      STORAGE_VALUE
    );
  }

  #[test]
  fn test_stack_depth() {
    utils::setup_logger();
    let client = ExecutionClient::new();
    let elf = include_bytes!("../../tests/stack_depth/elf/stack-depth-test");
    let stdin = AthenaStdin::new();
    let vm = AthenaVm::new();
    let host = Arc::new(RefCell::new(HostProvider::new(MockHost::new_with_vm(&vm))));
    host.borrow_mut().deploy_code(ADDRESS_ALICE, elf);
    let ctx = AthenaContext::new(ADDRESS_ALICE, ADDRESS_ALICE, 0);
    let res = client.execute::<MockHost>(elf, stdin, Some(host), Some(1_000_000), Some(ctx));
    match res {
      Ok(_) => panic!("expected stack depth error"),
      Err(ExecutionError::HostCallFailed(StatusCode::CallDepthExceeded)) => (),
      Err(_) => panic!("expected stack depth error"),
    }
  }
}
