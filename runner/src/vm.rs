use athena_core::runtime::ExecutionError;
use athena_interface::{
  AthenaCapability, AthenaContext, AthenaMessage, AthenaOption, AthenaRevision, ExecutionResult,
  HostInterface, SetOptionError, StatusCode, VmInterface,
};
use athena_sdk::{utils::setup_logger, AthenaStdin, ExecutionClient};

pub struct AthenaVm {
  client: ExecutionClient,
}

impl AthenaVm {
  pub fn new() -> Self {
    AthenaVm {
      client: ExecutionClient,
    }
  }
}

impl Default for AthenaVm {
  fn default() -> Self {
    Self::new()
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
    host: &mut T,
    _rev: AthenaRevision,
    msg: AthenaMessage,
    // note: ignore msg.code, should only be used on deploy
    code: &[u8],
  ) -> ExecutionResult {
    setup_logger();

    // construct context object
    let context = AthenaContext::new(msg.recipient, msg.sender, msg.depth);

    let mut stdin = AthenaStdin::new();

    // input data is optional
    if let Some(input_data) = msg.input_data {
      stdin.write_vec(input_data);
    }

    // method name is also optional
    let execution_result = if let Some(method_name) = msg.method {
      // TODO: use a fixed-length method selector here rather than a string
      // https://github.com/athenavm/athena/issues/113
      let method_name_str = match std::str::from_utf8(&method_name) {
        Ok(name) => name,
        Err(err) => {
          log::info!("malformed utf-8 method name: {:?}", err);
          return ExecutionResult::new(StatusCode::Failure, 0, None, None);
        }
      };
      log::info!("Executing method {} with input data", method_name_str);
      self.client.execute_function(
        code,
        method_name_str,
        stdin,
        Some(host),
        Some(msg.gas),
        Some(context),
      )
    } else {
      log::info!("Executing default method with input data");
      self
        .client
        .execute(code, stdin, Some(host), Some(msg.gas), Some(context))
    };

    match execution_result {
      Ok((public_values, gas_left)) => ExecutionResult::new(
        StatusCode::Success,
        gas_left.unwrap(),
        Some(public_values.to_vec()),
        None,
      ),
      // map error to execution result
      Err(e) => {
        log::info!("Execution error: {:?}", e);
        match e {
          ExecutionError::OutOfGas() => ExecutionResult::new(StatusCode::OutOfGas, 0, None, None),
          ExecutionError::SyscallFailed(code) => ExecutionResult::new(code, 0, None, None),
          // general error
          _ => ExecutionResult::new(StatusCode::Failure, 0, None, None),
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use athena_interface::{
    Address, AthenaMessage, AthenaRevision, Balance, MessageKind, MockHost, ADDRESS_ALICE,
    SOME_COINS, STORAGE_KEY, STORAGE_VALUE,
  };
  use athena_sdk::utils;

  #[test]
  #[should_panic]
  fn test_empty_code() {
    // construct a vm
    AthenaVm::new().execute(
      &mut MockHost::new(),
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        1000,
        Address::default(),
        Address::default(),
        None,
        None,
        Balance::default(),
        vec![],
      ),
      &[],
    );
  }

  #[test]
  fn test_method_selector() {
    let elf = include_bytes!("../../tests/entrypoint/elf/entrypoint-test");

    // deploy the contract to ADDRESS_ALICE and pass in the address so it can call itself recursively
    let vm = AthenaVm::new();
    let mut host = MockHost::new_with_vm(&vm);
    host.deploy_code(ADDRESS_ALICE, elf.to_vec());
    let input = bincode::serialize(&ADDRESS_ALICE).unwrap();

    // this will execute from the default entry point
    let result = AthenaVm::new().execute(
      &mut host,
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        1000000,
        Address::default(),
        Address::default(),
        None,
        None,
        Balance::default(),
        vec![],
      ),
      elf,
    );
    // this should fail: in noentrypoint mode, this panics.
    assert_eq!(result.status_code, StatusCode::Failure);

    // this will execute a specific method
    let result = AthenaVm::new().execute(
      &mut host,
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        1000000,
        Address::default(),
        Address::default(),
        Some(input),
        Some("athexp_test1".as_bytes().to_vec()),
        Balance::default(),
        vec![],
      ),
      elf,
    );
    // this should succeed
    assert_eq!(result.status_code, StatusCode::Success);

    // this will execute a specific method
    let result = AthenaVm::new().execute(
      &mut host,
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        1000000,
        Address::default(),
        Address::default(),
        None,
        Some("athexp_test2".as_bytes().to_vec()),
        Balance::default(),
        vec![],
      ),
      elf,
    );
    // this should also succeed
    assert_eq!(result.status_code, StatusCode::Success);

    // this will execute a specific method
    let result = AthenaVm::new().execute(
      &mut host,
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        1000000,
        Address::default(),
        Address::default(),
        None,
        Some("athexp_test3".as_bytes().to_vec()),
        Balance::default(),
        vec![],
      ),
      elf,
    );
    // this should fail, as this method is not `callable`
    assert_eq!(result.status_code, StatusCode::Failure);
  }

  // Note: we run this test here, as opposed to at a lower level (inside the SDK), since recursive host calls
  // require access to an actual VM instance.
  #[test]
  fn test_recursive_call() {
    utils::setup_logger();
    let client = ExecutionClient::new();
    let elf = include_bytes!("../../tests/recursive_call/elf/recursive-call-test");
    let mut stdin = AthenaStdin::new();
    stdin.write::<u32>(&7);
    let vm = AthenaVm::new();
    let mut host = MockHost::new_with_vm(&vm);
    host.deploy_code(ADDRESS_ALICE, elf.to_vec());
    let ctx = AthenaContext::new(ADDRESS_ALICE, ADDRESS_ALICE, 0);
    assert_eq!(
      host.get_storage(&ADDRESS_ALICE, &STORAGE_KEY),
      STORAGE_VALUE
    );
    let (mut output, _) = client
      .execute(
        elf,
        stdin,
        Some(&mut host),
        Some(150_000),
        Some(ctx.clone()),
      )
      .unwrap();
    let result = output.read::<u32>();
    assert_eq!(result, 13, "got wrong fibonacci value");

    // expect storage value to also have been updated
    assert_eq!(
      host.get_storage(&ADDRESS_ALICE, &STORAGE_KEY),
      [
        13u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0
      ]
    );
  }

  #[test]
  fn test_minimal() {
    utils::setup_logger();
    let client = ExecutionClient::new();
    let elf = include_bytes!("../../tests/minimal/getbalance.bin");
    let stdin = AthenaStdin::new();
    let vm = AthenaVm::new();
    let mut host = MockHost::new_with_vm(&vm);
    host.deploy_code(ADDRESS_ALICE, elf.to_vec());
    let ctx = AthenaContext::new(ADDRESS_ALICE, ADDRESS_ALICE, 0);
    let (mut output, _) = client
      .execute(elf, stdin, Some(&mut host), Some(1000), Some(ctx.clone()))
      .unwrap();
    let result = output.read::<Balance>();
    assert_eq!(result, SOME_COINS, "got wrong output value");
  }

  #[test]
  fn test_recursive_call_fail() {
    utils::setup_logger();
    let elf = include_bytes!("../../tests/recursive_call/elf/recursive-call-test");
    let vm = AthenaVm::new();

    // if we go any higher than in the previous test, we should run out of gas.
    // run the program entirely through the host this time. that will allow us to check storage reversion.
    // when the program fails (runs out of gas), the storage changes should be reverted.

    // trying to go any higher should result in an out-of-gas error
    let mut host = MockHost::new_with_vm(&vm);
    host.deploy_code(ADDRESS_ALICE, elf.to_vec());
    assert_eq!(
      host.get_storage(&ADDRESS_ALICE, &STORAGE_KEY),
      STORAGE_VALUE
    );
    let msg = AthenaMessage::new(
      MessageKind::Call,
      0,
      150_000,
      ADDRESS_ALICE,
      ADDRESS_ALICE,
      Some(vec![8u8, 0, 0, 0]),
      None,
      0,
      vec![],
    );
    let res = host.call(msg);
    assert_eq!(
      res.status_code,
      StatusCode::OutOfGas,
      "expected out of gas error"
    );

    // expect storage value changes to have been reverted
    assert_eq!(
      host.get_storage(&ADDRESS_ALICE, &STORAGE_KEY),
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
    let mut host = MockHost::new_with_vm(&vm);
    host.deploy_code(ADDRESS_ALICE, elf.to_vec());
    let ctx = AthenaContext::new(ADDRESS_ALICE, ADDRESS_ALICE, 0);
    let res = client.execute(elf, stdin, Some(&mut host), Some(1_000_000), Some(ctx));
    match res {
      Ok(_) => panic!("expected stack depth error"),
      Err(ExecutionError::SyscallFailed(StatusCode::CallDepthExceeded)) => (),
      Err(_) => panic!("expected stack depth error"),
    }
  }
}
