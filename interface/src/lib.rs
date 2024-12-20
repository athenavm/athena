//! # Athena Interface
//!
//! A library with no external dependencies that includes core types and traits.

mod context;
pub mod payload;
pub use context::*;

use blake3::Hasher;
pub use parity_scale_codec::{Decode, Encode};
use payload::ExecutionPayload;

use std::{collections::BTreeMap, convert::TryFrom, error::Error, fmt};

pub const ADDRESS_LENGTH: usize = 24;
pub const BYTES32_LENGTH: usize = 32;
pub const METHOD_SELECTOR_LENGTH: usize = 4;
pub type Balance = u64;
pub type Bytes32 = [u8; BYTES32_LENGTH];
pub type Bytes = [u8];

#[derive(Clone, Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub struct MethodSelector([u8; METHOD_SELECTOR_LENGTH]);

impl From<&str> for MethodSelector {
  fn from(value: &str) -> Self {
    let hash = blake3::hash(value.as_bytes());
    MethodSelector(
      hash.as_bytes()[..METHOD_SELECTOR_LENGTH]
        .try_into()
        .unwrap(),
    )
  }
}

impl std::fmt::Display for MethodSelector {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", hex::encode(self.0))
  }
}

#[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Decode, Encode)]
pub struct Address([u8; ADDRESS_LENGTH]);

impl From<&Address> for [u8; ADDRESS_LENGTH] {
  fn from(value: &Address) -> Self {
    value.0
  }
}

impl From<Address> for [u8; ADDRESS_LENGTH] {
  fn from(value: Address) -> Self {
    value.0
  }
}

impl From<[u8; ADDRESS_LENGTH]> for Address {
  fn from(value: [u8; ADDRESS_LENGTH]) -> Self {
    Address(value)
  }
}

impl AsRef<[u8; ADDRESS_LENGTH]> for Address {
  fn as_ref(&self) -> &[u8; ADDRESS_LENGTH] {
    &self.0
  }
}

impl std::fmt::Display for Address {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", hex::encode(self.0))
  }
}

pub struct Bytes32Wrapper(Bytes32);

impl Bytes32Wrapper {
  pub fn new(bytes: Bytes32) -> Self {
    Bytes32Wrapper(bytes)
  }
}

impl From<Vec<u32>> for Bytes32Wrapper {
  fn from(value: Vec<u32>) -> Self {
    assert!(value.len() == 8, "Invalid address length");
    let mut bytes = [0u8; 32];
    for (i, word) in value.iter().enumerate() {
      let value_bytes = word.to_le_bytes();
      bytes[i * 4..(i + 1) * 4].copy_from_slice(&value_bytes);
    }
    Bytes32Wrapper(bytes)
  }
}

impl From<Bytes32Wrapper> for Vec<u32> {
  fn from(value: Bytes32Wrapper) -> Vec<u32> {
    bytemuck::cast::<[u8; 32], [u32; 8]>(value.0).to_vec()
  }
}

impl From<Bytes32Wrapper> for Bytes32 {
  fn from(value: Bytes32Wrapper) -> Bytes32 {
    value.0
  }
}

// This is based on EIP-2200.
// See https://evmc.ethereum.org/storagestatus.html.
#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum StorageStatus {
  StorageAssigned,
  StorageAdded,
  StorageDeleted,
  StorageModified,
  StorageDeletedAdded,
  StorageModifiedDeleted,
  StorageDeletedRestored,
  StorageAddedDeleted,
  StorageModifiedRestored,
}

impl TryFrom<u32> for StorageStatus {
  type Error = &'static str;
  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(StorageStatus::StorageAssigned),
      1 => Ok(StorageStatus::StorageAdded),
      2 => Ok(StorageStatus::StorageDeleted),
      3 => Ok(StorageStatus::StorageModified),
      4 => Ok(StorageStatus::StorageDeletedAdded),
      5 => Ok(StorageStatus::StorageModifiedDeleted),
      6 => Ok(StorageStatus::StorageDeletedRestored),
      7 => Ok(StorageStatus::StorageAddedDeleted),
      8 => Ok(StorageStatus::StorageModifiedRestored),
      _ => Err("Invalid storage status"),
    }
  }
}

impl fmt::Display for StorageStatus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      StorageStatus::StorageAssigned => write!(f, "The storage item is assigned without affecting the cost structure."),
      StorageStatus::StorageAdded => write!(f, "A new storage item is added by changing the current clean zero to a nonzero value."),
      StorageStatus::StorageDeleted => write!(f, "A storage item is deleted by changing the current clean nonzero to the zero value."),
      StorageStatus::StorageModified => write!(f, "A storage item is modified by changing the current clean nonzero to another nonzero value."),
      StorageStatus::StorageDeletedAdded => write!(f, "A storage item is added by changing the current dirty zero to a nonzero value other than the original."),
      StorageStatus::StorageModifiedDeleted => write!(f, "A storage item is deleted by changing the current dirty nonzero to the zero value and the original value is not zero."),
      StorageStatus::StorageDeletedRestored => write!(f, "A storage item is added by changing the current dirty zero to the original value."),
      StorageStatus::StorageAddedDeleted => write!(f, "A storage item is deleted by changing the current dirty nonzero to the original zero value."),
      StorageStatus::StorageModifiedRestored => write!(f, "A storage item is modified by changing the current dirty nonzero to the original nonzero value other than the current."),
    }
  }
}

#[derive(Copy, Clone)]
pub struct TransactionContext {
  pub gas_price: u64,
  pub origin: Address,
  pub block_height: i64,
  pub block_timestamp: i64,
  pub block_gas_limit: i64,
  pub chain_id: Bytes32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageKind {
  Call,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AthenaMessage {
  pub kind: MessageKind,
  pub depth: u32,
  pub gas: u32,
  pub recipient: Address,
  pub sender: Address,
  pub input_data: Option<Vec<u8>>,
  pub value: Balance,
  // code is currently unused, and it seems redundant.
  // it's not in the yellow paper.
  // TODO: remove me?
  pub code: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
impl AthenaMessage {
  pub fn new(
    kind: MessageKind,
    depth: u32,
    gas: u32,
    recipient: Address,
    sender: Address,
    input_data: Option<Vec<u8>>,
    value: Balance,
    code: Vec<u8>,
  ) -> Self {
    AthenaMessage {
      kind,
      depth,
      gas,
      recipient,
      sender,
      input_data,
      value,
      code,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum StatusCode {
  Success,
  Failure,
  Revert,
  OutOfGas,
  InvalidInstruction,
  UndefinedInstruction,
  StackOverflow,
  StackUnderflow,
  BadJumpDestination,
  InvalidMemoryAccess,
  CallDepthExceeded,
  StaticModeViolation,
  PrecompileFailure,
  ContractValidationFailure,
  ArgumentOutOfRange,
  UnreachableInstruction,
  Trap,
  InsufficientBalance,
  InternalError,
  Rejected,
  OutOfMemory,
  InsufficientInput,
  InvalidSyscallArgument,
}

impl TryFrom<u32> for StatusCode {
  type Error = &'static str;

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      x if x == StatusCode::Success as u32 => Ok(StatusCode::Success),
      x if x == StatusCode::Failure as u32 => Ok(StatusCode::Failure),
      x if x == StatusCode::Revert as u32 => Ok(StatusCode::Revert),
      x if x == StatusCode::OutOfGas as u32 => Ok(StatusCode::OutOfGas),
      x if x == StatusCode::InvalidInstruction as u32 => Ok(StatusCode::InvalidInstruction),
      x if x == StatusCode::UndefinedInstruction as u32 => Ok(StatusCode::UndefinedInstruction),
      x if x == StatusCode::StackOverflow as u32 => Ok(StatusCode::StackOverflow),
      x if x == StatusCode::StackUnderflow as u32 => Ok(StatusCode::StackUnderflow),
      x if x == StatusCode::BadJumpDestination as u32 => Ok(StatusCode::BadJumpDestination),
      x if x == StatusCode::InvalidMemoryAccess as u32 => Ok(StatusCode::InvalidMemoryAccess),
      x if x == StatusCode::CallDepthExceeded as u32 => Ok(StatusCode::CallDepthExceeded),
      x if x == StatusCode::StaticModeViolation as u32 => Ok(StatusCode::StaticModeViolation),
      x if x == StatusCode::PrecompileFailure as u32 => Ok(StatusCode::PrecompileFailure),
      x if x == StatusCode::ContractValidationFailure as u32 => {
        Ok(StatusCode::ContractValidationFailure)
      }
      x if x == StatusCode::ArgumentOutOfRange as u32 => Ok(StatusCode::ArgumentOutOfRange),
      x if x == StatusCode::UnreachableInstruction as u32 => Ok(StatusCode::UnreachableInstruction),
      x if x == StatusCode::Trap as u32 => Ok(StatusCode::Trap),
      x if x == StatusCode::InsufficientBalance as u32 => Ok(StatusCode::InsufficientBalance),
      x if x == StatusCode::InternalError as u32 => Ok(StatusCode::InternalError),
      x if x == StatusCode::Rejected as u32 => Ok(StatusCode::Rejected),
      x if x == StatusCode::OutOfMemory as u32 => Ok(StatusCode::OutOfMemory),
      _ => Err("Invalid StatusCode"),
    }
  }
}

impl fmt::Display for StatusCode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      StatusCode::Success => write!(f, "Execution finished with success."),
      StatusCode::Failure => write!(f, "Generic execution failure."),
      StatusCode::Revert => write!(f, "Execution terminated with REVERT opcode."),
      StatusCode::OutOfGas => write!(f, "The execution has run out of gas."),
      StatusCode::InvalidInstruction => write!(f, "The execution has encountered an invalid instruction."),
      StatusCode::UndefinedInstruction => write!(f, "An undefined instruction has been encountered."),
      StatusCode::StackOverflow => write!(f, "A stack overflow has been encountered."),
      StatusCode::StackUnderflow => write!(f, "A stack underflow has been encountered."),
      StatusCode::BadJumpDestination => write!(f, "A bad jump destination has been encountered."),
      StatusCode::InvalidMemoryAccess => write!(f, "Tried to read outside memory bounds."),
      StatusCode::CallDepthExceeded => write!(f, "Call depth has exceeded the limit."),
      StatusCode::StaticModeViolation => write!(f, "Static mode violation."),
      StatusCode::PrecompileFailure => write!(f, "A call to a precompiled or system contract has ended with a failure."),
      StatusCode::ContractValidationFailure => write!(f, "Contract validation has failed."),
      StatusCode::ArgumentOutOfRange => write!(f, "An argument to a state accessing method has a value outside of the accepted range."),
      StatusCode::UnreachableInstruction => write!(f, "An unreachable instruction has been encountered."),
      StatusCode::Trap => write!(f, "A trap has been encountered."),
      StatusCode::InsufficientBalance => write!(f, "The caller does not have enough funds for value transfer."),
      StatusCode::InternalError => write!(f, "Athena implementation generic internal error."),
      StatusCode::Rejected => write!(f, "The execution of the given code and/or message has been rejected by the Athena implementation."),
      StatusCode::OutOfMemory => write!(f, "The VM failed to allocate the amount of memory needed for execution."),
      StatusCode::InsufficientInput => write!(f, "Tried to read more input than was available."),
      StatusCode::InvalidSyscallArgument => write!(f, "Invalid syscall arguments."),
    }
  }
}

#[derive(Debug)]
pub struct ExecutionResult {
  pub status_code: StatusCode,
  pub gas_left: u32,
  pub output: Option<Vec<u8>>,
}

impl ExecutionResult {
  pub fn new(status_code: StatusCode, gas_left: u32, output: Option<Vec<u8>>) -> Self {
    ExecutionResult {
      status_code,
      gas_left,
      output,
    }
  }

  pub fn failed(gas_left: u32) -> Self {
    ExecutionResult::new(StatusCode::Failure, gas_left, None)
  }
}

#[mockall::automock]
pub trait HostInterface {
  fn get_storage(&self, addr: &Address, key: &Bytes32) -> Bytes32;
  fn set_storage(&mut self, addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus;
  fn get_balance(&self, addr: &Address) -> Balance;
  fn call(&mut self, msg: AthenaMessage) -> ExecutionResult;
  fn spawn(&mut self, blob: Vec<u8>) -> Address;
  fn deploy(&mut self, code: Vec<u8>) -> Result<Address, Box<dyn Error>>;
}

// Calculates a spawned program address on the basis of the template address, state blob,
// spawning principal and nonce.
pub fn calculate_address(
  template: &Address,
  blob: &[u8],
  principal: &Address,
  nonce: u64,
) -> Address {
  // calculate address by hashing the template, blob, principal, and nonce
  let mut hasher = Hasher::new();
  hasher.update(&template.0);
  hasher.update(blob);
  hasher.update(&principal.0);
  hasher.update(&nonce.to_le_bytes());
  Address(hasher.finalize().as_bytes()[..24].try_into().unwrap())
}
// Stores some of the context that a running host would store to keep
// track of what's going on in the VM execution
// static context is set from the transaction and doesn't change until
// the execution stack is done.
pub struct HostStaticContext {
  // the ultimate initiator of the current execution stack. also the
  // account that pays gas for the execution stack.
  principal: Address,

  // the principal's nonce from the tx
  nonce: u64,

  // the destination of the transaction. note that, while this is the
  // program that was initiated, it likely made additional calls.
  // this is generally the caller's wallet, and is generally the same
  // as the principal.
  _destination: Address,
  // in the future we'll probably need things here like block height,
  // block hash, etc.
}

impl HostStaticContext {
  pub fn new(principal: Address, nonce: u64, destination: Address) -> HostStaticContext {
    HostStaticContext {
      principal,
      nonce,
      _destination: destination,
    }
  }
}

// this context is relevant only for the current execution frame
pub struct HostDynamicContext {
  // the initiator and recipient programs of the current message/call frame
  template: Address,
  _callee: Address,
}

impl HostDynamicContext {
  pub fn new(template: Address, callee: Address) -> HostDynamicContext {
    HostDynamicContext {
      template,
      _callee: callee,
    }
  }
}

#[derive(Debug, Clone)]
pub struct SpawnResult {
  pub address: Address,
  pub blob: Vec<u8>,
  pub template: Address,
  pub principal: Address,
  pub nonce: u64,
}

// a very simple mock host implementation for testing
// also useful for filling in the missing generic type
// when running the VM in standalone mode, without a bound host interface
#[derive(Default)]
pub struct MockHost<'a> {
  // VM instance
  vm: Option<&'a dyn VmInterface<MockHost<'a>>>,

  // stores state keyed by address and key
  storage: BTreeMap<(Address, Bytes32), Bytes32>,

  // stores balance keyed by address
  balance: BTreeMap<Address, Balance>,

  // stores contract code
  templates: BTreeMap<Address, Vec<u8>>,

  // stores program instances
  programs: BTreeMap<Address, Vec<u8>>,

  // context information
  static_context: Option<HostStaticContext>,
  dynamic_context: Option<HostDynamicContext>,
}

impl<'a> MockHost<'a> {
  pub fn new() -> Self {
    MockHost::default()
  }

  pub fn new_with_vm(vm: &'a dyn VmInterface<MockHost<'a>>) -> Self {
    MockHost {
      vm: Some(vm),
      ..MockHost::default()
    }
  }

  pub fn new_with_context(
    static_context: HostStaticContext,
    dynamic_context: HostDynamicContext,
  ) -> Self {
    MockHost {
      dynamic_context: Some(dynamic_context),
      static_context: Some(static_context),
      ..MockHost::default()
    }
  }

  /// Set balance of given address.
  /// The previous balance is discarded.
  pub fn set_balance(&mut self, address: &Address, balance: Balance) {
    self.balance.insert(*address, balance);
  }

  pub fn spawn_program(
    &mut self,
    template: &Address,
    blob: Vec<u8>,
    principal: &Address,
    nonce: u64,
  ) -> Address {
    let address = calculate_address(template, &blob, principal, nonce);
    tracing::info!("spawning program {blob:?} at address {address:?} for principal {principal:?} with template {template:?}");

    self.programs.insert(address, blob.clone());
    address
  }

  pub fn get_program(&self, address: &Address) -> Option<&Vec<u8>> {
    self.programs.get(address)
  }

  pub fn template(&self, address: &Address) -> Option<&Vec<u8>> {
    self.templates.get(address)
  }

  pub fn deploy_code(&mut self, address: Address, code: Vec<u8>) {
    self.templates.insert(address, code);
  }

  pub fn transfer_balance(&mut self, from: &Address, to: &Address, value: u64) -> StatusCode {
    let balance_from = self.get_balance(from);
    let balance_to = self.get_balance(to);
    if value > balance_from {
      return StatusCode::InsufficientBalance;
    }
    match balance_to.checked_add(value) {
      Some(new_balance) => {
        self.balance.insert(*from, balance_from - value);
        self.balance.insert(*to, new_balance);
        StatusCode::Success
      }
      None => StatusCode::InternalError,
    }
  }
}

pub const ADDRESS_ALICE: Address = Address([1u8; ADDRESS_LENGTH]);
pub const ADDRESS_CHARLIE: Address = Address([3u8; ADDRESS_LENGTH]);
// "sentinel value" useful for testing: 0xc0ffee
// also useful as a morning wake up!
pub const STORAGE_KEY: Bytes32 = [
  0xc0, 0xff, 0xee, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
pub const STORAGE_VALUE: Bytes32 = [
  0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

impl HostInterface for MockHost<'_> {
  fn get_storage(&self, addr: &Address, key: &Bytes32) -> Bytes32 {
    self
      .storage
      .get(&(*addr, *key))
      .copied()
      .unwrap_or(Bytes32::default())
  }

  fn set_storage(&mut self, addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
    // this is a very simplistic implementation and does NOT handle all possible cases correctly
    match self.storage.insert((*addr, *key), *value) {
      None => StorageStatus::StorageAdded,
      Some(_) => StorageStatus::StorageModified,
    }
  }

  fn get_balance(&self, addr: &Address) -> u64 {
    self.balance.get(addr).copied().unwrap_or(0)
  }

  #[tracing::instrument(skip(self, msg), fields(id = self as *const Self as usize, depth = msg.depth))]
  fn call(&mut self, msg: AthenaMessage) -> ExecutionResult {
    tracing::info!(msg = ?msg);

    // don't go too deep!
    if msg.depth > 10 {
      return ExecutionResult::new(StatusCode::CallDepthExceeded, 0, None);
    }

    // take snapshots of the state in case we need to roll back
    // this is relatively expensive and we'd want to do something more sophisticated in production
    // (journaling? CoW?) but it's fine for testing.
    tracing::debug!(
      "before backup storage item is {:?}",
      self.get_storage(&ADDRESS_ALICE, &STORAGE_KEY)
    );
    let backup_storage = self.storage.clone();
    let backup_balance = self.balance.clone();
    let backup_programs = self.templates.clone();

    // transfer balance
    // note: the host should have already subtracted an amount from the sender
    // equal to the maximum amount of gas that could be paid, so this should
    // not allow an out of gas error.
    match self.transfer_balance(&msg.sender, &msg.recipient, msg.value) {
      StatusCode::Success => {}
      status => {
        return ExecutionResult::new(status, 0, None);
      }
    }

    // save message for context
    let old_dynamic_context = self.dynamic_context.replace(HostDynamicContext {
      template: msg.sender,
      _callee: msg.recipient,
    });

    // check programs list first
    let res = if let Some(code) = self.templates.get(&msg.recipient).cloned() {
      // create an owned copy of VM before taking the host from self
      let vm = self.vm;

      // The optional msg.input_data must be enriched with optional account state
      // and then passed to the VM.
      let msg = match msg.input_data {
        Some(data) => {
          // TODO: figure out when to provide a state here
          let state = vec![];
          AthenaMessage {
            input_data: Some(ExecutionPayload::encode_with_encoded_payload(state, data)),
            ..msg
          }
        }
        None => msg,
      };

      vm.expect("missing VM instance")
        .execute(self, AthenaRevision::AthenaFrontier, msg, &code)
    } else {
      // otherwise, pass a call to Charlie, fail all other calls
      let status_code = if msg.recipient == ADDRESS_CHARLIE {
        // calling charlie works
        StatusCode::Success
      } else {
        // no one else picks up the phone
        StatusCode::Failure
      };

      let gas_left = msg.gas.checked_sub(1).expect("gas underflow");
      ExecutionResult::new(status_code, gas_left, None)
    };

    self.dynamic_context = old_dynamic_context;

    tracing::debug!(
      "finished with storage item {:?}",
      self.get_storage(&ADDRESS_ALICE, &STORAGE_KEY)
    );
    if res.status_code != StatusCode::Success {
      // rollback state
      self.storage = backup_storage;
      self.balance = backup_balance;
      self.templates = backup_programs;
      tracing::debug!(
        "after restore storage item is {:?}",
        self.get_storage(&ADDRESS_ALICE, &STORAGE_KEY)
      );
    }
    res
  }

  fn spawn(&mut self, blob: Vec<u8>) -> Address {
    // TODO: double-check these semantics and how Spacemesh principal account semantics map to this

    // Extract the necessary values before calling spawn_program
    let template = self
      .dynamic_context
      .as_ref()
      .expect("missing dynamic host context")
      .template;

    let static_context = self
      .static_context
      .as_ref()
      .expect("missing static host context");

    // Now call spawn_program with the extracted values
    self.spawn_program(
      &template,
      blob,
      &static_context.principal.clone(),
      static_context.nonce,
    )
  }

  fn deploy(&mut self, code: Vec<u8>) -> Result<Address, Box<dyn Error>> {
    // template_address := HASH(template_code)
    let hash = blake3::hash(&code);
    let hash_bytes = hash.as_bytes().as_slice();
    let address = Address(hash_bytes[..ADDRESS_LENGTH].try_into().unwrap());

    if self.templates.contains_key(&address) {
      return Err("template already exists".into());
    }
    self.deploy_code(address, code);
    Ok(address)
  }
}

// currently unused
#[derive(Debug, Clone, Copy)]
pub enum AthenaCapability {}

// currently unused
#[derive(Debug, Clone)]
pub enum AthenaOption {}

#[derive(Debug)]
pub enum SetOptionError {
  InvalidKey,
  InvalidValue,
}

#[derive(Debug)]
pub enum AthenaRevision {
  AthenaFrontier,
}

pub trait VmInterface<T: HostInterface> {
  fn get_capabilities(&self) -> Vec<AthenaCapability>;
  fn set_option(&self, option: AthenaOption, value: &str) -> Result<(), SetOptionError>;
  fn execute(
    &self,
    host: &mut T,
    rev: AthenaRevision,
    msg: AthenaMessage,
    code: &[u8],
  ) -> ExecutionResult;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_storage() {
    let mut host = MockHost::new();
    let address = Address([8; 24]);
    let key = [1; 32];
    let value = [2; 32];
    assert_eq!(
      host.set_storage(&address, &key, &value),
      StorageStatus::StorageAdded
    );
    let retrieved_value = host.get_storage(&address, &key);
    assert_eq!(retrieved_value, value);
  }

  #[test]
  fn test_get_balance() {
    let host = MockHost::new();
    let address = Address([8; 24]);
    let balance = host.get_balance(&address);
    assert_eq!(balance, 0);
  }

  #[test]
  fn test_transfer_balance() {
    let mut host = MockHost::new();
    host.set_balance(&ADDRESS_ALICE, 10000);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 0);
    assert_eq!(host.get_balance(&ADDRESS_ALICE), 10000);
    assert_eq!(
      host.transfer_balance(&ADDRESS_ALICE, &ADDRESS_CHARLIE, 1000),
      StatusCode::Success
    );
    assert_eq!(host.get_balance(&ADDRESS_ALICE), 9000);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 1000);
    assert_eq!(
      host.transfer_balance(&ADDRESS_CHARLIE, &ADDRESS_ALICE, 1001),
      StatusCode::InsufficientBalance
    );
    assert_eq!(
      host.transfer_balance(&ADDRESS_CHARLIE, &ADDRESS_ALICE, 1000),
      StatusCode::Success
    );
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 0);

    // test overflow
    host.balance.insert(ADDRESS_CHARLIE, u64::MAX);
    assert_eq!(
      host.transfer_balance(&ADDRESS_ALICE, &ADDRESS_CHARLIE, 1),
      StatusCode::InternalError
    );
  }

  #[test]
  fn test_send_coins() {
    let mut host = MockHost::new();

    // send zero balance
    host.set_balance(&ADDRESS_ALICE, 10000);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 0);
    let msg = AthenaMessage::new(
      MessageKind::Call,
      0,
      1000,
      ADDRESS_CHARLIE,
      ADDRESS_ALICE,
      None,
      0,
      vec![],
    );
    let res = host.call(msg);
    assert_eq!(res.status_code, StatusCode::Success);
    assert_eq!(host.get_balance(&ADDRESS_ALICE), 10000);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 0);

    // send some balance
    let msg = AthenaMessage::new(
      MessageKind::Call,
      0,
      1000,
      ADDRESS_CHARLIE,
      ADDRESS_ALICE,
      None,
      100,
      vec![],
    );
    let res = host.call(msg);
    assert_eq!(res.status_code, StatusCode::Success);
    assert_eq!(host.get_balance(&ADDRESS_ALICE), 10000 - 100);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 100);

    // try to send more than the sender has
    let msg = AthenaMessage::new(
      MessageKind::Call,
      0,
      1000,
      ADDRESS_CHARLIE,
      ADDRESS_ALICE,
      None,
      10000,
      vec![],
    );
    let res = host.call(msg);
    assert_eq!(res.status_code, StatusCode::InsufficientBalance);
    assert_eq!(host.get_balance(&ADDRESS_ALICE), 10000 - 100);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 100);

    // bob is not callable (which means coins also cannot be sent, even if we have them)
    let address_bob = Address([0xBB; 24]);
    let msg = AthenaMessage::new(
      MessageKind::Call,
      0,
      1000,
      address_bob,
      ADDRESS_ALICE,
      None,
      100,
      vec![],
    );
    let res = host.call(msg);
    assert_eq!(res.status_code, StatusCode::Failure);
    assert_eq!(host.get_balance(&ADDRESS_ALICE), 10000 - 100);
    assert_eq!(host.get_balance(&address_bob), 0);
  }

  #[test]
  fn test_spawn() {
    let mut host = MockHost::new();
    let blob = vec![1, 2, 3, 4];
    let address = host.spawn_program(&ADDRESS_ALICE, blob.clone(), &ADDRESS_ALICE, 0);
    assert_eq!(host.get_program(&address), Some(&blob));
  }

  #[test]
  fn test_deploy() {
    let mut host = MockHost::new();
    let blob = vec![1, 2, 3, 4];
    let address = host.deploy(blob.clone());
    assert_eq!(*host.template(&address.unwrap()).unwrap(), blob);

    // deploying again should fail
    let address = host.deploy(blob.clone());
    assert!(address.is_err());
  }

  #[test]
  fn test_method_selector() {
    let selector = MethodSelector::from("test");
    assert_eq!(selector.0, [72, 120, 202, 4]);

    let selector = MethodSelector::from("test2");
    assert_eq!(selector.0, [116, 112, 75, 76]);
  }
}
