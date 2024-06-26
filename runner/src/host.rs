use athena_interface::{Bytes32, HostInterface, TransactionContext};

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
  host: Arc<RefCell<dyn HostInterface>>,
  tx_context: TransactionContext,
  // unused
  // context: *mut ffi::athcon_host_context,
}

impl ExecutionContext {
  pub fn new(host: Arc<RefCell<dyn HostInterface>>) -> Self {
    ExecutionContext {
      tx_context: host.clone().borrow().get_tx_context(),
      host: host,
    }
  }

  pub fn get_host(&self) -> Arc<RefCell<dyn HostInterface>> {
    self.host.clone()
  }

  pub fn get_tx_context(&self) -> &TransactionContext {
    &self.tx_context
  }
}

#[cfg(test)]
use std::collections::BTreeMap;
use std::{cell::RefCell, sync::Arc};

#[cfg(test)]
use athena_interface::{Address, AthenaMessage, ExecutionResult, StorageStatus};

#[cfg(test)]
pub(crate) struct MockHost {
  context: Option<TransactionContext>,

  // stores state keyed by address and key
  storage: BTreeMap<(Address, Bytes32), Bytes32>,

  // stores balance keyed by address
  balance: BTreeMap<Address, Bytes32>,
}

#[cfg(test)]
impl MockHost {
  pub fn new(context: Option<TransactionContext>) -> Self {
    MockHost {
      context: context,
      storage: BTreeMap::new(),
      balance: BTreeMap::new(),
    }
  }
}

#[cfg(test)]
impl HostInterface for MockHost {
  fn account_exists(&self, addr: &Address) -> bool {
    self.balance.contains_key(addr)
  }

  // return null bytes if the account or key do not exist
  fn get_storage(&self, address: &Address, key: &Bytes32) -> Bytes32 {
    *self
      .storage
      .get(&(*address, *key))
      .unwrap_or(&Bytes32::default())
  }

  fn set_storage(&mut self, address: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
    self.storage.insert((*address, *key), *value);
    return StorageStatus::StorageAssigned;
  }

  // return 0 if the account does not exist
  fn get_balance(&self, address: &Address) -> u64 {
    self
      .balance
      .get(address)
      .map_or_else(|| 0, |balance| Bytes32AsU64(*balance).into())
  }

  fn get_tx_context(&self) -> TransactionContext {
    self.context.unwrap()
  }

  fn get_block_hash(&self, _number: i64) -> Bytes32 {
    Bytes32::default()
  }

  fn call(&mut self, _msg: AthenaMessage) -> ExecutionResult {
    ExecutionResult::new(athena_interface::StatusCode::Failure, 0, None, None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_storage() {
    let mut host = MockHost::new(None);
    let address = [8; 24];
    let key = [1; 32];
    let value = [2; 32];
    assert_eq!(
      host.set_storage(&address, &key, &value),
      StorageStatus::StorageAssigned
    );
    let retrieved_value = host.get_storage(&address, &key);
    assert_eq!(retrieved_value, value);
  }

  #[test]
  fn test_get_balance() {
    let host = MockHost::new(None);
    let address = [8; 24];
    let balance = host.get_balance(&address);
    assert_eq!(balance, 0);
  }
}
