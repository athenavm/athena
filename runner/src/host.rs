use std::collections::BTreeMap;
use athcon_sys as ffi;
use athcon_host::host::HostContext as HostFfiInterface;

type Bytes32 = [u8; 32];
type Address = [u8; 24];

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

// struct FfiHost {
//   get_storage_callback: athcon_get_storage_fn,
//   set_storage_callback: athcon_set_storage_fn,
//   get_balance_callback: athcon_get_balance_fn,
// }

// impl FfiHost {
//   pub fn new(
//     get_storage_callback: athcon_get_storage_fn,
//     set_storage_callback: athcon_set_storage_fn,
//     get_balance_callback: athcon_get_balance_fn) -> Self {
//     FfiHost {
//       get_storage_callback,
//       set_storage_callback,
//       get_balance_callback,
//     }
//   }
// }

// impl AthconHost for FfiHost {
//   fn get_storage(&self, address: &[u8; 24], key: &[u8; 32]) -> Option<[u8; 32]> {
//     unsafe {
//       let mut value = [0u8; 32];
//       if let Some(get_storage_fn) = self.get_storage_callback {
//         let status = get_storage_fn(
//           address as *const _ as *const athcon_address,
//           key as *const _ as *const athcon_bytes32,
//         );
//         if status == athcon_storage_status::SUCCESS {
//           return Some(value);
//         } else {
//           None
//         }
//       } else {
//         None
//       }
//     }
//   }

//   fn set_storage(&self, key: &[u8; 32], value: &[u8; 32]) -> bool {
//     unsafe {
//       let status = athcon_set_storage_fn(
//         context,
//         key as *const _ as *const athcon_bytes32,
//         value as *const _ as *const athcon_bytes32
//       );
//       status == athcon_storage_status::SUCCESS
//     }
//   assert_eq!(host.set_storage(&address, &key, &value), StorageStatus::StorageAssigned)}

//   fn get_balance(&self, address: &[u8; 20]) -> u128 {
//     unsafe {
//       let balance = athcon_get_balance_fn(
//         context,
//         address as *const _ as *const athcon_address
//       );
//       // Assuming `athcon_uint256be` can be converted to u128
//       balance.to_u128()
//     }
//   }

//   // Implement other methods as needed
// }


#[cfg(test)]
mod tests {
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
        // take most significant 8 bytes
        return u64::from_le_bytes([
          b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]);
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
