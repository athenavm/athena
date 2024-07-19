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
    // note: ignore _msg.code, should only be used on deploy
    code: &[u8],
  ) -> ExecutionResult {
    // construct context object
    let context = AthenaContext::new(msg.recipient, msg.sender, msg.depth);

    let mut stdin = AthenaStdin::new();

    // input data is optional
    if let Some(input_data) = msg.input_data {
      stdin.write_vec(input_data);
    }

    // let (output, gas_left) = self
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
  use athena_interface::{Address, AthenaMessage, AthenaRevision, Balance, MessageKind, MockHost};

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
}
