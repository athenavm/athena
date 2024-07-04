use std::borrow::BorrowMut;

use crate::runtime::{Register, Syscall, SyscallContext};
use athena_interface::{
  AddressWrapper, AthenaMessage, Bytes32Wrapper, HostInterface, MessageKind, ADDRESS_LENGTH,
  BYTES32_LENGTH,
};

pub struct SyscallHostRead;

impl SyscallHostRead {
  pub const fn new() -> Self {
    Self
  }
}

impl<T> Syscall<T> for SyscallHostRead
where
  T: HostInterface,
{
  fn execute(&self, ctx: &mut SyscallContext<T>, arg1: u32, arg2: u32) -> Option<u32> {
    // marshal inputs
    let address_words = ADDRESS_LENGTH / 4;
    let key = ctx.slice_unsafe(arg1, BYTES32_LENGTH / 4);
    let address = ctx.slice_unsafe(arg2, address_words);

    // read value from host
    let host = ctx.rt.host.as_mut().expect("Missing host interface");
    let value = host.borrow().get_storage(
      &AddressWrapper::from(address).into(),
      &Bytes32Wrapper::from(key).into(),
    );

    // set return value
    let value_vec: Vec<u32> = Bytes32Wrapper::new(value).into();
    ctx.mw_slice(arg1, value_vec.as_slice());
    None
  }
}

pub struct SyscallHostWrite;

impl SyscallHostWrite {
  pub const fn new() -> Self {
    Self
  }
}

impl<T> Syscall<T> for SyscallHostWrite
where
  T: HostInterface,
{
  fn execute(&self, ctx: &mut SyscallContext<T>, arg1: u32, arg2: u32) -> Option<u32> {
    // marshal inputs
    let address_words = ADDRESS_LENGTH / 4;
    let key = ctx.slice_unsafe(arg1, BYTES32_LENGTH / 4);
    let address = ctx.slice_unsafe(arg2, address_words);

    // we need to read the value to write from the next register
    let a2 = Register::X12;
    let rt = &mut ctx.rt;
    let value_ptr = rt.register(a2);
    let value = ctx.slice_unsafe(value_ptr, BYTES32_LENGTH / 4);

    // write value to host
    let host = ctx.rt.host.as_mut().expect("Missing host interface");
    let status_code = host.borrow_mut().set_storage(
      &AddressWrapper::from(address).into(),
      &Bytes32Wrapper::from(key).into(),
      &Bytes32Wrapper::from(value).into(),
    );

    // save return code
    let mut status_word = [0u32; 8];
    status_word[0] = status_code as u32;
    ctx.mw_slice(arg1, &status_word);
    None
  }
}

pub struct SyscallHostCall;

impl SyscallHostCall {
  pub const fn new() -> Self {
    Self
  }
}

impl<T> Syscall<T> for SyscallHostCall
where
  T: HostInterface,
{
  fn execute(&self, ctx: &mut SyscallContext<T>, arg1: u32, arg2: u32) -> Option<u32> {
    // make sure we have a runtime context
    let athena_ctx = ctx
      .rt
      .context
      .as_ref()
      .expect("Missing Athena runtime context");
    let host = ctx
      .rt
      .host
      .as_ref()
      .expect("Missing host interface")
      .borrow_mut();

    // get remaining gas
    // note: this does not factor in the cost of the current instruction
    let gas_left = ctx.rt.gas_left().expect("Missing gas information");

    // note: the host is responsible for checking stack depth, not us

    // marshal inputs
    let address_words = ADDRESS_LENGTH / 4;
    let address = ctx.slice_unsafe(arg1, address_words);
    let address = AddressWrapper::from(address);

    // we need to read the input length from the next register
    let a2 = Register::X12;
    let rt = &mut ctx.rt;
    let len = rt.register(a2) as usize;
    // let len = ctx.word_unsafe(len_ptr);

    // `len` is denominated in number of bytes; we read words in chunks of four bytes
    let input = ctx.slice_unsafe(arg2, len / 4);

    // construct the outbound message
    let msg = AthenaMessage::new(
      MessageKind::Call,
      athena_ctx.depth() + 1,
      u32::try_from(gas_left).expect("Invalid gas left"),
      address.into(),
      athena_ctx.address().clone(),
      None,
      0,
      Vec::new(),
    );
    Some(host.call(msg) as u32)
  }
}
