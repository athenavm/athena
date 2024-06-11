use crate::bindings::*;

trait AthconHost {
  fn get_storage(&self, address: &[u8; 20], key: &[u8; 32]) -> Option<[u8; 32]>;
  fn set_storage(&self, address: &[u8; 20], key: &[u8; 32], value: &[u8; 32]) -> bool;
  fn get_balance(&self, address: &[u8; 20]) -> u128;
  // Define other methods as needed
}

struct FfiHost {
  get_storage_callback: athcon_get_storage_fn,
  set_storage_callback: athcon_set_storage_fn,
  get_balance_callback: athcon_get_balance_fn,
}

impl FfiHost {
  pub fn new(
    get_storage_callback: athcon_get_storage_fn,
    set_storage_callback: athcon_set_storage_fn,
    get_balance_callback: athcon_get_balance_fn) -> Self {
    FfiHost {
      get_storage_callback,
      set_storage_callback,
      get_balance_callback,
    }
  }
}

impl AthconHost for FfiHost {
  fn get_storage(&self, address: &[u8; 24], key: &[u8; 32]) -> Option<[u8; 32]> {
    unsafe {
      let mut value = [0u8; 32];
      if let Some(get_storage_fn) = self.get_storage_callback {
        let status = get_storage_fn(
          address as *const _ as *const athcon_address,
          key as *const _ as *const athcon_bytes32,
        );
        if status == athcon_storage_status::SUCCESS {
          return Some(value);
        } else {
          None
        }
      } else {
        None
      }
    }
  }

  fn set_storage(&self, key: &[u8; 32], value: &[u8; 32]) -> bool {
    unsafe {
      let status = athcon_set_storage_fn(
        context,
        key as *const _ as *const athcon_bytes32,
        value as *const _ as *const athcon_bytes32
      );
      status == athcon_storage_status::SUCCESS
    }
  }

  fn get_balance(&self, address: &[u8; 20]) -> u128 {
    unsafe {
      let balance = athcon_get_balance_fn(
        context,
        address as *const _ as *const athcon_address
      );
      // Assuming `athcon_uint256be` can be converted to u128
      balance.to_u128()
    }
  }

  // Implement other methods as needed
}

struct MockHost;

impl AthconHost for MockHost {
  fn get_storage(&self, key: &[u8; 32]) -> Option<[u8; 32]> {
    // Mock implementation
    Some(*key)
  }

  fn set_storage(&self, key: &[u8; 32], value: &[u8; 32]) -> bool {
    // Mock implementation
    true
  }

  fn get_balance(&self, address: &[u8; 20]) -> u128 {
    // Mock implementation
    1000
  }
  // Implement other methods as needed
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_storage() {
    let host = MockHost;
    assert_eq!(host.get_storage(&[1; 32]), Some([1; 32]));
  }

  // Add more tests as needed
}
