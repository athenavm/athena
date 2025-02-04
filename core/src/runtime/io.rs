use std::io::Read;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::Runtime;

impl Read for Runtime<'_> {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    self.read_public_values_slice(buf);
    Ok(buf.len())
  }
}

impl Runtime<'_> {
  pub fn write_stdin<U: Serialize>(&mut self, input: &U) {
    bincode::serialize_into(&mut self.state.input_stream, input).expect("serialization failed");
  }

  pub fn write_stdin_slice(&mut self, input: &[u8]) {
    self.write_from(input.iter().copied());
  }

  pub fn write_from<T: IntoIterator<Item = u8>>(&mut self, input: T) {
    self.state.input_stream.extend(input);
  }

  pub fn read_public_values<U: DeserializeOwned>(&mut self) -> U {
    let result = bincode::deserialize_from::<_, U>(self);
    result.unwrap()
  }

  pub fn read_public_values_slice(&mut self, buf: &mut [u8]) {
    let len = buf.len();
    let start = self.state.public_values_stream_ptr;
    let end = start + len;
    assert!(end <= self.state.public_values_stream.len());
    buf.copy_from_slice(&self.state.public_values_stream[start..end]);
    self.state.public_values_stream_ptr = end;
  }
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use crate::runtime::tests::setup_logger;
  use crate::runtime::Program;
  use crate::utils::{tests::IO_ELF, AthenaCoreOpts};
  use serde::Deserialize;

  #[derive(Serialize, Deserialize, Debug, PartialEq)]
  struct MyPointUnaligned {
    pub x: usize,
    pub y: usize,
    pub b: bool,
  }

  fn points() -> (MyPointUnaligned, MyPointUnaligned) {
    (
      MyPointUnaligned {
        x: 3,
        y: 5,
        b: true,
      },
      MyPointUnaligned {
        x: 8,
        y: 19,
        b: true,
      },
    )
  }

  #[test]
  fn test_io_run() {
    setup_logger();
    let program = Program::from(IO_ELF).unwrap();
    let mut runtime = Runtime::new(program, None, AthenaCoreOpts::default(), None);
    let points = points();
    runtime.write_stdin(&points.0);
    runtime.write_stdin(&points.1);
    runtime.execute().unwrap();
    let added_point = runtime.read_public_values::<MyPointUnaligned>();
    assert_eq!(
      added_point,
      MyPointUnaligned {
        x: 11,
        y: 24,
        b: true
      }
    );
  }
}
