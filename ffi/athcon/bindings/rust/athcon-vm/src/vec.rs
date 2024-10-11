#[no_mangle]
/// Free the memory of an athcon_vector.
///
/// # Safety
/// The athcon_vector pointer must have been obtained from a Box<Vec<u8>>.
pub unsafe extern "C" fn athcon_free_vector(v: *mut athcon_sys::athcon_vector) {
  let v = Box::from_raw(v);
  let _ = Vec::from_raw_parts(v.ptr as *mut u8, v.len, v.cap);
}
