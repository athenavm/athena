//! # Athena Interface
//!
//! A library with no external dependencies that includes core types and traits.

use std::{
  fmt,
  ops::{Deref, DerefMut},
};

pub const ADDRESS_LENGTH: usize = 24;
pub const BYTES32_LENGTH: usize = 32;
pub type Address = [u8; ADDRESS_LENGTH];
pub type Balance = u64;
pub type Bytes32 = [u8; BYTES32_LENGTH];
pub type Bytes = [u8];

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
  pub depth: i32,
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
    depth: i32,
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
  fn account_exists(&self, addr: &Address) -> bool;
  fn get_storage(&self, addr: &Address, key: &Bytes32) -> Bytes32;
  fn set_storage(&mut self, addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus;
  fn get_balance(&self, addr: &Address) -> Balance;
  fn get_tx_context(&self) -> TransactionContext;
  fn get_block_hash(&self, number: i64) -> Bytes32;
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
pub struct MockHost;

impl MockHost {
  pub fn new() -> Self {
    MockHost {}
  }
}

impl HostInterface for MockHost {
  fn account_exists(&self, _addr: &Address) -> bool {
    true
  }

  fn get_storage(&self, _addr: &Address, _key: &Bytes32) -> Bytes32 {
    // return all 1's
    [1u8; BYTES32_LENGTH]
  }

  fn set_storage(&mut self, _addr: &Address, _key: &Bytes32, _value: &Bytes32) -> StorageStatus {
    StorageStatus::StorageAssigned
  }

  fn get_balance(&self, _addr: &Address) -> u64 {
    0
  }

  fn get_tx_context(&self) -> TransactionContext {
    unimplemented!()
  }

  fn get_block_hash(&self, _block_height: i64) -> Bytes32 {
    unimplemented!()
  }

  fn call(&mut self, _msg: AthenaMessage) -> ExecutionResult {
    unimplemented!()
  }
}
