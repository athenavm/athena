/// The runtime context struct stores information received as part of the inbound
/// execution message that's guaranteed not to change during the context of the
/// program execution.
use crate::Address;
pub struct AthenaContext {
  address: Address,
  caller: Address,
  depth: u32,
  max_depth: u32,
}

impl AthenaContext {
  pub fn new(address: Address, caller: Address, depth: u32, max_depth: u32) -> Self {
    AthenaContext {
      address,
      caller,
      depth,
      max_depth,
    }
  }

  pub fn address(&self) -> &Address {
    &self.address
  }

  pub fn caller(&self) -> &Address {
    &self.caller
  }

  pub fn depth(&self) -> u32 {
    self.depth
  }

  pub fn max_depth(&self) -> u32 {
    self.max_depth
  }
}
