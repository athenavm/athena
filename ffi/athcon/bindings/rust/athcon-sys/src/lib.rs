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

impl athcon_vector_t {
  /// Convert Vec<u8> into athcon_vector, releasing ownership of the memory.
  ///
  /// # Safety
  /// the caller is responsible for freeing the returned vector
  /// via the `athcon_free_vector` function.
  pub unsafe fn from_vec(v: Vec<u8>) -> Self {
    let (ptr, len, cap) = (v.as_ptr(), v.len(), v.capacity());
    std::mem::forget(v);
    Self { ptr, len, cap }
  }

  /// Convert athcon_vector into Vec<u8>, claiming ownership of the memory.
  ///
  /// # Safety
  /// The self must have been constructed from a Vec<u8>.
  pub unsafe fn to_vec(self) -> Vec<u8> {
    Vec::from_raw_parts(self.ptr as *mut u8, self.len, self.cap)
  }

  /// Convert athcon_vector into slice
  ///
  /// # Safety
  /// The ptr and len must satisfy the safety requirements of
  /// std::slice::from_raw_parts.
  pub unsafe fn as_slice(&self) -> &[u8] {
    std::slice::from_raw_parts(self.ptr, self.len)
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
  fn vector_conversion() {
    let v = vec![1, 2, 3];
    let athcon_vec = unsafe { athcon_vector_t::from_vec(v.clone()) };
    assert_eq!(unsafe { athcon_vec.to_vec() }, v);
  }
}
