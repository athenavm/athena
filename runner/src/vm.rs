use athcon_sys;
use athcon_vm::AthconVm;

struct FfiVm {
  instance: athcon_sys::athcon_vm,
}

impl AthconVm for FfiVm {
  fn init() -> Self {
    let instance = athcon_sys::athcon_vm {
      name: "FfiVm".as_ptr(),
      version: "0.1.0".as_ptr(),
      abi_version: 0,
      destroy: None,
      execute: None,
      get_capabilities: None,
      set_option: None,
    };
  }

  fn set_option(&mut self, _: &str, _: &str) -> Result<(), athcon_vm::SetOptionError> {
    unimplemented!()
  }

  fn execute<'a>(
          &self,
          revision: athcon_vm::Revision,
          code: &'a [u8],
          message: &'a athcon_vm::ExecutionMessage,
          context: Option<&'a mut athcon_vm::ExecutionContext<'a>>,
      ) -> athcon_vm::ExecutionResult {
    unimplemented!()
  }
}

struct MockVm {
    name: String,
    version: String,
    abi_version: i32,
}

impl MockVm {
  pub fn new(name: &str) -> Self {
    MockVm {
      name: name.to_string(),
      version: "0.1.0".to_string(),
      abi_version: 0,
    }
  }
}

impl AthconVm for MockVm {
  fn init() -> Self {
    MockVm::new("MockVm")
  }

  fn set_option(&mut self, _: &str, _: &str) -> Result<(), athcon_vm::SetOptionError> {
    unimplemented!()
  }

  fn execute(
    &self,
    _revision: athcon_sys::athcon_revision,
    _code: &[u8],
    _message: &athcon_vm::ExecutionMessage,
    _context: Option<&mut athcon_vm::ExecutionContext>,
  ) -> athcon_vm::ExecutionResult {
    athcon_vm::ExecutionResult::failure()
  }
}
