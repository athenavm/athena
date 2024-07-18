use std::{cell::RefCell, sync::Arc};

use crate::host::{AthenaCapability, AthenaOption, SetOptionError};
use athena_core::runtime::ExecutionError;
use athena_interface::{
  AthenaContext, AthenaMessage, ExecutionResult, HostInterface, HostProvider, StatusCode,
};
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
  #[should_panic]
  fn test_empty_code() {
    // construct a mock host
    let host = MockHost::new(None);
    let host_provider = HostProvider::new(host);
    let host_interface = Arc::new(RefCell::new(host_provider));

    // construct a vm
    AthenaVm::new().execute(
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

  // #[test]
  // fn test_minimal_elf() {
  //   // construct a mock host
  //   let host = MockHost::new(None);
  //   let host_provider = HostProvider::new(host);
  //   let host_interface = Arc::new(RefCell::new(host_provider));

  //   // construct a vm
  //   let vm = AthenaVm::new();

  //   let execution_result = vm.execute(
  //     host_interface,
  //     0,
  //     AthenaMessage::new(
  //       MessageKind::Call,
  //       0,
  //       1000,
  //       Address::default(),
  //       Address::default(),
  //       None,
  //       Balance::default(),
  //       vec![],
  //     ),
  //     include_bytes!("../../tests/minimal/elf/minimal-test.elf"),
  //   );
  //   assert_eq!(execution_result.gas_left, 1000);
  //   assert_eq!(execution_result.status_code, StatusCode::Success);
  // }

  #[test]
  fn test_mock_vm() {
    // construct a mock host
    let host = MockHost::new(None);
    let host_provider = HostProvider::new(host);
    let host_interface = Arc::new(RefCell::new(host_provider));

    // construct a mock vm
    let vm = MockVm::new();

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
