use crate::host::{
  AthenaCapability,
  AthenaMessage,
  AthenaOption,
  ExecutionContext,
  ExecutionResult,
  HostInterface,
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
    code: Vec<u8>,
  ) -> ExecutionResult;
}

pub struct AthenaVm {}

impl AthenaVm {
  pub fn new() -> Self {
    AthenaVm {}
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
    _code: Vec<u8>,
  ) -> ExecutionResult {


    ExecutionResult::new(
      StatusCode::Success,
      1337,
      None,
      None,
    )
  }
}
