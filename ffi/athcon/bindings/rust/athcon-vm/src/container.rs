use crate::AthconVm;

use std::ops::{Deref, DerefMut};

/// Container struct for ATHCON instances and user-defined data.
pub struct AthconContainer<T>
where
  T: AthconVm + Sized,
{
  #[allow(dead_code)]
  instance: ::athcon_sys::athcon_vm,
  vm: T,
}

impl<T> AthconContainer<T>
where
  T: AthconVm + Sized,
{
  /// Basic constructor.
  pub fn new(_instance: ::athcon_sys::athcon_vm) -> Box<Self> {
    Box::new(Self {
      instance: _instance,
      vm: T::init(),
    })
  }

  /// Take ownership of the given pointer and return a box.
  ///
  /// # Safety
  /// This function expects a valid instance to be passed.
  pub unsafe fn from_ffi_pointer(instance: *mut ::athcon_sys::athcon_vm) -> Box<Self> {
    assert!(!instance.is_null(), "from_ffi_pointer received NULL");
    Box::from_raw(instance as *mut AthconContainer<T>)
  }

  /// Convert boxed self into an FFI pointer, surrendering ownership of the heap data.
  ///
  /// # Safety
  /// This function will return a valid instance pointer.
  pub unsafe fn into_ffi_pointer(boxed: Box<Self>) -> *mut ::athcon_sys::athcon_vm {
    Box::into_raw(boxed) as *mut ::athcon_sys::athcon_vm
  }
}

impl<T> Deref for AthconContainer<T>
where
  T: AthconVm,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.vm
  }
}

impl<T> DerefMut for AthconContainer<T>
where
  T: AthconVm,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.vm
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::*;
  use crate::{ExecutionMessage, ExecutionResult};

  struct TestVm {}

  impl AthconVm for TestVm {
    fn init() -> Self {
      TestVm {}
    }
    fn execute(
      &self,
      _revision: athcon_sys::athcon_revision,
      _code: &[u8],
      _message: &ExecutionMessage,
      _host: &athcon_sys::athcon_host_interface,
      _context: *mut athcon_sys::athcon_host_context,
    ) -> ExecutionResult {
      ExecutionResult::failure()
    }
  }

  unsafe extern "C" fn get_dummy_tx_context(
    _context: *mut athcon_sys::athcon_host_context,
  ) -> athcon_sys::athcon_tx_context {
    athcon_sys::athcon_tx_context {
      tx_gas_price: 0,
      tx_origin: Address::default(),
      block_height: 0,
      block_timestamp: 0,
      block_gas_limit: 0,
      chain_id: Uint256::default(),
    }
  }

  #[test]
  fn container_new() {
    let instance = ::athcon_sys::athcon_vm {
      abi_version: ::athcon_sys::ATHCON_ABI_VERSION as i32,
      name: std::ptr::null(),
      version: std::ptr::null(),
      destroy: None,
      execute: None,
      get_capabilities: None,
      set_option: None,
    };

    let message = &::athcon_sys::athcon_message {
      kind: ::athcon_sys::athcon_call_kind::ATHCON_CALL,
      depth: 0,
      gas: 0,
      recipient: ::athcon_sys::athcon_address::default(),
      sender: ::athcon_sys::athcon_address::default(),
      sender_template: ::athcon_sys::athcon_address::default(),
      input_data: std::ptr::null(),
      input_size: 0,
      value: 0,
    };
    let message: ExecutionMessage = message.try_into().unwrap();

    let host = ::athcon_sys::athcon_host_interface {
      account_exists: None,
      get_storage: None,
      set_storage: None,
      get_balance: None,
      call: None,
      get_tx_context: Some(get_dummy_tx_context),
      get_block_hash: None,
      spawn: None,
      deploy: None,
    };
    let host_context = std::ptr::null_mut();

    let container = AthconContainer::<TestVm>::new(instance);
    assert_eq!(
      container
        .execute(
          athcon_sys::athcon_revision::ATHCON_FRONTIER,
          &[],
          &message,
          &host,
          host_context,
        )
        .status_code(),
      ::athcon_sys::athcon_status_code::ATHCON_FAILURE
    );

    let ptr = unsafe { AthconContainer::into_ffi_pointer(container) };

    let container = unsafe { AthconContainer::<TestVm>::from_ffi_pointer(ptr) };
    assert_eq!(
      container
        .execute(
          athcon_sys::athcon_revision::ATHCON_FRONTIER,
          &[],
          &message,
          &host,
          host_context,
        )
        .status_code(),
      ::athcon_sys::athcon_status_code::ATHCON_FAILURE
    );
  }
}
