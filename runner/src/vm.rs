use crate::host::{AthenaMessage, HostContext, MessageKind};
use athcon_sys as ffi;
pub use athcon_vm::{ExecutionContext, ExecutionResult, SetOptionError, StatusCode};

// currently unused
#[derive(Debug, Clone, Copy)]
pub enum Capability {}

// currently unused
#[derive(Debug, Clone)]
pub enum Option {}

pub trait VmInterface {
  fn get_capabilities(&self) -> Vec<Capability>;
  fn set_option(&self, option: Option, value: &str) -> Result<(), SetOptionError>;
  fn execute(
    &self,
    host: ExecutionContext,
    context: *mut ffi::athcon_host_context,
    rev: u32,
    msg: AthenaMessage,
    code: *const u8,
    code_size: usize,
  ) -> ExecutionResult;
}

pub struct AthenaVm {}

impl AthenaVm {
  pub fn new() -> Self {
    AthenaVm {}
  }
}

impl VmInterface for AthenaVm {
  fn get_capabilities(&self) -> Vec<Capability> {
    vec![]
  }

  fn set_option(&self, _option: Option, _value: &str) -> Result<(), SetOptionError> {
    Err(SetOptionError::InvalidKey)
  }

  fn execute(
    &self,
    _host: ExecutionContext,
    _context: *mut ffi::athcon_host_context,
    _rev: u32,
    _msg: AthenaMessage,
    _code: *const u8,
    _code_size: usize,
  ) -> ExecutionResult {
    ExecutionResult::new(
      StatusCode::ATHCON_SUCCESS,
      1337,
      None,
    )
  }
}
