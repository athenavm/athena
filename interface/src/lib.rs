//! # Athena Interface
//!
//! A library with no external dependencies that includes core types and traits.

mod context;
pub mod payload;
use bytemuck::NoUninit;
pub use context::*;

pub use parity_scale_codec::{Decode, Encode};

use std::{convert::TryFrom, fmt};

pub const ADDRESS_LENGTH: usize = 24;
pub const BYTES32_LENGTH: usize = 32;
pub const METHOD_SELECTOR_LENGTH: usize = 4;
pub type Balance = u64;
pub type Bytes32 = [u8; BYTES32_LENGTH];
pub type Bytes = [u8];

#[derive(Clone, Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub struct MethodSelector(pub [u8; METHOD_SELECTOR_LENGTH]);

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

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Decode, Encode, NoUninit)]
pub struct Address(pub [u8; ADDRESS_LENGTH]);

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
  ) -> Self {
    AthenaMessage {
      kind,
      depth,
      gas,
      recipient,
      sender,
      input_data,
      value,
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_method_selector() {
    let selector = MethodSelector::from("test");
    assert_eq!(selector.0, [72, 120, 202, 4]);

    let selector = MethodSelector::from("test2");
    assert_eq!(selector.0, [116, 112, 75, 76]);
  }
}
