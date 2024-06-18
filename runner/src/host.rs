use std::fmt;

pub type Address = [u8; 24];
pub type Balance = u64;
pub type Bytes32 = [u8; 32];
pub type Bytes = [u8];
pub struct Bytes32AsBalance(Bytes32);

impl Bytes32AsBalance {
  pub fn new(bytes: Bytes32) -> Self {
    Bytes32AsBalance(bytes)
  }
}

impl From<Bytes32AsBalance> for u64 {
  fn from(bytes: Bytes32AsBalance) -> Self {
    // take most significant 8 bytes, assume little-endian
    let slice = &bytes.0[..8];
    u64::from_le_bytes(slice.try_into().expect("slice with incorrect length"))
  }
}

pub struct TransactionContext {
  pub gas_price: u64,
  pub origin: Address,
  pub block_height: u64,
  pub block_timestamp: u64,
  pub block_gas_limit: u64,
  pub chain_id: Bytes32,
}

#[derive(Debug)]
pub struct ExecutionResult {
    status_code: StatusCode,
    gas_left: i64,
    output: Option<Vec<u8>>,
    create_address: Option<Address>,
}

impl ExecutionResult {
  pub fn new(status_code: StatusCode, gas_left: i64, output: Option<Vec<u8>>, create_address: Option<Address>) -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageKind {
  Call,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AthenaMessage {
  pub kind: MessageKind,
  pub depth: i32,
  pub gas: i64,
  pub recipient: Address,
  pub sender: Address,
  pub input_data: Vec<u8>,
  pub value: Balance,
  pub code: Vec<u8>,
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

pub struct ExecutionContext {
  host: Box<dyn HostInterface>,
  tx_context: TransactionContext,
  // unused
  // context: *mut ffi::athcon_host_context,
}

impl ExecutionContext {
  pub fn new(host: Box<dyn HostInterface>) -> Self {
    ExecutionContext {
      tx_context: host.get_tx_context(),
      host,
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

// impl From<ffi::athcon_status_code> for StatusCode {
//   fn from(status: ffi::athcon_status_code) -> Self {
//     match status {
//       ffi::athcon_status_code::ATHCON_SUCCESS => StatusCode::Success,
//       ffi::athcon_status_code::ATHCON_FAILURE => StatusCode::Failure,
//       ffi::athcon_status_code::ATHCON_REVERT => StatusCode::Revert,
//       ffi::athcon_status_code::ATHCON_OUT_OF_GAS => StatusCode::OutOfGas,
//       ffi::athcon_status_code::ATHCON_UNDEFINED_INSTRUCTION => StatusCode::UndefinedInstruction,
//       ffi::athcon_status_code::ATHCON_INVALID_MEMORY_ACCESS => StatusCode::InvalidMemoryAccess,
//       ffi::athcon_status_code::ATHCON_CALL_DEPTH_EXCEEDED => StatusCode::CallDepthExceeded,
//       ffi::athcon_status_code::ATHCON_PRECOMPILE_FAILURE => StatusCode::PrecompileFailure,
//       ffi::athcon_status_code::ATHCON_CONTRACT_VALIDATION_FAILURE => StatusCode::ContractValidationFailure,
//       ffi::athcon_status_code::ATHCON_ARGUMENT_OUT_OF_RANGE => StatusCode::ArgumentOutOfRange,
//       ffi::athcon_status_code::ATHCON_INSUFFICIENT_BALANCE => StatusCode::InsufficientBalance,
//       ffi::athcon_status_code::ATHCON_INTERNAL_ERROR => StatusCode::InternalError,
//       ffi::athcon_status_code::ATHCON_REJECTED => StatusCode::Rejected,
//       ffi::athcon_status_code::ATHCON_OUT_OF_MEMORY => StatusCode::OutOfMemory,
//     }
//   }
// }

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;
  use super::*;

  struct MockHost {
    // stores state keyed by address and key
    storage: BTreeMap<(Address, Bytes32), Bytes32>,

    // stores balance keyed by address
    balance: BTreeMap<Address, Bytes32>,
  }

  impl MockHost {
    pub fn new() -> Self {
      MockHost {
        storage: BTreeMap::new(),
        balance: BTreeMap::new(),
      }
    }
  }

  // impl HostInterface for MockHost {
  //   fn get_storage(&self, address: &Address, key: &Bytes32) -> Option<Bytes32> {
  //     return self.storage.get(&(*address, *key)).cloned();
  //   }

  //   fn set_storage(&mut self, address: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
  //     self.storage.insert((*address, *key), *value);
  //     return StorageStatus::StorageAssigned;
  //   }

  //   fn get_balance(&self, address: &Address) -> u64 {
  //     let balance = self.balance.get(address);
  //     if let Some(b) = balance {
  //       return Bytes32AsBalance(*b).into();
  //     } else {
  //       return 0;
  //     }
  //   }
  // }

  // #[test]
  // fn test_get_storage() {
  //   let mut host = MockHost::new();
  //   let address = [8; 24];
  //   let key = [1; 32];
  //   let value = [2; 32];
  //   assert_eq!(host.set_storage(&address, &key, &value), StorageStatus::StorageAssigned);
  //   let retrieved_value = host.get_storage(&address, &key);
  //   match retrieved_value {
  //     Some(v) => assert_eq!(v, value),
  //     None => panic!("Value not found"),
  //   }
  // }

  // #[test]
  // fn test_get_balance() {
  //   let host = MockHost::new();
  //   let address = [8; 24];
  //   let balance = host.get_balance(&address);
  //   assert_eq!(balance, 0);
  // }
}
