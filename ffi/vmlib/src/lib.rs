use std::error::Error;

use athcon_declare::athcon_declare_vm;
use athcon_sys as ffi;
use athcon_vm::{
  AthconVm, ExecutionContext, ExecutionMessage as AthconExecutionMessage,
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
    let _ = tracing_subscriber::fmt()
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .try_init();
    Self {
      athena_vm: AthenaVm::new(),
    }
  }

  fn set_option(&mut self, _key: &str, _value: &str) -> Result<(), SetOptionError> {
    // we don't currently support any options
    Err(SetOptionError::InvalidKey)
  }

  /// `execute` is the main entrypoint from FFI. It's called from the macro-generated `__athcon_execute` fn.
  /// Note that we have to pass in a raw `context` pointer here. If we wrap it into the
  /// `AthenaExecutionContext` object inside the top-level FFI function and pass it in here, it causes
  /// lifetime issues.
  fn execute(
    &self,
    rev: Revision,
    code: &[u8],
    message: &AthconExecutionMessage,
    host: &ffi::athcon_host_interface,
    context: *mut ffi::athcon_host_context,
  ) -> AthconExecutionResult {
    // note that host context is allowed to be null. it's opaque and totally up to the host
    // whether and how to use it.
    if message.kind() != AthconMessageKind::ATHCON_CALL || code.is_empty() {
      return AthconExecutionResult::failure();
    }

    // Perform the conversion from `ffi::athcon_message` to `AthenaMessage`
    let athena_msg = AthenaMessageWrapper::from(message);

    // Unpack the context
    let execution_context = ExecutionContext::new(host, context);
    let mut host = WrappedHostInterface::new(execution_context);

    // Execute the code and proxy the result back to the caller
    let execution_result =
      self
        .athena_vm
        .execute(&mut host, RevisionWrapper::from(rev).0, athena_msg.0, code);
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
      ffi::athcon_status_code::ATHCON_INSUFFICIENT_INPUT => {
        StatusCodeWrapper(StatusCode::InsufficientInput)
      }
      ffi::athcon_status_code::ATHCON_INVALID_SYSCALL_ARGUMENT => {
        StatusCodeWrapper(StatusCode::InvalidSyscallArgument)
      }
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
      StatusCode::InsufficientInput => ffi::athcon_status_code::ATHCON_INSUFFICIENT_INPUT,
      StatusCode::InvalidSyscallArgument => {
        ffi::athcon_status_code::ATHCON_INVALID_SYSCALL_ARGUMENT
      }
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
  context: ExecutionContext<'a>,
}

impl<'a> WrappedHostInterface<'a> {
  fn new(context: ExecutionContext<'a>) -> Self {
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

  fn deploy(&mut self, code: Vec<u8>) -> Result<Address, Box<dyn Error>> {
    Ok(AddressWrapper::from(self.context.deploy(&code)).into())
  }
}
