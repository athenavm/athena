use crate::athconVm;

use std::ops::{Deref, DerefMut};

/// Container struct for ATHCON instances and user-defined data.
pub struct athconContainer<T>
where
    T: athconVm + Sized,
{
    #[allow(dead_code)]
    instance: ::athcon_sys::athcon_vm,
    vm: T,
}

impl<T> athconContainer<T>
where
    T: athconVm + Sized,
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
        Box::from_raw(instance as *mut athconContainer<T>)
    }

    /// Convert boxed self into an FFI pointer, surrendering ownership of the heap data.
    ///
    /// # Safety
    /// This function will return a valid instance pointer.
    pub unsafe fn into_ffi_pointer(boxed: Box<Self>) -> *mut ::athcon_sys::athcon_vm {
        Box::into_raw(boxed) as *mut ::athcon_sys::athcon_vm
    }
}

impl<T> Deref for athconContainer<T>
where
    T: athconVm,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.vm
    }
}

impl<T> DerefMut for athconContainer<T>
where
    T: athconVm,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vm
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::{ExecutionContext, ExecutionMessage, ExecutionResult};

    struct TestVm {}

    impl athconVm for TestVm {
        fn init() -> Self {
            TestVm {}
        }
        fn execute(
            &self,
            _revision: athcon_sys::athcon_revision,
            _code: &[u8],
            _message: &ExecutionMessage,
            _context: Option<&mut ExecutionContext>,
        ) -> ExecutionResult {
            ExecutionResult::failure()
        }
    }

    unsafe extern "C" fn get_dummy_tx_context(
        _context: *mut athcon_sys::athcon_host_context,
    ) -> athcon_sys::athcon_tx_context {
        athcon_sys::athcon_tx_context {
            tx_gas_price: Uint256::default(),
            tx_origin: Address::default(),
            block_coinbase: Address::default(),
            block_number: 0,
            block_timestamp: 0,
            block_gas_limit: 0,
            block_prev_randao: Uint256::default(),
            chain_id: Uint256::default(),
            block_base_fee: Uint256::default(),
            blob_base_fee: Uint256::default(),
            blob_hashes: std::ptr::null(),
            blob_hashes_count: 0,
            initcodes: std::ptr::null(),
            initcodes_count: 0,
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

        let code = [0u8; 0];

        let message = ::athcon_sys::athcon_message {
            kind: ::athcon_sys::athcon_call_kind::ATHCON_CALL,
            flags: 0,
            depth: 0,
            gas: 0,
            recipient: ::athcon_sys::athcon_address::default(),
            sender: ::athcon_sys::athcon_address::default(),
            input_data: std::ptr::null(),
            input_size: 0,
            value: ::athcon_sys::athcon_uint256be::default(),
            create2_salt: ::athcon_sys::athcon_bytes32::default(),
            code_address: ::athcon_sys::athcon_address::default(),
            code: std::ptr::null(),
            code_size: 0,
        };
        let message: ExecutionMessage = (&message).into();

        let host = ::athcon_sys::athcon_host_interface {
            account_exists: None,
            get_storage: None,
            set_storage: None,
            get_balance: None,
            get_code_size: None,
            get_code_hash: None,
            copy_code: None,
            selfdestruct: None,
            call: None,
            get_tx_context: Some(get_dummy_tx_context),
            get_block_hash: None,
            emit_log: None,
            access_account: None,
            access_storage: None,
            get_transient_storage: None,
            set_transient_storage: None,
        };
        let host_context = std::ptr::null_mut();

        let mut context = ExecutionContext::new(&host, host_context);
        let container = athconContainer::<TestVm>::new(instance);
        assert_eq!(
            container
                .execute(
                    athcon_sys::athcon_revision::ATHCON_PETERSBURG,
                    &code,
                    &message,
                    Some(&mut context)
                )
                .status_code(),
            ::athcon_sys::athcon_status_code::ATHCON_FAILURE
        );

        let ptr = unsafe { athconContainer::into_ffi_pointer(container) };

        let mut context = ExecutionContext::new(&host, host_context);
        let container = unsafe { athconContainer::<TestVm>::from_ffi_pointer(ptr) };
        assert_eq!(
            container
                .execute(
                    athcon_sys::athcon_revision::ATHCON_PETERSBURG,
                    &code,
                    &message,
                    Some(&mut context)
                )
                .status_code(),
            ::athcon_sys::athcon_status_code::ATHCON_FAILURE
        );
    }
}
