use athcon_sys as ffi;

/// ATHCON address
pub type Address = ffi::athcon_address;

/// ATHCON 32 bytes value (used for hashes)
pub type Bytes32 = ffi::athcon_bytes32;

/// ATHCON big-endian 256-bit integer
pub type Uint256 = ffi::athcon_uint256be;

/// ATHCON call kind.
pub type MessageKind = ffi::athcon_call_kind;

/// ATHCON status code.
pub type StatusCode = ffi::athcon_status_code;

/// ATHCON storage status.
pub type StorageStatus = ffi::athcon_storage_status;

/// ATHCON VM revision.
pub type Revision = ffi::athcon_revision;

#[cfg(test)]
mod tests {
  use super::*;

  // These tests check for Default, PartialEq and Clone traits.
  #[test]
  fn address_smoke_test() {
    let a = ffi::athcon_address::default();
    let b = Address::default();
    assert_eq!(a.clone(), b.clone());
  }

  #[test]
  fn bytes32_smoke_test() {
    let a = ffi::athcon_bytes32::default();
    let b = Bytes32::default();
    assert_eq!(a.clone(), b.clone());
  }

  #[test]
  fn uint26be_smoke_test() {
    let a = ffi::athcon_uint256be::default();
    let b = Uint256::default();
    assert_eq!(a.clone(), b.clone());
  }

  #[test]
  fn status_code() {
    assert_eq!(
      StatusCode::ATHCON_SUCCESS,
      ffi::athcon_status_code::ATHCON_SUCCESS
    );
    assert_eq!(
      StatusCode::ATHCON_FAILURE,
      ffi::athcon_status_code::ATHCON_FAILURE
    );
  }

  #[test]
  fn storage_status() {
    assert_eq!(
      StorageStatus::ATHCON_STORAGE_ASSIGNED,
      ffi::athcon_storage_status::ATHCON_STORAGE_ASSIGNED
    );
    assert_eq!(
      StorageStatus::ATHCON_STORAGE_MODIFIED,
      ffi::athcon_storage_status::ATHCON_STORAGE_MODIFIED
    );
  }

  #[test]
  fn revision() {
    assert_eq!(
      Revision::ATHCON_FRONTIER,
      ffi::athcon_revision::ATHCON_FRONTIER
    );
  }
}
