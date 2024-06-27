use std::{cell::RefCell, sync::Arc};

use athcon_declare::athcon_declare_vm;
use athcon_sys as ffi;
use athcon_vm::{
  AthconVm, ExecutionContext as AthconExecutionContext, ExecutionMessage as AthconExecutionMessage,
  ExecutionResult as AthconExecutionResult, MessageKind as AthconMessageKind, Revision,
  SetOptionError,
};
use athena_interface::{
  Address, AthenaMessage, Balance, Bytes32, ExecutionResult, HostInterface, MessageKind,
  StatusCode, StorageStatus, TransactionContext,
};
use athena_runner::{AthenaVm, Bytes32AsU64, VmInterface};

#[athcon_declare_vm("Athena", "athena1", "0.1.0")]
pub struct AthenaVMWrapper {
  // Internal, wrapped, Rust-native VM
  athena_vm: AthenaVm,
}

impl AthconVm for AthenaVMWrapper {
  fn init() -> Self {
    Self {
      athena_vm: AthenaVm::new(),
    }
  }

  fn set_option(&mut self, _key: &str, _value: &str) -> Result<(), SetOptionError> {
    // we don't currently support any options
    Err(SetOptionError::InvalidKey)
  }

  fn execute<'a>(
    &self,
    rev: Revision,
    code: &[u8],
    message: &AthconExecutionMessage,
    host: *const ffi::athcon_host_interface,
    context: *mut ffi::athcon_host_context,
  ) -> AthconExecutionResult {
    if host.is_null()
      || context.is_null()
      || message.kind() != AthconMessageKind::ATHCON_CALL
      || code.is_empty()
    {
      return AthconExecutionResult::failure();
    }

    // Perform the conversion from `ffi::athcon_message` to `AthenaMessage`
    let athena_msg = AthenaMessageWrapper::from(message);

    // Unpack the context
    let host_interface: &ffi::athcon_host_interface = unsafe { &*host };
    let execution_context = AthconExecutionContext::new(host_interface, context);
    let host = Arc::new(RefCell::new(WrappedHostInterface::new(execution_context)));

    // Execute the code and proxy the result back to the caller
    let execution_result = self.athena_vm.execute(host, rev as u32, athena_msg.0, code);
    ExecutionResultWrapper(execution_result).into()
  }
}

struct AddressWrapper(Address);

impl From<ffi::athcon_address> for AddressWrapper {
  fn from(address: ffi::athcon_address) -> Self {
    AddressWrapper(address.bytes)
  }
}

impl From<AddressWrapper> for Address {
  fn from(address: AddressWrapper) -> Self {
    address.0
  }
}

impl From<AddressWrapper> for ffi::athcon_address {
  fn from(address: AddressWrapper) -> Self {
    ffi::athcon_address { bytes: address.0 }
  }
}

struct Bytes32Wrapper(Bytes32);

impl From<Bytes32Wrapper> for ffi::athcon_bytes32 {
  fn from(bytes: Bytes32Wrapper) -> Self {
    ffi::athcon_bytes32 { bytes: bytes.0 }
  }
}

impl From<Bytes32Wrapper> for Bytes32 {
  fn from(bytes: Bytes32Wrapper) -> Self {
    bytes.0
  }
}

impl From<Bytes32Wrapper> for u64 {
  fn from(bytes: Bytes32Wrapper) -> Self {
    Bytes32AsU64::new(bytes.0).into()
  }
}

impl From<ffi::athcon_bytes32> for Bytes32Wrapper {
  fn from(bytes: ffi::athcon_bytes32) -> Self {
    Bytes32Wrapper(bytes.bytes)
  }
}

struct MessageKindWrapper(MessageKind);

impl From<ffi::athcon_call_kind> for MessageKindWrapper {
  fn from(kind: ffi::athcon_call_kind) -> Self {
    match kind {
      ffi::athcon_call_kind::ATHCON_CALL => MessageKindWrapper(MessageKind::Call),
    }
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

    let kind: MessageKindWrapper = item.kind.into();
    let byteswrapper: Bytes32Wrapper = item.value.into();
    AthenaMessageWrapper(AthenaMessage {
      kind: kind.0,
      depth: item.depth,
      gas: item.gas,
      recipient: AddressWrapper::from(item.recipient).into(),
      sender: AddressWrapper::from(item.sender).into(),
      input_data,
      value: Bytes32AsU64::new(byteswrapper.0).into(),
      code,
    })
  }
}

impl From<AthenaMessageWrapper> for ffi::athcon_message {
  fn from(item: AthenaMessageWrapper) -> Self {
    let input_data = item.0.input_data.as_ptr();
    let input_size = item.0.input_data.len();
    let code = item.0.code.as_ptr();
    let code_size = item.0.code.len();
    let kind = match item.0.kind {
      MessageKind::Call => ffi::athcon_call_kind::ATHCON_CALL,
    };
    let value: Bytes32AsU64 = item.0.value.into();
    ffi::athcon_message {
      kind,
      depth: item.0.depth,
      gas: item.0.gas,
      recipient: AddressWrapper(item.0.recipient).into(),
      sender: AddressWrapper(item.0.sender).into(),
      input_data,
      input_size,
      value: Bytes32Wrapper(value.into()).into(),
      code,
      code_size,
    }
  }
}

impl From<AthenaMessageWrapper> for AthconExecutionMessage {
  fn from(item: AthenaMessageWrapper) -> Self {
    // conversion is already implemented on the other side; utilize this
    AthconExecutionMessage::from(&ffi::athcon_message::from(item))
  }
}

impl From<&AthconExecutionMessage> for AthenaMessageWrapper {
  fn from(item: &AthconExecutionMessage) -> Self {
    let kind: MessageKindWrapper = item.kind().into();
    let byteswrapper = Bytes32Wrapper::from(*item.value());
    AthenaMessageWrapper(AthenaMessage {
      kind: kind.0,
      depth: item.depth(),
      gas: item.gas(),
      recipient: AddressWrapper::from(*item.recipient()).into(),
      sender: AddressWrapper::from(*item.sender()).into(),
      input_data: item.input().unwrap().clone(),
      value: Bytes32AsU64::new(byteswrapper.0).into(),
      code: item.code().unwrap().clone(),
    })
  }
}

struct StatusCodeWrapper(StatusCode);

impl From<StatusCodeWrapper> for StatusCode {
  fn from(status_code: StatusCodeWrapper) -> Self {
    status_code.0
  }
}

impl From<ffi::athcon_status_code> for StatusCodeWrapper {
  fn from(status_code: ffi::athcon_status_code) -> Self {
    match status_code {
      ffi::athcon_status_code::ATHCON_SUCCESS => StatusCodeWrapper(StatusCode::Success),
      ffi::athcon_status_code::ATHCON_FAILURE => StatusCodeWrapper(StatusCode::Failure),
      ffi::athcon_status_code::ATHCON_REVERT => StatusCodeWrapper(StatusCode::Revert),
      ffi::athcon_status_code::ATHCON_OUT_OF_GAS => StatusCodeWrapper(StatusCode::OutOfGas),
      ffi::athcon_status_code::ATHCON_UNDEFINED_INSTRUCTION => {
        StatusCodeWrapper(StatusCode::UndefinedInstruction)
      }
      ffi::athcon_status_code::ATHCON_INVALID_MEMORY_ACCESS => {
        StatusCodeWrapper(StatusCode::InvalidMemoryAccess)
      }
      ffi::athcon_status_code::ATHCON_CALL_DEPTH_EXCEEDED => {
        StatusCodeWrapper(StatusCode::CallDepthExceeded)
      }
      ffi::athcon_status_code::ATHCON_PRECOMPILE_FAILURE => {
        StatusCodeWrapper(StatusCode::PrecompileFailure)
      }
      ffi::athcon_status_code::ATHCON_CONTRACT_VALIDATION_FAILURE => {
        StatusCodeWrapper(StatusCode::ContractValidationFailure)
      }
      ffi::athcon_status_code::ATHCON_ARGUMENT_OUT_OF_RANGE => {
        StatusCodeWrapper(StatusCode::ArgumentOutOfRange)
      }
      ffi::athcon_status_code::ATHCON_INSUFFICIENT_BALANCE => {
        StatusCodeWrapper(StatusCode::InsufficientBalance)
      }
      ffi::athcon_status_code::ATHCON_INTERNAL_ERROR => {
        StatusCodeWrapper(StatusCode::InternalError)
      }
      ffi::athcon_status_code::ATHCON_REJECTED => StatusCodeWrapper(StatusCode::Rejected),
      ffi::athcon_status_code::ATHCON_OUT_OF_MEMORY => StatusCodeWrapper(StatusCode::OutOfMemory),
    }
  }
}

impl From<StatusCodeWrapper> for ffi::athcon_status_code {
  fn from(status_code: StatusCodeWrapper) -> Self {
    match status_code.0 {
      StatusCode::Success => ffi::athcon_status_code::ATHCON_SUCCESS,
      StatusCode::Failure => ffi::athcon_status_code::ATHCON_FAILURE,
      StatusCode::Revert => ffi::athcon_status_code::ATHCON_REVERT,
      StatusCode::OutOfGas => ffi::athcon_status_code::ATHCON_OUT_OF_GAS,
      StatusCode::UndefinedInstruction => ffi::athcon_status_code::ATHCON_UNDEFINED_INSTRUCTION,
      StatusCode::InvalidMemoryAccess => ffi::athcon_status_code::ATHCON_INVALID_MEMORY_ACCESS,
      StatusCode::CallDepthExceeded => ffi::athcon_status_code::ATHCON_CALL_DEPTH_EXCEEDED,
      StatusCode::PrecompileFailure => ffi::athcon_status_code::ATHCON_PRECOMPILE_FAILURE,
      StatusCode::ContractValidationFailure => {
        ffi::athcon_status_code::ATHCON_CONTRACT_VALIDATION_FAILURE
      }
      StatusCode::ArgumentOutOfRange => ffi::athcon_status_code::ATHCON_ARGUMENT_OUT_OF_RANGE,
      StatusCode::InsufficientBalance => ffi::athcon_status_code::ATHCON_INSUFFICIENT_BALANCE,
      StatusCode::InternalError => ffi::athcon_status_code::ATHCON_INTERNAL_ERROR,
      StatusCode::Rejected => ffi::athcon_status_code::ATHCON_REJECTED,
      StatusCode::OutOfMemory => ffi::athcon_status_code::ATHCON_OUT_OF_MEMORY,
    }
  }
}

struct ExecutionResultWrapper(ExecutionResult);

impl From<ExecutionResultWrapper> for ExecutionResult {
  fn from(wrapper: ExecutionResultWrapper) -> Self {
    wrapper.0
  }
}

impl From<AthconExecutionResult> for ExecutionResultWrapper {
  fn from(result: AthconExecutionResult) -> Self {
    ExecutionResultWrapper(ExecutionResult::new(
      StatusCodeWrapper::from(result.status_code()).into(),
      result.gas_left(),
      result.output().cloned(),
      result
        .create_address()
        .map(|address| AddressWrapper::from(*address).into()),
    ))
  }
}

impl From<ExecutionResultWrapper> for AthconExecutionResult {
  fn from(wrapper: ExecutionResultWrapper) -> Self {
    // use conversion implemented on the other side
    ffi::athcon_result::from(wrapper).into()
  }
}

impl From<ExecutionResultWrapper> for ffi::athcon_result {
  fn from(value: ExecutionResultWrapper) -> Self {
    let output = value.0.output.unwrap();
    let output_size = output.len();
    let output_data = output.as_ptr();
    let gas_left = value.0.gas_left;
    let create_address = value.0.create_address.map_or_else(
      || ffi::athcon_address::default(),
      |address| AddressWrapper(address).into(),
    );
    let status_code = StatusCodeWrapper(value.0.status_code).into();
    let release = None;
    ffi::athcon_result {
      output_data,
      output_size,
      gas_left,
      create_address,
      status_code,
      release,
    }
  }
}

struct AthenaVmWrapper {
  base: ffi::athcon_vm,
}

fn convert_storage_status(status: ffi::athcon_storage_status) -> StorageStatus {
  match status {
    ffi::athcon_storage_status::ATHCON_STORAGE_ASSIGNED => StorageStatus::StorageAssigned,
    ffi::athcon_storage_status::ATHCON_STORAGE_ADDED => StorageStatus::StorageAdded,
    ffi::athcon_storage_status::ATHCON_STORAGE_DELETED => StorageStatus::StorageDeleted,
    ffi::athcon_storage_status::ATHCON_STORAGE_MODIFIED => StorageStatus::StorageModified,
    ffi::athcon_storage_status::ATHCON_STORAGE_DELETED_ADDED => StorageStatus::StorageDeletedAdded,
    ffi::athcon_storage_status::ATHCON_STORAGE_MODIFIED_DELETED => {
      StorageStatus::StorageModifiedDeleted
    }
    ffi::athcon_storage_status::ATHCON_STORAGE_DELETED_RESTORED => {
      StorageStatus::StorageDeletedRestored
    }
    ffi::athcon_storage_status::ATHCON_STORAGE_ADDED_DELETED => StorageStatus::StorageAddedDeleted,
    ffi::athcon_storage_status::ATHCON_STORAGE_MODIFIED_RESTORED => {
      StorageStatus::StorageModifiedRestored
    }
  }
}

struct TransactionContextWrapper(ffi::athcon_tx_context);
impl From<TransactionContextWrapper> for TransactionContext {
  fn from(context: TransactionContextWrapper) -> Self {
    let tx_context = context.0;
    TransactionContext {
      gas_price: Bytes32Wrapper::from(tx_context.tx_gas_price).into(),
      origin: AddressWrapper::from(tx_context.tx_origin).into(),
      block_height: tx_context.block_height,
      block_timestamp: tx_context.block_timestamp,
      block_gas_limit: tx_context.block_gas_limit,
      chain_id: Bytes32Wrapper::from(tx_context.chain_id).into(),
    }
  }
}

struct WrappedHostInterface<'a> {
  context: AthconExecutionContext<'a>,
}

impl<'a> WrappedHostInterface<'a> {
  fn new(context: AthconExecutionContext<'a>) -> Self {
    WrappedHostInterface { context }
  }
}

impl<'a> HostInterface for WrappedHostInterface<'a> {
  fn account_exists(&self, addr: &Address) -> bool {
    self.context.account_exists(&AddressWrapper(*addr).into())
  }
  fn get_storage(&self, addr: &Address, key: &Bytes32) -> Bytes32 {
    let value_wrapper: Bytes32Wrapper = self
      .context
      .get_storage(&AddressWrapper(*addr).into(), &Bytes32Wrapper(*key).into())
      .into();
    value_wrapper.into()
  }
  fn set_storage(&mut self, addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
    convert_storage_status(self.context.set_storage(
      &AddressWrapper(*addr).into(),
      &Bytes32Wrapper(*key).into(),
      &Bytes32Wrapper(*value).into(),
    ))
  }
  fn get_balance(&self, addr: &Address) -> Balance {
    let balance = self.context.get_balance(&AddressWrapper(*addr).into());
    Bytes32AsU64::new(Bytes32Wrapper::from(balance).into()).into()
  }
  fn get_tx_context(&self) -> TransactionContext {
    TransactionContextWrapper(*self.context.get_tx_context()).into()
  }
  fn get_block_hash(&self, number: i64) -> Bytes32 {
    Bytes32Wrapper::from(self.context.get_block_hash(number)).into()
  }
  fn call(&mut self, msg: AthenaMessage) -> ExecutionResult {
    ExecutionResultWrapper::from(
      self
        .context
        .call(&AthconExecutionMessage::from(AthenaMessageWrapper(msg))),
    )
    .into()
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
    call: None,
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
    assert_eq!(
      std::ffi::CStr::from_ptr((*vm).name).to_str().unwrap(),
      "Athena",
      "VM name mismatch"
    );
    assert_eq!(
      std::ffi::CStr::from_ptr((*vm).version).to_str().unwrap(),
      "0.1.0",
      "Version mismatch"
    );

    let wrapper = &mut *(vm_ptr as *mut AthenaVmWrapper);

    // Test the FFI functions
    assert_eq!(
      (*vm).set_option.unwrap()(
        vm_ptr,
        "foo\0".as_ptr() as *const i8,
        "bar\0".as_ptr() as *const i8
      ),
      ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME
    );
    assert_eq!(
      (*vm).get_capabilities.unwrap()(vm_ptr),
      ffi::athcon_capabilities::ATHCON_CAPABILITY_Athena1 as u32
    );

    // Construct mock host, context, message, and code objects for test
    let host_interface = get_dummy_host_interface();
    let code = include_bytes!("../../../examples/hello_world/program/elf/hello-world-program");
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
      code_size: code.len(),
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
        code.len(),
      )
      .status_code,
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
        code.len(),
      )
      .status_code,
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
        code.len(),
      )
      .status_code,
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
        code.len(),
      )
      .status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_SUCCESS
    );

    // Call them a second way
    assert_eq!(
      wrapper.base.set_option.unwrap()(
        vm_ptr,
        "foo\0".as_ptr() as *const i8,
        "bar\0".as_ptr() as *const i8
      ),
      ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME
    );
    assert_eq!(
      wrapper.base.get_capabilities.unwrap()(vm_ptr),
      ffi::athcon_capabilities::ATHCON_CAPABILITY_Athena1 as u32
    );
    assert_eq!(
      wrapper.base.execute.unwrap()(
        vm_ptr,
        std::ptr::null(),
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        std::ptr::null(),
        std::ptr::null(),
        0
      )
      .status_code,
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
    vm_tests(athcon_create_athenavmwrapper());
  }
}
