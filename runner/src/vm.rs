use athena_core::runtime::ExecutionError;
use athena_interface::{
  payload::{ExecutionPayload, Payload},
  AthenaCapability, AthenaContext, AthenaMessage, AthenaOption, AthenaRevision, Decode,
  ExecutionResult, HostInterface, SetOptionError, StatusCode, VmInterface,
};

use athena_sdk::{AthenaStdin, ExecutionClient};

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
    // construct context object
    let context = AthenaContext::new(msg.recipient, msg.sender, msg.depth);

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
    Address, AthenaMessage, AthenaRevision, Balance, Encode, MessageKind, MethodSelector,
    MockHostInterface,
  };
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
        vec![],
      ),
      &[],
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
        vec![],
      ),
      elf,
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
        vec![],
      ),
      elf,
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
        vec![],
      ),
      elf,
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
        vec![],
      ),
      elf,
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
    let ctx = AthenaContext::new(ADDRESS_ALICE, ADDRESS_ALICE, 0);
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
    let ctx = AthenaContext::new(ADDRESS_ALICE, ADDRESS_ALICE, 0);
    let res = client.execute(elf, stdin, Some(&mut host), Some(1_000_000), Some(ctx));
    assert!(matches!(
      res,
      Err(ExecutionError::SyscallFailed(StatusCode::CallDepthExceeded))
    ));
  }
}
