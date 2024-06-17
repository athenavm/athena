use athena_runner::{
  Address,
  AthenaMessage,
  AthenaVm,
  Balance,
  Bytes32,
  ExecutionContext as AthenaExecutionContext,
  HostInterface as AthenaHostInterface,
  TransactionContext,
};
use athcon_sys as ffi;
use athcon_vm::{self, ExecutionContext};

// Implementation for destroying the VM instance
extern "C" fn destroy_vm(vm: *mut ffi::athcon_vm) {
  // Safety check to ensure the pointer is not null
  if vm.is_null() {
    return;
  }
  unsafe {
    let wrapper = &mut *(vm as *mut AthenaVmWrapper);
    drop(Box::from_raw(wrapper.vm));
    drop(Box::from_raw(wrapper));
  }
}

fn error_result() -> ffi::athcon_result {
  ffi::athcon_result {
    output_data: std::ptr::null_mut(),
    output_size: 0,
    gas_left: 0,
    create_address: ffi::athcon_address::default(),
    status_code: athcon_vm::StatusCode::ATHCON_FAILURE,
    release: None,
  }
}

struct AddressWrapper(ffi::athcon_address);
impl From<AddressWrapper> for Address {
  fn from(address: AddressWrapper) -> Self {
    address.0.bytes
  }
}

struct Bytes32Wrapper(ffi::athcon_bytes32);

impl From<Bytes32Wrapper> for Bytes32 {
  fn from(bytes: Bytes32Wrapper) -> Self {
    bytes.0.bytes
  }
}

struct AthenaMessageWrapper(AthenaMessage);
impl From<ffi::athcon_message> for AthenaMessageWrapper {
  fn from(item: ffi::athcon_message) -> Self {
    // Convert input_data pointer and size to Vec<u8>
    let input_data = if !item.input_data.is_null() && item.input_size > 0 {
      unsafe { std::slice::from_raw_parts(item.input_data, item.input_size) }.to_vec()
    } else {
      Vec::new()
    };

    // Convert code pointer and size to Vec<u8>
    let code = if !item.code.is_null() && item.code_size > 0 {
      unsafe { std::slice::from_raw_parts(item.code, item.code_size) }.to_vec()
    } else {
      Vec::new()
    };

    AthenaMessageWrapper(AthenaMessage{
      kind: item.kind.into(),
      depth: item.depth,
      gas: item.gas,
      recipient: AddressWrapper(item.recipient).into(),
      sender: AddressWrapper(item.sender).into(),
      input_data,
      value: Bytes32Wrapper(item.value).into(),
      code,
    })
  }
}

extern "C" fn execute_code(
  vm: *mut ffi::athcon_vm,
  host: *const ffi::athcon_host_interface,
  context: *mut ffi::athcon_host_context,
  rev: ffi::athcon_revision,
  msg: *const ffi::athcon_message,
  code: *const u8,
  code_size: usize,
) -> ffi::athcon_result {
  // First, check for null pointers
  // According to the spec these are optional but
  // we require all of msg, host, and vm
  if msg.is_null() || host.is_null() || vm.is_null() {
    return error_result();
  }

  // SAFETY: We've checked that the pointers aren't null, so it's safe to dereference
  unsafe {
    // Unpack the VM
    let wrapper = &mut *(vm as *mut AthenaVmWrapper);
    let athena_vm = &mut *(wrapper.vm);

    // Unpack the context
    let ec_raw: &ffi::athcon_host_interface = &*host;
    let ec = AthenaExecutionContext::new(ec_raw, context);

    // Convert the raw pointer to a reference
    let msg_ref: &ffi::athcon_message = &*msg;

    // Perform the conversion from `ffi::athcon_message` to `AthenaMessage`
    let athena_msg: AthenaMessageWrapper = (*msg_ref).into();

    // Execute the code and proxy the result back to the caller
    let execution_result = athena_vm.execute(
      ec,
      // context,
      rev as u32,
      athena_msg.0,
      code,
      code_size,
    );
    let athcon_result: *const ffi::athcon_result = execution_result.into();
    *athcon_result
  }
}

extern "C" fn get_capabilities(_vm: *mut ffi::athcon_vm) -> ffi::athcon_capabilities_flagset {
  // we only support one capability for now
  ffi::athcon_capabilities::ATHCON_CAPABILITY_Athena1 as u32
}

// Make this pub because it's not referenced inside the athcon_vm struct below,
// i.e., must be called separately
pub extern "C" fn result_dispose(result: *const ffi::athcon_result) {
  unsafe {
    if !result.is_null() {
      let owned = *result;
      Vec::from_raw_parts(
        owned.output_data as *mut u8,
        owned.output_size,
        owned.output_size,
      );
    }
  }
}

// Implementation for setting options of the VM instance
extern "C" fn set_option(
  _vm: *mut ffi::athcon_vm,
  _name: *const ::std::os::raw::c_char,
  _value: *const ::std::os::raw::c_char,
) -> ffi::athcon_set_option_result {
  // not currently supported
  return ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME;
}

struct AthenaVmWrapper {
  base: ffi::athcon_vm,
  vm: *mut AthenaVm,
}

// Main FFI "endpoint" called to create a new VM instance
#[no_mangle]
pub extern "C" fn athcon_create() -> *mut ffi::athcon_vm {
  let athena_vm = Box::new(AthenaVm::new());
  let wrapper = Box::new(AthenaVmWrapper {
    base: ffi::athcon_vm {
      abi_version: 0,
      name: "Athena VM\0".as_ptr() as *const ::std::os::raw::c_char,
      version: "0.1.0\0".as_ptr() as *const ::std::os::raw::c_char,
      destroy: Some(destroy_vm),
      execute: Some(execute_code),
      get_capabilities: Some(get_capabilities),
      set_option: Some(set_option),
    },
    vm: Box::into_raw(athena_vm),
  });
  Box::into_raw(wrapper) as *mut ffi::athcon_vm
}

unsafe extern "C" fn execute_call(
    _context: *mut ffi::athcon_host_context,
    _msg: *const ffi::athcon_message,
) -> ffi::athcon_result {
    // Some dumb validation for testing.
    let msg = *_msg;
    let success = if msg.input_size != 0 && msg.input_data.is_null() {
        false
    } else {
        msg.input_size != 0 || msg.input_data.is_null()
    };

    ffi::athcon_result {
        status_code: if success {
            athcon_vm::StatusCode::ATHCON_SUCCESS
        } else {
            athcon_vm::StatusCode::ATHCON_INTERNAL_ERROR
        },
        gas_left: 2,
        // NOTE: we are passing the input pointer here, but for testing the lifetime is ok
        output_data: msg.input_data,
        output_size: msg.input_size,
        release: None,
        create_address: athcon_vm::Address::default(),
    }
}

struct WrappedHostInterface {
  context: ExecutionContext,
}

impl WrappedHostInterface {
  fn new(context: ExecutionContext) -> Self {
    WrappedHostInterface {
      context: *context,
    }
  }
}

impl AthenaHostInterface for WrappedHostInterface {
  fn account_exists(&self, addr: &Address) -> bool {
    self.context.account_exists(address)
  }
  fn get_storage(&self, addr: &Address, key: &Bytes32) -> Bytes32 {
    self.context.get_storage(address, key)
  }
  fn set_storage(&mut self, addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
    self.context.set_storage(address, key, value)
  }
  fn get_balance(&self, addr: &Address) -> Balance {
    self.context.get_balance(address)
}
  fn get_tx_context(&self) -> TransactionContext {
    self.context.get_tx_context()
  }
  fn get_block_hash(&self, number: i64) -> Bytes32 {
    self.context.get_block_hash(number)
  }
  fn call(&mut self, msg: AthenaMessage) -> (Vec<u8>, i64, Address, StatusCode) {
    self.context.call(msg)
  }
}

impl From<ffi::athcon_host_interface> for WrappedHostInterface {
  fn from(interface: ffi::athcon_host_interface) -> Self {
    WrappedHostInterface::new(&interface)
  }
}

// TEST CODE follows
// should probably be moved into a separate module

unsafe extern "C" fn get_dummy_tx_context(
    _context: *mut ffi::athcon_host_context,
) -> ffi::athcon_tx_context {
    ffi::athcon_tx_context {
        tx_gas_price: athcon_vm::Uint256 { bytes: [0u8; 32] },
        tx_origin: athcon_vm::Address { bytes: [0u8; 24] },
        block_height: 42,
        block_timestamp: 235117,
        block_gas_limit: 105023,
        chain_id: athcon_vm::Uint256::default(),
    }
}

// Update these when needed for tests
fn get_dummy_host_interface() -> ffi::athcon_host_interface {
    ffi::athcon_host_interface {
        account_exists: None,
        get_storage: None,
        set_storage: None,
        get_balance: None,
        call: Some(execute_call),
        get_tx_context: Some(get_dummy_tx_context),
        get_block_hash: None,
    }
}

// This code is shared with the external FFI tests
pub fn vm_tests(vm_ptr: *mut ffi::athcon_vm) {
  unsafe {
    // Ensure the returned pointer is not null
    assert!(!vm_ptr.is_null(), "VM creation returned a null pointer");

    // Perform additional checks on the returned VM instance
    let vm = &*vm_ptr;
    assert_eq!((*vm).abi_version, 0, "ABI version mismatch");
    assert_eq!(std::ffi::CStr::from_ptr((*vm).name).to_str().unwrap(), "Athena VM", "VM name mismatch");
    assert_eq!(std::ffi::CStr::from_ptr((*vm).version).to_str().unwrap(), "0.1.0", "Version mismatch");

    let wrapper = &mut *(vm_ptr as *mut AthenaVmWrapper);

    // Test the FFI functions
    assert_eq!(
      (*vm).set_option.unwrap()(vm_ptr, "foo\0".as_ptr() as *const i8, "bar\0".as_ptr() as *const i8),
      ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME
    );
    assert_eq!((*vm).get_capabilities.unwrap()(vm_ptr), ffi::athcon_capabilities::ATHCON_CAPABILITY_Athena1 as u32);

    // Construct mock host, context, message, and code objects for test
    let host_interface = get_dummy_host_interface();
    // let host_context = std::ptr::null_mut();
    // let mut context = ExecutionContext::new(&host_interface, host_context);
    let code = [0u8; 0];
    let message = ::athcon_sys::athcon_message {
        kind: ::athcon_sys::athcon_call_kind::ATHCON_CALL,
        depth: 0,
        gas: 0,
        recipient: ::athcon_sys::athcon_address::default(),
        sender: ::athcon_sys::athcon_address::default(),
        input_data: std::ptr::null(),
        input_size: 0,
        value: ::athcon_sys::athcon_uint256be::default(),
        code: std::ptr::null(),
        code_size: 0,
    };

    // fail due to null vm pointer
    assert_eq!(
      (*vm).execute.unwrap()(
        std::ptr::null_mut(),
        &host_interface,
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        &message,
        code.as_ptr(),
        0,
      ).status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_FAILURE
    );

    // fail due to null host pointer
    assert_eq!(
      (*vm).execute.unwrap()(
        vm_ptr,
        std::ptr::null(),
        // host_context is unused and opaque
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        &message,
        code.as_ptr(),
        0,
      ).status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_FAILURE
    );

    // fail due to null msg
    assert_eq!(
      (*vm).execute.unwrap()(
        vm_ptr,
        &host_interface,
        // host_context is unused and opaque
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        std::ptr::null(),
        code.as_ptr(),
        0,
      ).status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_FAILURE
    );

    // this one should succeed
    assert_eq!(
      (*vm).execute.unwrap()(
        vm_ptr,
        &host_interface,
        // host_context is unused and opaque
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        &message,
        code.as_ptr(),
        0,
      ).status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_SUCCESS
    );

    // Call them a second way
    assert_eq!(
      wrapper.base.set_option.unwrap()(vm_ptr, "foo\0".as_ptr() as *const i8, "bar\0".as_ptr() as *const i8),
      ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME
    );
    assert_eq!(
      wrapper.base.get_capabilities.unwrap()(vm_ptr),
      ffi::athcon_capabilities::ATHCON_CAPABILITY_Athena1 as u32
    );
    assert_eq!(
      wrapper.base.execute.unwrap()(vm_ptr, std::ptr::null(), std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void, ffi::athcon_revision::ATHCON_FRONTIER, std::ptr::null(), std::ptr::null(), 0).status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_FAILURE
    );

    // Cleanup: Destroy the VM instance to prevent memory leaks
    (*vm).destroy.unwrap()(vm_ptr);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_athcon_create() {
    vm_tests(athcon_create());
  }
}
