use crate::syscall_write;
use crate::{syscall_hint_len, syscall_hint_read};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::alloc::Layout;
use std::io::Result;
use std::io::Write;

/// The file descriptor for public values.
pub const FD_PUBLIC_VALUES: u32 = 3;

/// The file descriptor for hints.
pub const FD_HINT: u32 = 4;

/// A writer that writes to a file descriptor inside the VM.
struct SyscallWriter {
  fd: u32,
}

impl Write for SyscallWriter {
  fn write(&mut self, buf: &[u8]) -> Result<usize> {
    let nbytes = buf.len();
    let write_buf = buf.as_ptr();
    unsafe {
      syscall_write(self.fd, write_buf, nbytes);
    }
    Ok(nbytes)
  }

  fn flush(&mut self) -> Result<()> {
    Ok(())
  }
}

/// Read a buffer from the input stream.
///
/// ### Examples
/// ```ignore
/// let data: Vec<u8> = athena_vm::io::read_vec();
/// ```
pub fn read_vec() -> Vec<u8> {
  // Round up to the nearest multiple of 4 so that the memory allocated is in whole words
  let len = unsafe { syscall_hint_len() };
  let capacity = (len + 3) / 4 * 4;

  // Allocate a buffer of the required length that is 4 byte aligned
  let layout = Layout::from_size_align(capacity, 4).expect("vec is too large");
  let ptr = unsafe { std::alloc::alloc(layout) };

  // SAFETY:
  // 1. `ptr` was allocated using alloc
  // 2. We assuume that the VM global allocator doesn't dealloc
  // 3/6. Size is correct from above
  // 4/5. Length is 0
  // 7. Layout::from_size_align already checks this
  let mut vec = unsafe { Vec::from_raw_parts(ptr, 0, capacity) };

  // Read the vec into uninitialized memory. The syscall assumes the memory is uninitialized,
  // which should be true because the allocator does not dealloc, so a new alloc should be fresh.
  unsafe {
    syscall_hint_read(ptr, len);
    vec.set_len(len);
  }
  vec
}

/// Read a deserializable object from the input stream.
///
/// ### Examples
/// ```ignore
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct MyStruct {
///     a: u32,
///     b: u32,
/// }
///
/// let data: MyStruct = athena_vm::io::read();
/// ```
pub fn read<T: DeserializeOwned>() -> T {
  let vec = read_vec();
  bincode::deserialize(&vec).expect("deserialization failed")
}

/// Write a serializable object to the public values stream.
///
/// ### Examples
/// ```ignore
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct MyStruct {
///     a: u32,
///     b: u32,
/// }
///
/// let data = MyStruct {
///     a: 1,
///     b: 2,
/// };
/// athena_vm::io::write(&data);
/// ```
pub fn write<T: Serialize>(value: &T) {
  let writer = SyscallWriter {
    fd: FD_PUBLIC_VALUES,
  };
  bincode::serialize_into(writer, value).expect("serialization failed");
}

/// Write bytes to the public values stream.
///
/// ### Examples
/// ```ignore
/// let data = vec![1, 2, 3, 4];
/// athena_vm::io::write_slice(&data);
/// ```
pub fn write_slice(buf: &[u8]) {
  let mut my_writer = SyscallWriter {
    fd: FD_PUBLIC_VALUES,
  };
  my_writer.write_all(buf).unwrap();
}

/// Hint a serializable object to the hint stream.
///
/// ### Examples
/// ```ignore
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct MyStruct {
///     a: u32,
///     b: u32,
/// }
///
/// let data = MyStruct {
///     a: 1,
///     b: 2,
/// };
/// athena_vm::io::hint(&data);
/// ```
pub fn hint<T: Serialize>(value: &T) {
  let writer = SyscallWriter { fd: FD_HINT };
  bincode::serialize_into(writer, value).expect("serialization failed");
}

/// Hint bytes to the hint stream.
///
/// ### Examples
/// ```ignore
/// let data = vec![1, 2, 3, 4];
/// athena_vm::io::hint_slice(&data);
/// ```
pub fn hint_slice(buf: &[u8]) {
  let mut my_reader = SyscallWriter { fd: FD_HINT };
  my_reader.write_all(buf).unwrap();
}

#[derive(Default)]
pub struct Io {
  read_remainder: std::collections::VecDeque<u8>,
}

impl std::io::Read for Io {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    if self.read_remainder.is_empty() {
      self.read_remainder.extend(crate::io::read_vec());
    }
    self.read_remainder.read(buf)
  }
}

impl std::io::Write for Io {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    crate::io::write_slice(buf);
    Ok(buf.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}
