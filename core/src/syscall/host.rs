use crate::runtime::{Register, Syscall, SyscallContext};
use athena_interface::{AddressWrapper, Bytes32Wrapper, ADDRESS_LENGTH, BYTES32_LENGTH};

pub struct SyscallHostRead;

impl SyscallHostRead {
  pub const fn new() -> Self {
    Self
  }
}

impl Syscall for SyscallHostRead {
  fn execute(&self, ctx: &mut SyscallContext, arg1: u32, arg2: u32) -> Option<u32> {
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

impl Syscall for SyscallHostWrite {
  fn execute(&self, ctx: &mut SyscallContext, arg1: u32, arg2: u32) -> Option<u32> {
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
