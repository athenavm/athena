use std::{cell::RefCell, sync::Arc};

use crate::host::{AthenaCapability, AthenaOption, SetOptionError};
use athena_interface::{AthenaMessage, ExecutionResult, HostInterface, StatusCode};
use athena_sdk::{AthenaStdin, ExecutionClient};

pub trait VmInterface {
  fn get_capabilities(&self) -> Vec<AthenaCapability>;
  fn set_option(&self, option: AthenaOption, value: &str) -> Result<(), SetOptionError>;
  fn execute(
    &self,
    host: Arc<RefCell<dyn HostInterface>>,
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

impl VmInterface for AthenaVm {
  fn get_capabilities(&self) -> Vec<AthenaCapability> {
    vec![]
  }

  fn set_option(&self, _option: AthenaOption, _value: &str) -> Result<(), SetOptionError> {
    Err(SetOptionError::InvalidKey)
  }

  fn execute(
    &self,
    host: Arc<RefCell<dyn HostInterface>>,
    _rev: u32,
    _msg: AthenaMessage,
    // note: ignore _msg.code, should only be used on deploy
    code: &[u8],
  ) -> ExecutionResult {
    let mut stdin = AthenaStdin::new();
    stdin.write_vec(_msg.input_data);
    // TODO: pass execution context/callbacks into VM
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

  impl VmInterface for MockVm {
    fn get_capabilities(&self) -> Vec<AthenaCapability> {
      vec![]
    }

    fn set_option(&self, _option: AthenaOption, _value: &str) -> Result<(), SetOptionError> {
      Err(SetOptionError::InvalidKey)
    }

    fn execute(
      &self,
      host: Arc<RefCell<dyn HostInterface>>,
      _rev: u32,
      msg: AthenaMessage,
      _code: &[u8],
    ) -> ExecutionResult {
      // process a few basic messages
      // let host_interface = host.get_host();

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

    // construct a mock vm
    let vm = MockVm::new();
    let host_interface: Arc<RefCell<dyn athena_interface::HostInterface>> =
      Arc::new(RefCell::new(host));

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
        vec![],
        Balance::default(),
        vec![],
      ),
      &[],
    );
  }
}
