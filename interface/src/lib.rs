//! # Athena Interface
//!
//! A library with no external dependencies that includes core types and traits.

mod context;
pub use context::*;

use std::{
  cell::RefCell,
  collections::BTreeMap,
  convert::TryFrom,
  fmt,
  ops::{Deref, DerefMut},
  sync::Arc,
};

pub const ADDRESS_LENGTH: usize = 24;
pub const BYTES32_LENGTH: usize = 32;
pub type Address = [u8; ADDRESS_LENGTH];
pub type Balance = u64;
pub type Bytes32 = [u8; BYTES32_LENGTH];
pub type Bytes = [u8];

pub struct Bytes32AsU64(Bytes32);

impl Bytes32AsU64 {
  pub fn new(bytes: Bytes32) -> Self {
    Bytes32AsU64(bytes)
  }
}

impl From<Bytes32AsU64> for u64 {
  fn from(bytes: Bytes32AsU64) -> Self {
    // take most significant 8 bytes, assume little-endian
    let slice = &bytes.0[..8];
    u64::from_le_bytes(slice.try_into().expect("slice with incorrect length"))
  }
}

impl From<Bytes32AsU64> for Bytes32 {
  fn from(bytes: Bytes32AsU64) -> Self {
    bytes.0
  }
}

impl From<u64> for Bytes32AsU64 {
  fn from(value: u64) -> Self {
    let mut bytes = [0u8; 32];
    let value_bytes = value.to_le_bytes();
    bytes[..8].copy_from_slice(&value_bytes);
    Bytes32AsU64(bytes)
  }
}

pub struct AddressWrapper(Address);

impl From<Vec<u32>> for AddressWrapper {
  fn from(value: Vec<u32>) -> Self {
    assert!(value.len() == ADDRESS_LENGTH / 4, "Invalid address length");
    let mut bytes = [0u8; ADDRESS_LENGTH];
    // let mut value_bytes = [0u8; 4];
    for (i, word) in value.iter().enumerate() {
      let value_bytes = word.to_le_bytes();
      bytes[i * 4..(i + 1) * 4].copy_from_slice(&value_bytes);
    }
    AddressWrapper(bytes)
  }
}

impl From<AddressWrapper> for Address {
  fn from(value: AddressWrapper) -> Address {
    value.0
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

impl From<Address> for Bytes32Wrapper {
  fn from(value: Address) -> Bytes32Wrapper {
    let mut bytes = [0u8; 32];
    bytes[..24].copy_from_slice(&value);
    Bytes32Wrapper(bytes)
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
  UndefinedInstruction,
  InvalidMemoryAccess,
  CallDepthExceeded,
  PrecompileFailure,
  ContractValidationFailure,
  ArgumentOutOfRange,
  InsufficientBalance,
  InternalError,
  Rejected,
  OutOfMemory,
}

impl TryFrom<u32> for StatusCode {
  type Error = &'static str;

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      x if x == StatusCode::Success as u32 => Ok(StatusCode::Success),
      x if x == StatusCode::Failure as u32 => Ok(StatusCode::Failure),
      x if x == StatusCode::Revert as u32 => Ok(StatusCode::Revert),
      x if x == StatusCode::OutOfGas as u32 => Ok(StatusCode::OutOfGas),
      x if x == StatusCode::UndefinedInstruction as u32 => Ok(StatusCode::UndefinedInstruction),
      x if x == StatusCode::InvalidMemoryAccess as u32 => Ok(StatusCode::InvalidMemoryAccess),
      x if x == StatusCode::CallDepthExceeded as u32 => Ok(StatusCode::CallDepthExceeded),
      x if x == StatusCode::PrecompileFailure as u32 => Ok(StatusCode::PrecompileFailure),
      x if x == StatusCode::ContractValidationFailure as u32 => {
        Ok(StatusCode::ContractValidationFailure)
      }
      x if x == StatusCode::ArgumentOutOfRange as u32 => Ok(StatusCode::ArgumentOutOfRange),
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
      StatusCode::UndefinedInstruction => write!(f, "An undefined instruction has been encountered."),
      StatusCode::InvalidMemoryAccess => write!(f, "Tried to read outside memory bounds."),
      StatusCode::CallDepthExceeded => write!(f, "Call depth has exceeded the limit."),
      StatusCode::PrecompileFailure => write!(f, "A call to a precompiled or system contract has ended with a failure."),
      StatusCode::ContractValidationFailure => write!(f, "Contract validation has failed."),
      StatusCode::ArgumentOutOfRange => write!(f, "An argument to a state accessing method has a value outside of the accepted range."),
      StatusCode::InsufficientBalance => write!(f, "The caller does not have enough funds for value transfer."),
      StatusCode::InternalError => write!(f, "Athena implementation generic internal error."),
      StatusCode::Rejected => write!(f, "The execution of the given code and/or message has been rejected by the Athena implementation."),
      StatusCode::OutOfMemory => write!(f, "The VM failed to allocate the amount of memory needed for execution."),
    }
  }
}

#[derive(Debug)]
pub struct ExecutionResult {
  pub status_code: StatusCode,
  pub gas_left: u32,
  pub output: Option<Vec<u8>>,
  pub create_address: Option<Address>,
}

impl ExecutionResult {
  pub fn new(
    status_code: StatusCode,
    gas_left: u32,
    output: Option<Vec<u8>>,
    create_address: Option<Address>,
  ) -> Self {
    ExecutionResult {
      status_code,
      gas_left,
      output,
      create_address,
    }
  }
}

pub trait HostInterface {
  fn get_storage(&self, addr: &Address, key: &Bytes32) -> Bytes32;
  fn set_storage(&mut self, addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus;
  fn get_balance(&self, addr: &Address) -> Balance;
  fn call(&mut self, msg: AthenaMessage) -> ExecutionResult;
}

// provide a trait-bound generic struct to represent the host interface
// this is better, and more performant, than using a trait object
// since it allows more compile-time checks, and we don't need polymorphism.
pub struct HostProvider<T: HostInterface> {
  host: T,
}

impl<T> HostProvider<T>
where
  T: HostInterface,
{
  pub fn new(host: T) -> Self {
    HostProvider { host }
  }
}

// pass calls directly to the underlying host instance
impl<T> Deref for HostProvider<T>
where
  T: HostInterface,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.host
  }
}

impl<T> DerefMut for HostProvider<T>
where
  T: HostInterface,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.host
  }
}

// a very simple mock host implementation for testing
// also useful for filling in the missing generic type
// when running the VM in standalone mode, without a bound host interface
pub struct MockHost<'a> {
  // VM instance
  vm: Option<&'a dyn VmInterface<MockHost<'a>>>,

  // stores state keyed by address and key
  storage: BTreeMap<(Address, Bytes32), Bytes32>,

  // stores balance keyed by address
  balance: BTreeMap<Address, Balance>,

  // stores contract code
  programs: BTreeMap<Address, &'a [u8]>,
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

  pub fn deploy_code(&mut self, address: Address, code: &'a [u8]) {
    self.programs.insert(address, code);
  }

  fn transfer_balance(&mut self, from: &Address, to: &Address, value: u64) -> StatusCode {
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

pub const ADDRESS_ALICE: Address = [1u8; ADDRESS_LENGTH];
pub const ADDRESS_BOB: Address = [2u8; ADDRESS_LENGTH];
pub const ADDRESS_CHARLIE: Address = [3u8; ADDRESS_LENGTH];
pub const SOME_COINS: Balance = 1000000;
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

impl<'a> Default for MockHost<'a> {
  fn default() -> Self {
    // init
    let mut storage = BTreeMap::new();
    let mut balance = BTreeMap::new();
    let programs = BTreeMap::new();

    // pre-populate some balances, values, and code for testing
    balance.insert(ADDRESS_ALICE, SOME_COINS);
    // balance.insert(ADDRESS_BOB, SOME_COINS);
    storage.insert((ADDRESS_ALICE, STORAGE_KEY), STORAGE_VALUE);

    Self {
      vm: None,
      storage,
      balance,
      programs,
    }
  }
}

impl<'a> HostInterface for MockHost<'a> {
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

  fn call(&mut self, msg: AthenaMessage) -> ExecutionResult {
    let depth = msg.depth;
    log::info!("MockHost::call:depth {} :: {:?}", msg.depth, msg);

    // don't go too deep!
    if msg.depth > 10 {
      return ExecutionResult::new(StatusCode::CallDepthExceeded, 0, None, None);
    }

    // take snapshots of the state in case we need to roll back
    // this is relatively expensive and we'd want to do something more sophisticated in production
    // (journaling? CoW?) but it's fine for testing.
    log::info!(
      "MockHost::call:depth {} before backup storage item is :: {:?}",
      depth,
      self.get_storage(&ADDRESS_ALICE, &STORAGE_KEY)
    );
    let backup_storage = self.storage.clone();
    let backup_balance = self.balance.clone();
    let backup_programs = self.programs.clone();

    // transfer balance
    // note: the host should have already subtracted an amount from the sender
    // equal to the maximum amount of gas that could be paid, so this should
    // not allow an out of gas error.
    match self.transfer_balance(&msg.sender, &msg.recipient, msg.value) {
      StatusCode::Success => {}
      status => {
        return ExecutionResult::new(status, 0, None, None);
      }
    }

    // check programs list first
    let res = if let Some(code) = self.programs.get(&msg.recipient).cloned() {
      // create an owned, cloned copy of VM before taking the host from self
      let vm = self.vm.clone();

      // HostProvider requires an owned instance, so we need to take it from self
      let provider = HostProvider::new(std::mem::take(self));
      let host = Arc::new(RefCell::new(provider));
      let res = vm.expect("missing VM instance").execute(
        host.clone(),
        AthenaRevision::AthenaFrontier,
        msg,
        code,
      );

      // Restore self
      *self = Arc::try_unwrap(host)
        .unwrap_or_else(|_| panic!("Arc still has multiple strong references"))
        .into_inner()
        .host;
      res
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
      ExecutionResult::new(status_code, gas_left, None, None)
    };

    if res.status_code != StatusCode::Success {
      log::info!(
        "MockHost::call:depth {} before restore storage item is :: {:?}",
        depth,
        self.get_storage(&ADDRESS_ALICE, &STORAGE_KEY)
      );
      // rollback state
      self.storage = backup_storage;
      self.balance = backup_balance;
      self.programs = backup_programs;
      log::info!(
        "MockHost::call:depth {} after restore storage item is :: {:?}",
        depth,
        self.get_storage(&ADDRESS_ALICE, &STORAGE_KEY)
      );
    }
    res
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
    host: Arc<RefCell<HostProvider<T>>>,
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
    let address = [8; 24];
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
    let address = [8; 24];
    let balance = host.get_balance(&address);
    assert_eq!(balance, 0);
  }

  #[test]
  fn test_transfer_balance() {
    let mut host = MockHost::new();
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 0);
    assert_eq!(host.get_balance(&ADDRESS_ALICE), SOME_COINS);
    assert_eq!(
      host.transfer_balance(&ADDRESS_ALICE, &ADDRESS_CHARLIE, 1000),
      StatusCode::Success
    );
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
    assert_eq!(host.get_balance(&ADDRESS_ALICE), SOME_COINS);
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
    assert_eq!(host.get_balance(&ADDRESS_ALICE), SOME_COINS);
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
    assert_eq!(host.get_balance(&ADDRESS_ALICE), SOME_COINS - 100);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 100);

    // try to send more than the sender has
    let msg = AthenaMessage::new(
      MessageKind::Call,
      0,
      1000,
      ADDRESS_CHARLIE,
      ADDRESS_ALICE,
      None,
      SOME_COINS,
      vec![],
    );
    let res = host.call(msg);
    assert_eq!(res.status_code, StatusCode::InsufficientBalance);
    assert_eq!(host.get_balance(&ADDRESS_ALICE), SOME_COINS - 100);
    assert_eq!(host.get_balance(&ADDRESS_CHARLIE), 100);

    // bob is not callable (which means coins also cannot be sent, even if we have them)
    let msg = AthenaMessage::new(
      MessageKind::Call,
      0,
      1000,
      ADDRESS_BOB,
      ADDRESS_ALICE,
      None,
      100,
      vec![],
    );
    let res = host.call(msg);
    assert_eq!(res.status_code, StatusCode::Failure);
    assert_eq!(host.get_balance(&ADDRESS_ALICE), SOME_COINS - 100);
    assert_eq!(host.get_balance(&ADDRESS_BOB), 0);
  }
}
