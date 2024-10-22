//! Helpers for converting data (input arguments and result)
//! between the host IO and the program functions.
use std::io::{Read, Write};

use parity_scale_codec::{Decode, Encode, IoReader};

pub trait Function<T, R, IO> {
  fn call_func(self, io: &mut IO);
}

pub trait Method<T, R, IO> {
  fn call_method(self, io: &mut IO);
}

pub trait MethodMut<T, R, IO> {
  fn call_method_mut(self, io: &mut IO);
}

/// IntoArgument allows for creating an argument for a function call
/// from a reader.
pub trait IntoArgument<T, R>
where
  R: Read,
{
  fn into_argument(r: &mut R) -> T;
}

impl<T, R> IntoArgument<T, R> for T
where
  R: Read,
  T: Decode,
{
  fn into_argument(r: &mut R) -> T {
    T::decode(&mut IoReader(r)).expect("deserializing")
  }
}

/// IntoResult allows for converting the result of a function call
/// and writing it to a writer.
pub trait IntoResult<W>
where
  W: Write,
{
  fn into_result(self, w: &mut W);
}

impl<T, W> IntoResult<W> for T
where
  W: Write,
  T: Encode,
{
  fn into_result(self, w: &mut W) {
    self.encode_to(w);
  }
}

/// Implement Function for functions taking no arguments:
/// # Example:
/// ```ignore
///fn foo() -> impl IntoResult<IO>
/// ```
impl<F, IO, R> Function<(), R, IO> for F
where
  IO: Read + Write,
  F: Fn() -> R,
  R: IntoResult<IO>,
{
  fn call_func(self, io: &mut IO) {
    self().into_result(io);
  }
}

/// Implement Function for functions taking 1 argument:
/// # Example:
/// ```ignore
///fn foo(arg: impl IntoArgument<T1, IO>) -> impl IntoResult<IO>
/// ```
impl<F, IO, T1, R> Function<(T1,), R, IO> for F
where
  IO: Read + Write,
  F: Fn(T1) -> R,
  T1: IntoArgument<T1, IO>,
  R: IntoResult<IO>,
{
  fn call_func(self, io: &mut IO) {
    let arg1 = T1::into_argument(io);
    self(arg1).into_result(io);
  }
}

/// Implement Method for methods taking 1 argument:
/// # Example:
/// ```ignore
/// impl Foo {
///   fn foo(&self, arg: impl IntoArgument<T1, IO>) -> impl IntoResult<IO>
/// }
/// ```
impl<F, IO, S, T1, R> Method<(&S, T1), R, IO> for F
where
  IO: Read + Write,
  F: Fn(&S, T1) -> R,
  S: IntoArgument<S, IO>,
  T1: IntoArgument<T1, IO>,
  R: IntoResult<IO>,
{
  fn call_method(self, io: &mut IO) {
    let instance = S::into_argument(io);
    let arg1 = T1::into_argument(io);
    self(&instance, arg1).into_result(io);
  }
}

/// Implement Method for methods taking 2 arguments:
/// # Example:
/// ```ignore
/// impl Foo {
///   fn foo(&self, arg1: impl IntoArgument<T1, IO>, arg2: impl IntoArgument<T2, IO>) -> impl IntoResult<IO>
/// }
/// ```
impl<F, IO, S, T1, T2, R> Method<(&S, T1, T2), R, IO> for F
where
  IO: Read + Write,
  F: Fn(&S, T1, T2) -> R,
  S: IntoArgument<S, IO>,
  T1: IntoArgument<T1, IO>,
  T2: IntoArgument<T2, IO>,
  R: IntoResult<IO>,
{
  fn call_method(self, io: &mut IO) {
    let instance = S::into_argument(io);
    let arg1 = T1::into_argument(io);
    let arg2 = T2::into_argument(io);
    self(&instance, arg1, arg2).into_result(io);
  }
}

/// Implement Function for functions taking 2 arguments:
/// # Example:
/// ```ignore
///fn foo(arg: impl IntoArgument<T1, IO>, arg2:impl IntoArgument<T2, IO>) -> impl IntoResult<IO>
/// ```
impl<F, IO, T1, T2, R> Function<(T1, T2), R, IO> for F
where
  IO: Read + Write,
  F: Fn(T1, T2) -> R,
  T1: IntoArgument<T1, IO>,
  T2: IntoArgument<T2, IO>,
  R: IntoResult<IO>,
{
  fn call_func(self, io: &mut IO) {
    let arg1 = T1::into_argument(io);
    let arg2 = T2::into_argument(io);
    self(arg1, arg2).into_result(io);
  }
}

#[cfg(test)]
mod tests {
  use std::collections::VecDeque;

  use super::*;

  struct TestProgram {
    value: u64,
  }
  impl TestProgram {
    fn sub(&self, arg: u64) -> u64 {
      self.value - arg
    }
  }

  impl<R: Read> IntoArgument<TestProgram, R> for TestProgram {
    fn into_argument(r: &mut R) -> TestProgram {
      let mut buf = [0u8; 8];
      r.read_exact(&mut buf).unwrap();
      TestProgram {
        value: u64::from_le_bytes(buf),
      }
    }
  }

  #[test]
  fn test_takes_self() {
    let mut io = VecDeque::<u8>::new();
    100u64.encode_to(&mut io);
    77u64.encode_to(&mut io);

    Method::call_method(TestProgram::sub, &mut io);

    let result: u64 = u64::decode(&mut IoReader(io)).unwrap();
    assert_eq!(result, 23);
  }
}
