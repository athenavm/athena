use std::{cell::RefCell, sync::Arc};

use crate::host::{AthenaCapability, AthenaOption, SetOptionError};
use athena_interface::{AthenaMessage, ExecutionResult, HostInterface, HostProvider, StatusCode};
use athena_sdk::{AthenaStdin, ExecutionClient};

pub trait VmInterface<T: HostInterface> {
  fn get_capabilities(&self) -> Vec<AthenaCapability>;
  fn set_option(&self, option: AthenaOption, value: &str) -> Result<(), SetOptionError>;
  fn execute(
    &self,
    host: Arc<RefCell<HostProvider<T>>>,
    rev: u32,
    msg: AthenaMessage,
    code: &[u8],
  ) -> ExecutionResult;
}

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
    _rev: u32,
    msg: AthenaMessage,
    // note: ignore _msg.code, should only be used on deploy
    code: &[u8],
  ) -> ExecutionResult {
    let mut stdin = AthenaStdin::new();

    // input data is optional
    if let Some(input_data) = msg.input_data {
      stdin.write_vec(input_data);
    }
    let output = self.client.execute(&code, stdin, Some(host)).unwrap();
    ExecutionResult::new(StatusCode::Success, 1337, Some(output.to_vec()), None)
  }
}

#[cfg(test)]
mod tests {
  use std::{cell::RefCell, sync::Arc};

  use super::*;
  use crate::host::MockHost;
  use crate::VmInterface;
  use athena_interface::{Address, AthenaMessage, Balance, MessageKind, StatusCode};

  struct MockVm {}

  impl MockVm {
    fn new() -> Self {
      MockVm {}
    }
  }

  impl<T> VmInterface<T> for MockVm
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
      _rev: u32,
      msg: AthenaMessage,
      _code: &[u8],
    ) -> ExecutionResult {
      // process a few basic messages

      // save context and perform a call

      // restore context

      // get block hash
      let output = host.borrow().get_block_hash(0);

      ExecutionResult::new(
        StatusCode::Success,
        msg.gas - 1,
        Some(output.to_vec()),
        None,
      )
    }
  }

  #[test]
  fn test_vm() {
    // construct a mock host
    let host = MockHost::new(None);
    let host_provider = HostProvider::new(host);

    // construct a mock vm
    let vm = MockVm::new();
    let host_interface = Arc::new(RefCell::new(host_provider));

    // test execution
    vm.execute(
      host_interface,
      0,
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
