use athcon_sys as ffi;
use athcon_client::host::HostContext as HostFfiInterface;

type Address = [u8; 24];
type Bytes32 = [u8; 32];
struct Bytes32AsBalance(Bytes32);

impl From<Bytes32AsBalance> for u64 {
  fn from(bytes: Bytes32AsBalance) -> Self {
    // take most significant 8 bytes, assume little-endian
    let slice = &bytes.0[..8];
    u64::from_le_bytes(slice.try_into().expect("slice with incorrect length"))
  }
}

trait HostInterface {
  fn get_storage(&self, address: &Address, key: &Bytes32) -> Option<Bytes32>;
  fn set_storage(&mut self, address: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus;
  fn get_balance(&self, address: &Address) -> u64;
  // Define other methods as needed
}

#[derive(Debug, PartialEq)]
enum StorageStatus {
  StorageAssigned = 0,
  StorageAdded = 1,
  StorageDeleted = 2,
  StorageModified = 3,
  StorageDeletedAdded = 4,
  StorageModifiedDeleted = 5,
  StorageDeletedRestored = 6,
  StorageAddedDeleted = 7,
  StorageModifiedRestored = 8,
}

impl From<ffi::athcon_storage_status> for StorageStatus {
  fn from(status: ffi::athcon_storage_status) -> Self {
    match status {
      ffi::athcon_storage_status::ATHCON_STORAGE_ASSIGNED => StorageStatus::StorageAssigned,
      ffi::athcon_storage_status::ATHCON_STORAGE_ADDED => StorageStatus::StorageAdded,
      ffi::athcon_storage_status::ATHCON_STORAGE_DELETED => StorageStatus::StorageDeleted,
      ffi::athcon_storage_status::ATHCON_STORAGE_MODIFIED => StorageStatus::StorageModified,
      ffi::athcon_storage_status::ATHCON_STORAGE_DELETED_ADDED => StorageStatus::StorageDeletedAdded,
      ffi::athcon_storage_status::ATHCON_STORAGE_MODIFIED_DELETED => StorageStatus::StorageModifiedDeleted,
      ffi::athcon_storage_status::ATHCON_STORAGE_DELETED_RESTORED => StorageStatus::StorageDeletedRestored,
      ffi::athcon_storage_status::ATHCON_STORAGE_ADDED_DELETED => StorageStatus::StorageAddedDeleted,
      ffi::athcon_storage_status::ATHCON_STORAGE_MODIFIED_RESTORED => StorageStatus::StorageModifiedRestored,
    }
  }
}

struct FfiHost<T: HostFfiInterface> {
  host: T,
}

impl<T: HostFfiInterface> HostInterface for FfiHost<T> {
  fn get_storage(&self, address: &Address, key: &Bytes32) -> Option<Bytes32> {
    let storage = self.host.get_storage(address, key);
    return Some(storage);
  }

  fn set_storage(&mut self, address: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
    let ffi_status = self.host.set_storage(address, key, value);
    return ffi_status.into();
  }

  fn get_balance(&self, address: &Address) -> u64 {
    let ffi_balance = self.host.get_balance(address);
    return Bytes32AsBalance(ffi_balance).into();
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

  impl HostInterface for MockHost {
    fn get_storage(&self, address: &Address, key: &Bytes32) -> Option<Bytes32> {
      return self.storage.get(&(*address, *key)).cloned();
    }

    fn set_storage(&mut self, address: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
      self.storage.insert((*address, *key), *value);
      return StorageStatus::StorageAssigned;
    }

    fn get_balance(&self, address: &Address) -> u64 {
      let balance = self.balance.get(address);
      if let Some(b) = balance {
        return Bytes32AsBalance(*b).into();
      } else {
        return 0;
      }
    }
  }

  #[test]
  fn test_get_storage() {
    let mut host = MockHost::new();
    let address = [8; 24];
    let key = [1; 32];
    let value = [2; 32];
    assert_eq!(host.set_storage(&address, &key, &value), StorageStatus::StorageAssigned);
    let retrieved_value = host.get_storage(&address, &key);
    match retrieved_value {
      Some(v) => assert_eq!(v, value),
      None => panic!("Value not found"),
    }
  }

  #[test]
  fn test_get_balance() {
    let host = MockHost::new();
    let address = [8; 24];
    let balance = host.get_balance(&address);
    assert_eq!(balance, 0);
  }
}
