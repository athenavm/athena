use crate::syscalls::syscall_write;
use crate::syscalls::{syscall_hint_len, syscall_hint_read};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{Result, Write};

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
    syscall_write(self.fd, write_buf, nbytes);
    Ok(nbytes)
  }

  fn flush(&mut self) -> Result<()> {
    Ok(())
  }
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
  bincode::deserialize_from(Io::default()).unwrap()
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
  bincode::serialize_into(writer, value).unwrap();
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
  bincode::serialize_into(writer, value).unwrap();
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
pub struct Io {}

impl std::io::Read for Io {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    let len = std::cmp::min(buf.len(), syscall_hint_len());
    if len == 0 {
      return Ok(0);
    }

    syscall_hint_read(buf.as_mut_ptr(), len);
    Ok(len)
  }
}

impl std::io::Write for Io {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    write_slice(buf);
    Ok(buf.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}
