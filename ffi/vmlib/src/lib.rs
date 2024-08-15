use std::{cell::RefCell, panic, sync::Arc};

use athcon_declare::athcon_declare_vm;
use athcon_sys as ffi;
use athcon_vm::{
  AthconVm, ExecutionContext as AthconExecutionContext, ExecutionMessage as AthconExecutionMessage,
  ExecutionResult as AthconExecutionResult, MessageKind as AthconMessageKind, Revision,
  SetOptionError,
};
use athena_interface::{
  Address, AthenaMessage, AthenaRevision, Balance, Bytes32, Bytes32AsU64, ExecutionResult,
  HostInterface, MessageKind, StatusCode, StorageStatus, TransactionContext, VmInterface,
};
use athena_runner::AthenaVm;

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

  /// `execute` is the main entrypoint from FFI. It's called from the macro-generated `__athcon_execute` fn.
  /// Note that we have to pass in raw `host` and `context` pointers here. If we wrap them into the
  /// `AthenaExecutionContext` object inside the top-level FFI function and pass it in here, it causes
  /// lifetime issues.
  unsafe fn execute<'a>(
    &self,
    rev: Revision,
    code: &[u8],
    message: &AthconExecutionMessage,
    host: *const ffi::athcon_host_interface,
    context: *mut ffi::athcon_host_context,
  ) -> AthconExecutionResult {
    // note that host context is allowed to be null. it's opaque and totally up to the host
    // whether and how to use it. but we require the host interface to be non-null.
    if host.is_null() || message.kind() != AthconMessageKind::ATHCON_CALL || code.is_empty() {
      return AthconExecutionResult::failure();
    }

    // Perform the conversion from `ffi::athcon_message` to `AthenaMessage`
    let athena_msg = AthenaMessageWrapper::from(message);

    // Unpack the context
    let host_interface: &ffi::athcon_host_interface = unsafe { &*host };
    let execution_context = AthconExecutionContext::new(host_interface, context);
    let host = WrappedHostInterface::new(execution_context);
    #[allow(clippy::arc_with_non_send_sync)]
    let host = Arc::new(RefCell::new(host));

    // Execute the code and proxy the result back to the caller
    let execution_result =
      self
        .athena_vm
        .execute(host, RevisionWrapper::from(rev).0, athena_msg.0, code);
    ExecutionResultWrapper(execution_result).into()
  }
}

struct RevisionWrapper(AthenaRevision);

impl From<Revision> for RevisionWrapper {
  fn from(rev: Revision) -> Self {
    match rev {
      Revision::ATHCON_FRONTIER => RevisionWrapper(AthenaRevision::AthenaFrontier),
    }
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
      Some(unsafe { std::slice::from_raw_parts(item.input_data, item.input_size) }.to_vec())
    } else {
      None
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
      depth: u32::try_from(item.depth).expect("Depth value out of range"),
      gas: u32::try_from(item.gas).expect("Gas value out of range"),
      recipient: AddressWrapper::from(item.recipient).into(),
      sender: AddressWrapper::from(item.sender).into(),
      input_data,
      value: Bytes32AsU64::new(byteswrapper.0).into(),
      code,
    })
  }
}

// probably not needed, but keeping it here for reference for now
// note: this code is memory safe, but would require manually freeing the input_data and code pointers.
// impl From<AthenaMessageWrapper> for ffi::athcon_message {
//   fn from(item: AthenaMessageWrapper) -> Self {
//     let (input_data, input_size) = if let Some(data) = item.0.input_data {
//       // need to transfer ownership of the data to the FFI
//       let boxed_data = data.into_boxed_slice();
//       let data_len = boxed_data.len();
//       let data_ptr = Box::into_raw(boxed_data) as *const u8;
//       (data_ptr, data_len)
//     } else {
//       (std::ptr::null(), 0)
//     };
//     let boxed_code = item.0.code.into_boxed_slice();
//     let code_size = boxed_code.len();
//     let code_ptr = Box::into_raw(boxed_code) as *const u8;
//     let kind = match item.0.kind {
//       MessageKind::Call => ffi::athcon_call_kind::ATHCON_CALL,
//     };
//     let value: Bytes32AsU64 = item.0.value.into();
//     ffi::athcon_message {
//       kind,
//       depth: item.0.depth as i32,
//       gas: item.0.gas as i64,
//       recipient: AddressWrapper(item.0.recipient).into(),
//       sender: AddressWrapper(item.0.sender).into(),
//       input_data,
//       input_size,
//       value: Bytes32Wrapper(value.into()).into(),
//       code: code_ptr,
//       code_size,
//     }
//   }
// }

impl From<AthenaMessageWrapper> for AthconExecutionMessage {
  fn from(item: AthenaMessageWrapper) -> Self {
    let kind = match item.0.kind {
      MessageKind::Call => ffi::athcon_call_kind::ATHCON_CALL,
    };
    let value: Bytes32AsU64 = item.0.value.into();
    let code = if !item.0.code.is_empty() {
      Some(item.0.code.as_slice())
    } else {
      None
    };
    AthconExecutionMessage::new(
      kind,
      item.0.depth as i32,
      item.0.gas as i64,
      AddressWrapper(item.0.recipient).into(),
      AddressWrapper(item.0.sender).into(),
      item.0.input_data.as_deref(),
      Bytes32Wrapper(value.into()).into(),
      code,
    )
  }
}

impl From<&AthconExecutionMessage> for AthenaMessageWrapper {
  fn from(item: &AthconExecutionMessage) -> Self {
    let kind: MessageKindWrapper = item.kind().into();
    let byteswrapper = Bytes32Wrapper::from(*item.value());
    AthenaMessageWrapper(AthenaMessage {
      kind: kind.0,
      depth: u32::try_from(item.depth()).expect("Depth value out of range"),
      gas: u32::try_from(item.gas()).expect("Gas value out of range"),
      recipient: AddressWrapper::from(*item.recipient()).into(),
      sender: AddressWrapper::from(*item.sender()).into(),
      input_data: item.input().cloned(),
      value: Bytes32AsU64::new(byteswrapper.0).into(),
      code: item.code().map_or(Vec::new(), |c| c.to_vec()),
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
      ffi::athcon_status_code::ATHCON_INVALID_INSTRUCTION => {
        StatusCodeWrapper(StatusCode::InvalidInstruction)
      }
      ffi::athcon_status_code::ATHCON_UNDEFINED_INSTRUCTION => {
        StatusCodeWrapper(StatusCode::UndefinedInstruction)
      }
      ffi::athcon_status_code::ATHCON_STACK_OVERFLOW => {
        StatusCodeWrapper(StatusCode::StackOverflow)
      }
      ffi::athcon_status_code::ATHCON_STACK_UNDERFLOW => {
        StatusCodeWrapper(StatusCode::StackUnderflow)
      }
      ffi::athcon_status_code::ATHCON_BAD_JUMP_DESTINATION => {
        StatusCodeWrapper(StatusCode::BadJumpDestination)
      }
      ffi::athcon_status_code::ATHCON_INVALID_MEMORY_ACCESS => {
        StatusCodeWrapper(StatusCode::InvalidMemoryAccess)
      }
      ffi::athcon_status_code::ATHCON_CALL_DEPTH_EXCEEDED => {
        StatusCodeWrapper(StatusCode::CallDepthExceeded)
      }
      ffi::athcon_status_code::ATHCON_STATIC_MODE_VIOLATION => {
        StatusCodeWrapper(StatusCode::StaticModeViolation)
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
      ffi::athcon_status_code::ATHCON_UNREACHABLE_INSTRUCTION => {
        StatusCodeWrapper(StatusCode::UnreachableInstruction)
      }
      ffi::athcon_status_code::ATHCON_TRAP => StatusCodeWrapper(StatusCode::Trap),
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
      StatusCode::InvalidInstruction => ffi::athcon_status_code::ATHCON_INVALID_INSTRUCTION,
      StatusCode::UndefinedInstruction => ffi::athcon_status_code::ATHCON_UNDEFINED_INSTRUCTION,
      StatusCode::StackOverflow => ffi::athcon_status_code::ATHCON_STACK_OVERFLOW,
      StatusCode::StackUnderflow => ffi::athcon_status_code::ATHCON_STACK_UNDERFLOW,
      StatusCode::BadJumpDestination => ffi::athcon_status_code::ATHCON_BAD_JUMP_DESTINATION,
      StatusCode::InvalidMemoryAccess => ffi::athcon_status_code::ATHCON_INVALID_MEMORY_ACCESS,
      StatusCode::CallDepthExceeded => ffi::athcon_status_code::ATHCON_CALL_DEPTH_EXCEEDED,
      StatusCode::StaticModeViolation => ffi::athcon_status_code::ATHCON_STATIC_MODE_VIOLATION,
      StatusCode::PrecompileFailure => ffi::athcon_status_code::ATHCON_PRECOMPILE_FAILURE,
      StatusCode::ContractValidationFailure => {
        ffi::athcon_status_code::ATHCON_CONTRACT_VALIDATION_FAILURE
      }
      StatusCode::ArgumentOutOfRange => ffi::athcon_status_code::ATHCON_ARGUMENT_OUT_OF_RANGE,
      StatusCode::UnreachableInstruction => ffi::athcon_status_code::ATHCON_UNREACHABLE_INSTRUCTION,
      StatusCode::Trap => ffi::athcon_status_code::ATHCON_TRAP,
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
      u32::try_from(result.gas_left()).expect("Gas value out of range"),
      result.output().cloned(),
      result
        .create_address()
        .map(|address| AddressWrapper::from(*address).into()),
    ))
  }
}

impl From<ExecutionResultWrapper> for AthconExecutionResult {
  fn from(wrapper: ExecutionResultWrapper) -> Self {
    AthconExecutionResult::new(
      StatusCodeWrapper(wrapper.0.status_code).into(),
      wrapper.0.gas_left as i64,
      wrapper.0.output.as_deref(),
    )
  }
}

// probably not needed, but keeping it here for reference for now
// note: this code is NOT MEMORY SAFE. it assumes that output lives at least as long as the result.
// otherwise, output_data will be a dangling pointer.
// impl From<ExecutionResultWrapper> for ffi::athcon_result {
//   fn from(value: ExecutionResultWrapper) -> Self {
//     let output = value.0.output.unwrap_or_else(Vec::new);
//     let output_size = output.len();

//     // in order to ensure that a slice can be reconstructed from empty output,
//     // we need some trickery here. see std::slice::from_raw_parts for more details.
//     let output_data = if output_size > 0 {
//       output.as_ptr()
//     } else {
//       core::ptr::NonNull::<u8>::dangling().as_ptr()
//     };

//     let gas_left = value.0.gas_left as i64;
//     let create_address = value.0.create_address.map_or_else(
//       || ffi::athcon_address::default(),
//       |address| AddressWrapper(address).into(),
//     );
//     let status_code = StatusCodeWrapper(value.0.status_code).into();
//     let release = None;
//     ffi::athcon_result {
//       output_data,
//       output_size,
//       gas_left,
//       create_address,
//       status_code,
//       release,
//     }
//   }
// }

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

  fn call(&mut self, msg: AthenaMessage) -> ExecutionResult {
    let execmsg = AthconExecutionMessage::from(AthenaMessageWrapper(msg));
    let res = ExecutionResultWrapper::from(self.context.call(&execmsg));
    // the execution message contains raw pointers that were passed over FFI and now need to be freed
    res.into()
  }

  fn spawn(&mut self, blob: Vec<u8>) -> Address {
    AddressWrapper::from(self.context.spawn(&blob)).into()
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
    spawn: None,
  }
}

// This code is shared with the external FFI tests
// These are raw tests, where the host context is null.
#[allow(clippy::missing_safety_doc)]
pub unsafe fn vm_tests(vm_ptr: *mut ffi::athcon_vm) {
  unsafe {
    // Ensure the returned pointer is not null
    assert!(!vm_ptr.is_null(), "VM creation returned a null pointer");

    // Perform additional checks on the returned VM instance
    let vm = &*vm_ptr;
    assert_eq!(vm.abi_version, 0, "ABI version mismatch");
    assert_eq!(
      std::ffi::CStr::from_ptr(vm.name).to_str().unwrap(),
      "Athena",
      "VM name mismatch"
    );
    assert_eq!(
      std::ffi::CStr::from_ptr(vm.version).to_str().unwrap(),
      "0.1.0",
      "Version mismatch"
    );

    let wrapper = &mut *(vm_ptr as *mut AthenaVmWrapper);

    // Test the FFI functions
    assert_eq!(
      vm.set_option.unwrap()(
        vm_ptr,
        "foo\0".as_ptr() as *const std::os::raw::c_char,
        "bar\0".as_ptr() as *const std::os::raw::c_char
      ),
      ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME
    );
    assert_eq!(
      vm.get_capabilities.unwrap()(vm_ptr),
      ffi::athcon_capabilities::ATHCON_CAPABILITY_Athena1 as u32
    );

    // Construct mock host, context, message, and code objects for test
    let host_interface = get_dummy_host_interface();
    let code = include_bytes!("../../../examples/hello_world/program/elf/hello-world-program");
    let empty_code = &[0u8; 0];
    let message = ::athcon_sys::athcon_message {
      kind: ::athcon_sys::athcon_call_kind::ATHCON_CALL,
      depth: 0,
      gas: 10000,
      recipient: ::athcon_sys::athcon_address::default(),
      sender: ::athcon_sys::athcon_address::default(),
      input_data: std::ptr::null(),
      input_size: 0,
      value: ::athcon_sys::athcon_uint256be::default(),
      code: code.as_ptr(),
      code_size: code.len(),
    };

    // this message is invalid because code_size doesn't match code length
    let bad_message = ::athcon_sys::athcon_message {
      kind: ::athcon_sys::athcon_call_kind::ATHCON_CALL,
      depth: 0,
      gas: 0,
      recipient: ::athcon_sys::athcon_address::default(),
      sender: ::athcon_sys::athcon_address::default(),
      input_data: std::ptr::null(),
      input_size: 0,
      value: ::athcon_sys::athcon_uint256be::default(),
      code: std::ptr::null(),
      code_size: 1,
    };

    // note: we cannot check for a null instance or message pointer here, as the VM wrapper code
    // calls `std::process::abort()`. this is a violation of the athcon spec.
    // host pointer is allowed to be null.

    // fail due to null host pointer
    assert_eq!(
      vm.execute.unwrap()(
        vm_ptr,
        // host can be null
        std::ptr::null(),
        // host_context is an opaque pointer
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

    // fail due to empty code
    assert_eq!(
      vm.execute.unwrap()(
        vm_ptr,
        &host_interface,
        // host_context is an opaque pointer
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        &message,
        empty_code.as_ptr(),
        empty_code.len(),
      )
      .status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_FAILURE
    );

    // fail due to bad message
    // fails an assertion inside the VM macro code
    let result = panic::catch_unwind(|| {
      vm.execute.unwrap()(
        vm_ptr,
        &host_interface,
        // host_context is an opaque pointer
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        &bad_message,
        code.as_ptr(),
        code.len(),
      )
    });
    assert!(result.is_err(), "Expected panic did not occur");

    // this one should succeed
    // note that host needs to be non-null, but the host context can be null.
    // the VM is unopinionated about it and doesn't rely on it directly.
    assert_eq!(
      vm.execute.unwrap()(
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
        "foo\0".as_ptr() as *const std::os::raw::c_char,
        "bar\0".as_ptr() as *const std::os::raw::c_char
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
        // host can be null
        std::ptr::null(),
        // host_context is an opaque pointer
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

    // Cleanup: Destroy the VM instance to prevent memory leaks
    vm.destroy.unwrap()(vm_ptr);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_athcon_create() {
    unsafe { vm_tests(athcon_create_athenavmwrapper()) };
  }
}
