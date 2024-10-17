#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Defining athcon_host_context here, because bindgen cannot create a useful declaration yet.

/// This is a void type given host context is an opaque pointer. Functions allow it to be a null ptr.
pub type athcon_host_context = ::std::os::raw::c_void;

// TODO: add `.derive_default(true)` to bindgen instead?

#[allow(clippy::derivable_impls)]
impl Default for athcon_address {
  fn default() -> Self {
    athcon_address { bytes: [0u8; 24] }
  }
}

impl From<[u8; 24]> for athcon_address {
  fn from(value: [u8; 24]) -> Self {
    Self { bytes: value }
  }
}

#[allow(clippy::derivable_impls)]
impl Default for athcon_bytes32 {
  fn default() -> Self {
    athcon_bytes32 { bytes: [0u8; 32] }
  }
}

impl athcon_bytes_t {
  /// Convert athcon_bytes_t into slice
  ///
  /// # Safety
  /// The ptr and len must satisfy the safety requirements of
  /// std::slice::from_raw_parts.
  pub unsafe fn as_slice(&self) -> &[u8] {
    std::slice::from_raw_parts(self.ptr, self.size)
  }
}

impl From<&[u8]> for athcon_bytes_t {
  fn from(value: &[u8]) -> Self {
    Self {
      ptr: value.as_ptr(),
      size: value.len(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::mem::size_of;

  use super::*;

  #[test]
  fn container_new() {
    assert_eq!(size_of::<athcon_bytes32>(), 32);
    assert_eq!(size_of::<athcon_address>(), 24);
    assert!(size_of::<athcon_vm>() <= 64);
  }

  #[test]
  fn bytes_as_slice() {
    let s = &[1, 2, 3];
    let bytes = athcon_bytes_t::from(s.as_slice());
    assert_eq!(unsafe { bytes.as_slice() }, s);
  }
}
