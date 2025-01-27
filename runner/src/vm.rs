use athena_core::runtime::ExecutionError;
use athena_interface::{
  payload::{ExecutionPayload, Payload},
  Address, AthenaContext, AthenaMessage, CallerBuilder, Decode, ExecutionResult, StatusCode,
};

use athena_sdk::{host::HostInterface, AthenaStdin, ExecutionClient};

// currently unused
#[derive(Debug, Clone, Copy)]
pub enum AthenaCapability {}

// currently unused
#[derive(Debug, Clone)]
pub enum AthenaOption {}

#[derive(Debug)]
pub enum SetOptionError {
  InvalidKey,
  InvalidValue,
}

#[derive(Debug)]
pub enum AthenaRevision {
  AthenaFrontier,
}

pub struct AthenaVm {
  client: ExecutionClient,
}

impl Default for AthenaVm {
  fn default() -> Self {
    Self::new()
  }
}

impl AthenaVm {
  pub fn new() -> Self {
    AthenaVm {
      client: ExecutionClient,
    }
  }

  pub fn get_capabilities(&self) -> Vec<AthenaCapability> {
    vec![]
  }

  pub fn set_option(&self, _option: AthenaOption, _value: &str) -> Result<(), SetOptionError> {
    Err(SetOptionError::InvalidKey)
  }

  pub fn execute<T: HostInterface>(
    &self,
    host: &mut T,
    _rev: AthenaRevision,
    msg: AthenaMessage,
    code: &[u8],
    caller_template: Address,
  ) -> ExecutionResult {
    let caller = CallerBuilder::new(msg.sender)
      .template(caller_template)
      .build();
    let mut context = AthenaContext::new(msg.recipient, caller, msg.depth);
    context.received = msg.value;

    let mut stdin = AthenaStdin::new();
    let execution_payload = match msg
      .input_data
      .map_or(Ok(ExecutionPayload::default()), |data| {
        ExecutionPayload::decode(&mut data.as_slice())
      }) {
      Ok(p) => p,
      Err(e) => {
        tracing::info!("Failed to deserialize execution payload: {e:?}");
        return ExecutionResult::new(StatusCode::Failure, 0, None);
      }
    };
    if !execution_payload.state.is_empty() {
      stdin.write_vec(execution_payload.state);
    }

    let Payload { selector, input } = execution_payload.payload;

    let input_len = input.len();
    if input_len > 0 {
      stdin.write_vec(input.clone());
    }
    let execution_result = match selector {
      Some(method) => {
        tracing::info!(
          "Executing method 0x{} with input length {}",
          method,
          input_len,
        );
        tracing::debug!("Input: 0x{}", hex::encode(input));
        self.client.execute_function(
          code,
          &method,
          stdin,
          Some(host),
          Some(msg.gas),
          Some(context),
        )
      }
      None => {
        tracing::info!("Executing default method with input length {}", input_len);
        tracing::debug!("Input: 0x{}", hex::encode(input));
        self
          .client
          .execute(code, stdin, Some(host), Some(msg.gas), Some(context))
      }
    };

    match execution_result {
      Ok((public_values, gas_left)) => ExecutionResult::new(
        StatusCode::Success,
        gas_left.unwrap(),
        Some(public_values.to_vec()),
      ),
      // map error to execution result
      Err(e) => {
        tracing::info!("Execution error: {e:?}");
        match e {
          ExecutionError::OutOfGas() => ExecutionResult::new(StatusCode::OutOfGas, 0, None),
          ExecutionError::SyscallFailed(code) => ExecutionResult::new(code, 0, None),
          // general error
          _ => ExecutionResult::new(StatusCode::Failure, 0, None),
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use athena_interface::{
    payload::{ExecutionPayloadBuilder, Payload},
    Address, AthenaMessage, Balance, Encode, MessageKind, MethodSelector,
  };
  use athena_sdk::host::MockHostInterface;
  const ADDRESS_ALICE: Address = Address([1u8; 24]);

  fn setup_logger() {
    let _ = tracing_subscriber::fmt()
      .with_test_writer()
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .try_init();
  }

  #[test]
  fn test_empty_code() {
    // construct a vm
    let result = AthenaVm::new().execute(
      &mut MockHostInterface::new(),
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        1000,
        Address::default(),
        Address::default(),
        None,
        Balance::default(),
      ),
      &[],
      Address::default(),
    );
    assert_eq!(StatusCode::Failure, result.status_code);
  }

  #[test]
  fn test_method_selector() {
    setup_logger();
    let elf = include_bytes!("../../tests/entrypoint/elf/entrypoint-test");

    let mut host = MockHostInterface::new();
    host
      .expect_call()
      .returning(|_| ExecutionResult::new(StatusCode::Success, 1000, None));
    let input = bincode::serialize(ADDRESS_ALICE.as_ref()).unwrap();

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
        Balance::default(),
      ),
      elf,
      Address::default(),
    );
    // this should fail: in noentrypoint mode, this panics.
    assert_eq!(result.status_code, StatusCode::Failure);

    // this will execute a specific method
    let payload = Payload {
      selector: Some(MethodSelector::from("athexp_test1")),
      input: input.clone(),
    };
    let payload = ExecutionPayloadBuilder::new().with_payload(payload).build();
    let result = AthenaVm::new().execute(
      &mut host,
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        1000000,
        Address::default(),
        Address::default(),
        Some(payload.encode()),
        Balance::default(),
      ),
      elf,
      Address::default(),
    );
    // this should succeed
    assert_eq!(result.status_code, StatusCode::Success);

    // this will execute a specific method
    let payload = Payload {
      selector: Some(MethodSelector::from("athexp_test2")),
      input: input.clone(),
    };
    let payload = ExecutionPayloadBuilder::new().with_payload(payload).build();
    let result = AthenaVm::new().execute(
      &mut host,
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        1000000,
        Address::default(),
        Address::default(),
        Some(payload.encode()),
        Balance::default(),
      ),
      elf,
      Address::default(),
    );
    // this should also succeed
    assert_eq!(result.status_code, StatusCode::Success);

    // this will execute a specific method
    let payload = Payload {
      selector: Some(MethodSelector::from("athexp_test3")),
      input: input.clone(),
    };
    let payload = ExecutionPayloadBuilder::new().with_payload(payload).build();
    let result = AthenaVm::new().execute(
      &mut host,
      AthenaRevision::AthenaFrontier,
      AthenaMessage::new(
        MessageKind::Call,
        0,
        1000000,
        Address::default(),
        Address::default(),
        Some(payload.encode()),
        Balance::default(),
      ),
      elf,
      Address::default(),
    );
    // this should fail, as this method is not `callable`
    assert_eq!(result.status_code, StatusCode::Failure);
  }

  #[test]
  fn test_minimal() {
    setup_logger();

    let client = ExecutionClient::new();
    let elf = include_bytes!("../../tests/minimal/getbalance.bin");
    let stdin = AthenaStdin::new();
    let mut host = MockHostInterface::new();
    host.expect_get_balance().return_const(9999u64);
    let caller = CallerBuilder::new(Address([0x88; 24])).build();
    let ctx = AthenaContext::new(ADDRESS_ALICE, caller, 0);
    let (mut output, _) = client
      .execute(elf, stdin, Some(&mut host), Some(1000), Some(ctx))
      .unwrap();
    let result = output.read::<Balance>();
    assert_eq!(result, 9999, "got wrong output value");
  }

  #[test]
  fn test_stack_depth() {
    setup_logger();

    let client = ExecutionClient::new();
    let elf = include_bytes!("../../tests/recursive_call/elf/recursive-call-test");
    let mut stdin = AthenaStdin::new();
    stdin.write(&(ADDRESS_ALICE.as_ref(), 7));
    let mut host = MockHostInterface::new();
    host
      .expect_call()
      .returning(|_| ExecutionResult::new(StatusCode::CallDepthExceeded, 0, None));
    let caller = CallerBuilder::new(Address([0x88; 24])).build();
    let ctx = AthenaContext::new(ADDRESS_ALICE, caller, 0);
    let res = client.execute(elf, stdin, Some(&mut host), Some(1_000_000), Some(ctx));
    assert!(matches!(
      res,
      Err(ExecutionError::SyscallFailed(StatusCode::CallDepthExceeded))
    ));
  }
}
