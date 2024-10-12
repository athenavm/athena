#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::too_many_arguments)]

mod container;
mod types;

use core::slice;

pub use athcon_sys as ffi;
pub use container::AthconContainer;
pub use types::*;

/// Trait ATHCON VMs have to implement.
pub trait AthconVm {
  /// This is called once at initialisation time.
  fn init() -> Self;

  /// This is called for each supplied option.
  fn set_option(&mut self, _: &str, _: &str) -> Result<(), SetOptionError> {
    Ok(())
  }

  /// This is called for every incoming message.
  fn execute(
    &self,
    revision: Revision,
    code: &[u8],
    message: &ExecutionMessage,
    host: &ffi::athcon_host_interface,
    context: *mut ffi::athcon_host_context,
  ) -> ExecutionResult;
}

/// Error codes for set_option.
#[derive(Debug)]
pub enum SetOptionError {
  InvalidKey,
  InvalidValue,
}

/// ATHCON result structure.
#[derive(Debug)]
pub struct ExecutionResult {
  status_code: StatusCode,
  gas_left: i64,
  output: Option<Vec<u8>>,
  create_address: Option<Address>,
}

/// ATHCON execution message structure.
#[derive(Debug)]
pub struct ExecutionMessage {
  kind: MessageKind,
  depth: i32,
  gas: i64,
  recipient: Address,
  sender: Address,
  input: Option<Vec<u8>>,
  method: Option<Vec<u8>>,
  value: u64,
  code: Option<Vec<u8>>,
}

/// ATHCON transaction context structure.
pub type ExecutionTxContext = ffi::athcon_tx_context;

/// ATHCON context structure. Exposes the ATHCON host functions, message data, and transaction context
/// to the executing VM.
pub struct ExecutionContext<'a> {
  host: &'a ffi::athcon_host_interface,
  context: *mut ffi::athcon_host_context,
  tx_context: ExecutionTxContext,
}

impl ExecutionResult {
  /// Manually create a result.
  pub fn new(status_code: StatusCode, gas_left: i64, output: Option<&[u8]>) -> Self {
    ExecutionResult {
      status_code,
      gas_left,
      output: output.map(|s| s.to_vec()),
      create_address: None,
    }
  }

  /// Create failure result.
  pub fn failure() -> Self {
    ExecutionResult::new(StatusCode::ATHCON_FAILURE, 0, None)
  }

  /// Create a revert result.
  pub fn revert(gas_left: i64, output: Option<&[u8]>) -> Self {
    ExecutionResult::new(StatusCode::ATHCON_REVERT, gas_left, output)
  }

  /// Create a successful result.
  pub fn success(gas_left: i64, output: Option<&[u8]>) -> Self {
    ExecutionResult::new(StatusCode::ATHCON_SUCCESS, gas_left, output)
  }

  /// Read the status code.
  pub fn status_code(&self) -> StatusCode {
    self.status_code
  }

  /// Read the amount of gas left.
  pub fn gas_left(&self) -> i64 {
    self.gas_left
  }

  /// Read the output returned.
  pub fn output(&self) -> Option<&Vec<u8>> {
    self.output.as_ref()
  }

  /// Read the address of the created account. This will likely be set when
  /// returned from a CREATE/CREATE2.
  pub fn create_address(&self) -> Option<&Address> {
    self.create_address.as_ref()
  }
}

impl ExecutionMessage {
  pub fn new(
    kind: MessageKind,
    depth: i32,
    gas: i64,
    recipient: Address,
    sender: Address,
    input: Option<&[u8]>,
    method: Option<&[u8]>,
    value: u64,
    code: Option<&[u8]>,
  ) -> Self {
    ExecutionMessage {
      kind,
      depth,
      gas,
      recipient,
      sender,
      input: input.map(|s| s.to_vec()),
      method: method.map(|s| s.to_vec()),
      value,
      code: code.map(|s| s.to_vec()),
    }
  }

  /// Read the message kind.
  pub fn kind(&self) -> MessageKind {
    self.kind
  }

  /// Read the call depth.
  pub fn depth(&self) -> i32 {
    self.depth
  }

  /// Read the gas limit supplied with the message.
  pub fn gas(&self) -> i64 {
    self.gas
  }

  /// Read the recipient address of the message.
  pub fn recipient(&self) -> &Address {
    &self.recipient
  }

  /// Read the sender address of the message.
  pub fn sender(&self) -> &Address {
    &self.sender
  }

  /// Read the optional input message.
  pub fn input(&self) -> Option<&Vec<u8>> {
    self.input.as_ref()
  }

  /// Read the optional method.
  pub fn method(&self) -> Option<&Vec<u8>> {
    self.method.as_ref()
  }

  /// Read the value of the message.
  pub fn value(&self) -> u64 {
    self.value
  }

  /// Read the optional init code.
  pub fn code(&self) -> Option<&Vec<u8>> {
    self.code.as_ref()
  }
}

impl<'a> ExecutionContext<'a> {
  pub fn new(host: &'a ffi::athcon_host_interface, context: *mut ffi::athcon_host_context) -> Self {
    let tx_context = unsafe { host.get_tx_context.unwrap()(context) };

    ExecutionContext {
      host,
      context,
      tx_context,
    }
  }

  /// Retrieve the transaction context.
  pub fn get_tx_context(&self) -> &ExecutionTxContext {
    &self.tx_context
  }

  /// Check if an account exists.
  pub fn account_exists(&self, address: &Address) -> bool {
    unsafe { self.host.account_exists.unwrap()(self.context, address as *const Address) }
  }

  /// Read from a storage key.
  pub fn get_storage(&self, address: &Address, key: &Bytes32) -> Bytes32 {
    unsafe {
      self.host.get_storage.unwrap()(
        self.context,
        address as *const Address,
        key as *const Bytes32,
      )
    }
  }

  /// Set value of a storage key.
  pub fn set_storage(
    &mut self,
    address: &Address,
    key: &Bytes32,
    value: &Bytes32,
  ) -> StorageStatus {
    unsafe {
      assert!(self.host.set_storage.is_some());
      self.host.set_storage.unwrap()(
        self.context,
        address as *const Address,
        key as *const Bytes32,
        value as *const Bytes32,
      )
    }
  }

  /// Get balance of an account.
  pub fn get_balance(&self, address: &Address) -> u64 {
    unsafe {
      assert!(self.host.get_balance.is_some());
      self.host.get_balance.unwrap()(self.context, address as *const Address)
    }
  }

  /// Call to another account.
  pub fn call(&mut self, message: &ExecutionMessage) -> ExecutionResult {
    // There is no need to make any kind of copies here, because the caller
    // won't go out of scope and ensures these pointers remain valid.
    let input = message.input();
    let (input_data, input_size) = if let Some(input) = input {
      (input.as_ptr(), input.len())
    } else {
      (std::ptr::null(), 0)
    };
    let (method_name, method_name_size) = if let Some(method) = message.method() {
      (method.as_ptr(), method.len())
    } else {
      (std::ptr::null(), 0)
    };
    let code = message.code();
    let code_size = if let Some(code) = code { code.len() } else { 0 };
    let code_data = if let Some(code) = code {
      code.as_ptr()
    } else {
      std::ptr::null()
    };
    // Cannot use a nice from trait here because that complicates memory management,
    // athcon_message doesn't have a release() method we could abstract it with.
    let message = ffi::athcon_message {
      kind: message.kind(),
      depth: message.depth(),
      gas: message.gas(),
      recipient: *message.recipient(),
      sender: *message.sender(),
      input_data,
      input_size,
      method_name,
      method_name_size,
      value: message.value,
      code: code_data,
      code_size,
    };
    unsafe {
      assert!(self.host.call.is_some());
      self.host.call.unwrap()(self.context, &message as *const ffi::athcon_message).into()
    }
  }

  /// Get block hash of an account.
  pub fn get_block_hash(&self, num: i64) -> Bytes32 {
    unsafe {
      assert!(self.host.get_block_hash.is_some());
      self.host.get_block_hash.unwrap()(self.context, num)
    }
  }

  /// Spawn a new program from a template
  pub fn spawn(&self, code: &[u8]) -> Address {
    unsafe { self.host.spawn.unwrap()(self.context, code.as_ptr(), code.len()) }
  }

  /// Deploy a new template
  /// Returns the newly-deployed template address, which is calculated as the hash of the template code
  /// The code is a pointer to the code buffer.
  pub fn deploy(&self, code: &[u8]) -> Address {
    unsafe { self.host.deploy.unwrap()(self.context, code.as_ptr(), code.len()) }
  }
}

impl From<ffi::athcon_result> for ExecutionResult {
  fn from(result: ffi::athcon_result) -> Self {
    let ret = Self {
      status_code: result.status_code,
      gas_left: result.gas_left,
      output: if result.output_data.is_null() {
        assert_eq!(result.output_size, 0);
        None
      } else if result.output_size == 0 {
        None
      } else {
        Some(unsafe { slice::from_raw_parts(result.output_data, result.output_size).to_vec() })
      },
      // Consider it is always valid.
      create_address: Some(result.create_address),
    };

    // Release allocated ffi struct.
    if result.release.is_some() {
      unsafe {
        result.release.unwrap()(&result as *const ffi::athcon_result);
      }
    }

    ret
  }
}

fn allocate_output_data(output: Option<&Vec<u8>>) -> (*const u8, usize) {
  match output {
    Some(buf) if !buf.is_empty() => {
      let buf_len = buf.len();

      // Manually allocate heap memory for the new home of the output buffer.
      let memlayout = std::alloc::Layout::from_size_align(buf_len, 1).expect("Bad layout");
      let new_buf = unsafe { std::alloc::alloc(memlayout) };
      unsafe {
        // Copy the data into the allocated buffer.
        std::ptr::copy(buf.as_ptr(), new_buf, buf_len);
      }

      (new_buf as *const u8, buf_len)
    }
    _ => (core::ptr::null(), 0),
  }
}

unsafe fn deallocate_output_data(ptr: *const u8, size: usize) {
  // be careful with dangling, aligned pointers here; they are not null but
  // not valid and cannot be deallocated!
  if !ptr.is_null() && size > 0 {
    let buf_layout = std::alloc::Layout::from_size_align(size, 1).expect("Bad layout");
    std::alloc::dealloc(ptr as *mut u8, buf_layout);
  }
}

/// Returns a pointer to a heap-allocated athcon_result.
impl From<ExecutionResult> for *const ffi::athcon_result {
  fn from(value: ExecutionResult) -> Self {
    let mut result: ffi::athcon_result = value.into();
    result.release = Some(release_heap_result);
    Box::into_raw(Box::new(result))
  }
}

/// Callback to pass across FFI, de-allocating the optional output_data.
extern "C" fn release_heap_result(result: *const ffi::athcon_result) {
  unsafe {
    let tmp = Box::from_raw(result as *mut ffi::athcon_result);
    deallocate_output_data(tmp.output_data, tmp.output_size);
  }
}

/// Returns a pointer to a stack-allocated athcon_result.
impl From<ExecutionResult> for ffi::athcon_result {
  fn from(value: ExecutionResult) -> Self {
    let (buffer, len) = allocate_output_data(value.output.as_ref());
    Self {
      status_code: value.status_code,
      gas_left: value.gas_left,
      output_data: buffer,
      output_size: len,
      release: Some(release_stack_result),
      create_address: if value.create_address.is_some() {
        value.create_address.unwrap()
      } else {
        Address { bytes: [0u8; 24] }
      },
    }
  }
}

/// Callback to pass across FFI, de-allocating the optional output_data.
extern "C" fn release_stack_result(result: *const ffi::athcon_result) {
  unsafe {
    let tmp = *result;
    deallocate_output_data(tmp.output_data, tmp.output_size);
  }
}

impl TryFrom<&ffi::athcon_message> for ExecutionMessage {
  type Error = String;

  fn try_from(message: &ffi::athcon_message) -> Result<Self, Self::Error> {
    Ok(ExecutionMessage {
      kind: message.kind,
      depth: message.depth,
      gas: message.gas,
      recipient: message.recipient,
      sender: message.sender,
      input: if message.input_data.is_null() {
        if message.input_size != 0 {
          return Err("msg.input_data is null but msg.input_size is not 0".to_string());
        }
        None
      } else if message.input_size == 0 {
        None
      } else {
        Some(unsafe { slice::from_raw_parts(message.input_data, message.input_size).to_vec() })
      },
      method: if message.method_name.is_null() {
        if message.method_name_size != 0 {
          return Err("msg.method_data is null but msg.method_size is not 0".to_string());
        }
        None
      } else if message.method_name_size == 0 {
        None
      } else {
        Some(unsafe {
          slice::from_raw_parts(message.method_name, message.method_name_size).to_vec()
        })
      },
      value: message.value,
      code: if message.code.is_null() {
        if message.code_size != 0 {
          return Err("msg.code is null but msg.code_size is not 0".to_string());
        }
        None
      } else if message.code_size == 0 {
        None
      } else {
        Some(unsafe { slice::from_raw_parts(message.code, message.code_size).to_vec() })
      },
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn result_new() {
    let r = ExecutionResult::new(StatusCode::ATHCON_FAILURE, 420, None);

    assert_eq!(r.status_code(), StatusCode::ATHCON_FAILURE);
    assert_eq!(r.gas_left(), 420);
    assert!(r.output().is_none());
    assert!(r.create_address().is_none());
  }

  // Test-specific helper to dispose of execution results in unit tests
  extern "C" fn test_result_dispose(result: *const ffi::athcon_result) {
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

  #[test]
  fn result_from_ffi() {
    let f = ffi::athcon_result {
      status_code: StatusCode::ATHCON_SUCCESS,
      gas_left: 1337,
      output_data: Box::into_raw(Box::new([0xde, 0xad, 0xbe, 0xef])) as *const u8,
      output_size: 4,
      release: Some(test_result_dispose),
      create_address: Address { bytes: [0u8; 24] },
    };

    let r: ExecutionResult = f.into();

    assert_eq!(r.status_code(), StatusCode::ATHCON_SUCCESS);
    assert_eq!(r.gas_left(), 1337);
    assert!(r.output().is_some());
    assert_eq!(r.output().unwrap().len(), 4);
    assert!(r.create_address().is_some());
  }

  #[test]
  fn result_into_heap_ffi() {
    let r = ExecutionResult::new(
      StatusCode::ATHCON_FAILURE,
      420,
      Some(&[0xc0, 0xff, 0xee, 0x71, 0x75]),
    );

    let f: *const ffi::athcon_result = r.into();
    assert!(!f.is_null());
    unsafe {
      assert_eq!((*f).status_code, StatusCode::ATHCON_FAILURE);
      assert_eq!((*f).gas_left, 420);
      assert!(!(*f).output_data.is_null());
      assert_eq!((*f).output_size, 5);
      assert_eq!(
        std::slice::from_raw_parts((*f).output_data, 5) as &[u8],
        &[0xc0, 0xff, 0xee, 0x71, 0x75]
      );
      assert_eq!((*f).create_address.bytes, [0u8; 24]);
      if (*f).release.is_some() {
        (*f).release.unwrap()(f);
      }
    }
  }

  #[test]
  fn result_into_heap_ffi_empty_data() {
    let r = ExecutionResult::new(StatusCode::ATHCON_FAILURE, 420, None);

    let f: *const ffi::athcon_result = r.into();
    assert!(!f.is_null());
    unsafe {
      assert_eq!((*f).status_code, StatusCode::ATHCON_FAILURE);
      assert_eq!((*f).gas_left, 420);
      assert!((*f).output_data.is_null(),);
      assert_eq!((*f).output_size, 0);
      assert_eq!((*f).create_address.bytes, [0u8; 24]);
      if (*f).release.is_some() {
        (*f).release.unwrap()(f);
      }
    }
  }

  #[test]
  fn result_into_stack_ffi() {
    let r = ExecutionResult::new(
      StatusCode::ATHCON_FAILURE,
      420,
      Some(&[0xc0, 0xff, 0xee, 0x71, 0x75]),
    );

    let f: ffi::athcon_result = r.into();
    unsafe {
      assert_eq!(f.status_code, StatusCode::ATHCON_FAILURE);
      assert_eq!(f.gas_left, 420);
      assert!(!f.output_data.is_null());
      assert_eq!(f.output_size, 5);
      assert_eq!(
        std::slice::from_raw_parts(f.output_data, 5) as &[u8],
        &[0xc0, 0xff, 0xee, 0x71, 0x75]
      );
      assert_eq!(f.create_address.bytes, [0u8; 24]);
      if f.release.is_some() {
        f.release.unwrap()(&f);
      }
    }
  }

  #[test]
  fn result_into_stack_ffi_empty_data() {
    let r = ExecutionResult::new(StatusCode::ATHCON_FAILURE, 420, None);

    let f: ffi::athcon_result = r.into();
    unsafe {
      assert_eq!(f.status_code, StatusCode::ATHCON_FAILURE);
      assert_eq!(f.gas_left, 420);
      assert!(f.output_data.is_null());
      assert_eq!(f.output_size, 0);
      assert_eq!(f.create_address.bytes, [0u8; 24]);
      if f.release.is_some() {
        f.release.unwrap()(&f);
      }
    }
  }

  #[test]
  fn message_new_with_input() {
    let input = vec![0xc0, 0xff, 0xee];
    let recipient = Address { bytes: [32u8; 24] };
    let sender = Address { bytes: [128u8; 24] };
    let value = 77;

    let ret = ExecutionMessage::new(
      MessageKind::ATHCON_CALL,
      66,
      4466,
      recipient,
      sender,
      Some(&input),
      None,
      value,
      None,
    );

    assert_eq!(ret.kind(), MessageKind::ATHCON_CALL);
    assert_eq!(ret.depth(), 66);
    assert_eq!(ret.gas(), 4466);
    assert_eq!(*ret.recipient(), recipient);
    assert_eq!(*ret.sender(), sender);
    assert!(ret.input().is_some());
    assert_eq!(*ret.input().unwrap(), input);
    assert_eq!(ret.value, value);
  }

  #[test]
  fn message_new_with_code() {
    let recipient = Address { bytes: [32u8; 24] };
    let sender = Address { bytes: [128u8; 24] };
    let value = 0;
    let code = vec![0x5f, 0x5f, 0xfd];

    let ret = ExecutionMessage::new(
      MessageKind::ATHCON_CALL,
      66,
      4466,
      recipient,
      sender,
      None,
      None,
      value,
      Some(&code),
    );

    assert_eq!(ret.kind(), MessageKind::ATHCON_CALL);
    assert_eq!(ret.depth(), 66);
    assert_eq!(ret.gas(), 4466);
    assert_eq!(*ret.recipient(), recipient);
    assert_eq!(*ret.sender(), sender);
    assert_eq!(ret.value, value);
    assert!(ret.code().is_some());
    assert_eq!(*ret.code().unwrap(), code);
  }

  fn valid_athcon_message() -> ffi::athcon_message {
    let recipient = Address { bytes: [32u8; 24] };
    let sender = Address { bytes: [128u8; 24] };
    let value = 0;

    ffi::athcon_message {
      kind: MessageKind::ATHCON_CALL,
      depth: 66,
      gas: 4466,
      recipient,
      sender,
      input_data: std::ptr::null(),
      input_size: 0,
      method_name: std::ptr::null(),
      method_name_size: 0,
      value,
      code: std::ptr::null(),
      code_size: 0,
    }
  }

  #[test]
  fn message_from_ffi() {
    let msg = &valid_athcon_message();
    let ret: ExecutionMessage = msg.try_into().unwrap();

    assert_eq!(ret.kind(), msg.kind);
    assert_eq!(ret.depth(), msg.depth);
    assert_eq!(ret.gas(), msg.gas);
    assert_eq!(*ret.recipient(), msg.recipient);
    assert_eq!(*ret.sender(), msg.sender);
    assert!(ret.input().is_none());
    assert_eq!(ret.value, msg.value);
    assert!(ret.code().is_none());
  }

  #[test]
  fn message_from_ffi_with_input() {
    let input = vec![0xc0, 0xff, 0xee];

    let msg = &ffi::athcon_message {
      input_data: input.as_ptr(),
      input_size: input.len(),
      ..valid_athcon_message()
    };

    let ret: ExecutionMessage = msg.try_into().unwrap();

    assert_eq!(ret.kind(), msg.kind);
    assert_eq!(ret.depth(), msg.depth);
    assert_eq!(ret.gas(), msg.gas);
    assert_eq!(*ret.recipient(), msg.recipient);
    assert_eq!(*ret.sender(), msg.sender);
    assert!(ret.input().is_some());
    assert_eq!(*ret.input().unwrap(), input);
    assert_eq!(ret.value, msg.value);
    assert!(ret.code().is_none());
  }

  #[test]
  fn message_from_ffi_with_code() {
    let code = vec![0x5f, 0x5f, 0xfd];

    let msg = &ffi::athcon_message {
      code: code.as_ptr(),
      code_size: code.len(),
      ..valid_athcon_message()
    };

    let ret: ExecutionMessage = msg.try_into().unwrap();

    assert_eq!(ret.kind(), msg.kind);
    assert_eq!(ret.depth(), msg.depth);
    assert_eq!(ret.gas(), msg.gas);
    assert_eq!(*ret.recipient(), msg.recipient);
    assert_eq!(*ret.sender(), msg.sender);
    assert!(ret.input().is_none());
    assert_eq!(ret.value, msg.value);
    assert!(ret.code().is_some());
    assert_eq!(*ret.code().unwrap(), code);
  }

  #[test]
  fn message_from_ffi_code_size_must_be_0_when_no_code() {
    let msg = &ffi::athcon_message {
      code: std::ptr::null(),
      code_size: 10,
      ..valid_athcon_message()
    };
    let ret: Result<ExecutionMessage, _> = msg.try_into();
    assert!(ret.is_err());
  }

  #[test]
  fn message_from_ffi_input_size_must_be_0_when_no_input() {
    let msg = &ffi::athcon_message {
      input_data: std::ptr::null(),
      input_size: 10,
      ..valid_athcon_message()
    };
    let ret: Result<ExecutionMessage, _> = msg.try_into();
    assert!(ret.is_err());
  }

  unsafe extern "C" fn get_dummy_tx_context(
    _context: *mut ffi::athcon_host_context,
  ) -> ffi::athcon_tx_context {
    ffi::athcon_tx_context {
      tx_gas_price: 0,
      tx_origin: Address { bytes: [0u8; 24] },
      block_height: 42,
      block_timestamp: 235117,
      block_gas_limit: 105023,
      chain_id: Uint256::default(),
    }
  }

  unsafe extern "C" fn execute_call(
    _context: *mut ffi::athcon_host_context,
    msg: *const ffi::athcon_message,
  ) -> ffi::athcon_result {
    let msg = &*msg;
    let success = if msg.input_size != 0 && msg.input_data.is_null() {
      false
    } else {
      msg.input_size != 0 || msg.input_data.is_null()
    };

    ffi::athcon_result {
      status_code: if success {
        StatusCode::ATHCON_SUCCESS
      } else {
        StatusCode::ATHCON_INTERNAL_ERROR
      },
      gas_left: 2,
      // NOTE: we are passing the input pointer here, but for testing the lifetime is ok
      output_data: msg.input_data,
      output_size: msg.input_size,
      release: None,
      create_address: Address::default(),
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
      spawn: None,
      deploy: None,
    }
  }

  #[test]
  fn execution_context() {
    let host_context = std::ptr::null_mut();
    let host_interface = get_dummy_host_interface();
    let exe_context = ExecutionContext::new(&host_interface, host_context);
    let a = exe_context.get_tx_context();

    let b = unsafe { get_dummy_tx_context(host_context) };

    assert_eq!(a.block_gas_limit, b.block_gas_limit);
    assert_eq!(a.block_timestamp, b.block_timestamp);
    assert_eq!(a.block_height, b.block_height);
  }

  #[test]
  fn test_call_empty_data() {
    // This address is useless. Just a dummy parameter for the interface function.
    let test_addr = Address::default();
    let host = get_dummy_host_interface();
    let host_context = std::ptr::null_mut();
    let mut exe_context = ExecutionContext::new(&host, host_context);

    let message = ExecutionMessage::new(
      MessageKind::ATHCON_CALL,
      0,
      6566,
      test_addr,
      test_addr,
      None,
      None,
      0,
      None,
    );

    let b = exe_context.call(&message);

    assert_eq!(b.status_code(), StatusCode::ATHCON_SUCCESS);
    assert_eq!(b.gas_left(), 2);
    assert!(b.output().is_none());
    assert!(b.create_address().is_some());
    assert_eq!(b.create_address().unwrap(), &Address::default());
  }

  #[test]
  fn test_call_with_data() {
    // This address is useless. Just a dummy parameter for the interface function.
    let test_addr = Address::default();
    let host = get_dummy_host_interface();
    let host_context = std::ptr::null_mut();
    let mut exe_context = ExecutionContext::new(&host, host_context);

    let data = vec![0xc0, 0xff, 0xfe];

    let message = ExecutionMessage::new(
      MessageKind::ATHCON_CALL,
      0,
      6566,
      test_addr,
      test_addr,
      Some(&data),
      None,
      0,
      None,
    );

    let b = exe_context.call(&message);

    assert_eq!(b.status_code(), StatusCode::ATHCON_SUCCESS);
    assert_eq!(b.gas_left(), 2);
    assert!(b.output().is_some());
    assert_eq!(b.output().unwrap(), &data);
    assert!(b.create_address().is_some());
    assert_eq!(b.create_address().unwrap(), &Address::default());
  }
}
