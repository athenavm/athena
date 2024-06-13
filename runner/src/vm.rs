use crate::host::{ExecutionResult, HostContext, HostInterface, MessageKind, StatusCode};
pub use athcon_vm::SetOptionError;

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
    host: *const dyn HostInterface,
    context: *mut dyn HostContext,
    rev: u32,
    msg: *const MessageKind,
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
    _host: *const dyn HostInterface,
    _context: *mut dyn HostContext,
    _rev: u32,
    _msg: *const MessageKind,
    _code: *const u8,
    _code_size: usize,
  ) -> ExecutionResult {
    ExecutionResult::new(
      StatusCode::Success,
      1337,
      None,
      None,
    )
  }
}
