use athena_sdk::{AthenaStdin, ExecutionClient};
use crate::host::{
  AthenaCapability,
  AthenaMessage,
  AthenaOption,
  ExecutionContext,
  ExecutionResult,
  SetOptionError,
  StatusCode,
};

pub trait VmInterface {
  fn get_capabilities(&self) -> Vec<AthenaCapability>;
  fn set_option(&self, option: AthenaOption, value: &str) -> Result<(), SetOptionError>;
  fn execute(
    &self,
    host: ExecutionContext,
    // context: dyn HostInterface,
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
    _host: ExecutionContext,
    // unused
    // _context: *mut ffi::athcon_host_context,
    _rev: u32,
    _msg: AthenaMessage,
    // note: ignore _msg.code, should only be used on deploy
    code: &[u8],
  ) -> ExecutionResult {
    let mut stdin = AthenaStdin::new();
    stdin.write_vec(_msg.input_data);
    // TODO: pass execution context/callbacks into VM
    let output = self.client.execute(&code, stdin).unwrap();
    ExecutionResult::new(
      StatusCode::Success,
      1337,
      Some(output.to_vec()),
      None,
    )
  }
}
