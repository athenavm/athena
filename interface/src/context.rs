use bytemuck::NoUninit;

use crate::Address;

/// The runtime context struct stores information received as part of the inbound
/// execution message that's guaranteed not to change during the context of the
/// program execution.
#[derive(Debug, Clone)]
pub struct AthenaContext {
  pub callee: Address,
  pub caller: Caller,
  pub depth: u32,
  pub received: u64,
}

#[derive(Debug, Clone)]
pub struct Caller {
  pub account: Address,
  pub template: Option<Address>,
}

#[derive(Debug)]
pub struct CallerBuilder {
  caller: Caller,
}

impl CallerBuilder {
  pub fn new(address: Address) -> CallerBuilder {
    Self {
      caller: Caller {
        account: address,
        template: None,
      },
    }
  }

  pub fn build(self) -> Caller {
    self.caller
  }

  pub fn template(mut self, template: Address) -> Self {
    self.caller.template = Some(template);
    self
  }
}

impl AthenaContext {
  pub fn new(callee: Address, caller: Caller, depth: u32) -> Self {
    AthenaContext {
      callee,
      caller,
      depth,
      received: 0,
    }
  }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, NoUninit)]
pub struct Context {
  pub received: u64,
  pub caller: Address,
  pub caller_template: Address,
}
