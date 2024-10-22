/// Free heap-allocated athcon_bytes.
///
/// # Safety
/// The athcon_bytes pointer must have been obtained from a `Box`
/// and the memory must have been obtained via `allocate_output_data`.
#[no_mangle]
pub unsafe extern "C" fn athcon_free_bytes(v: *mut athcon_sys::athcon_bytes) {
  let v = Box::from_raw(v);
  deallocate_output_data(v.ptr, v.size);
}

pub(crate) fn allocate_output_data<T: AsRef<[u8]>>(data: T) -> (*const u8, usize) {
  let buf = data.as_ref();
  if buf.as_ref().is_empty() {
    return (core::ptr::null(), 0);
  }

  let buf_len = buf.len();

  // Manually allocate heap memory for the new home of the output buffer.
  let memlayout = std::alloc::Layout::for_value(buf);
  let new_buf = unsafe { std::alloc::alloc(memlayout) };
  unsafe {
    // Copy the data into the allocated buffer.
    std::ptr::copy(buf.as_ptr(), new_buf, buf_len);
  }

  (new_buf as *const u8, buf_len)
}

pub(crate) unsafe fn deallocate_output_data(ptr: *const u8, size: usize) {
  // be careful with dangling, aligned pointers here; they are not null but
  // not valid and cannot be deallocated!
  if !ptr.is_null() && size > 0 {
    let slice = std::slice::from_raw_parts(ptr, size);
    let buf_layout = std::alloc::Layout::for_value(slice);
    std::alloc::dealloc(ptr as *mut u8, buf_layout);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn allocate_deallocate() {
    let (ptr, size) = allocate_output_data([0, 1, 2, 3]);
    assert!(!ptr.is_null());
    assert_eq!(size, 4);
    unsafe { deallocate_output_data(ptr, size) };
  }
}
