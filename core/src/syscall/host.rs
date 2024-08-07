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
  fn execute(&self, ctx: &mut SyscallContext<T>, arg1: u32, _arg2: u32) -> Option<u32> {
    let athena_ctx = ctx
      .rt
      .context
      .as_ref()
      .expect("Missing Athena runtime context");

    // marshal inputs
    let key = ctx.slice(arg1, BYTES32_LENGTH / 4);

    // read value from host
    let host = ctx.rt.host.as_mut().expect("Missing host interface");
    let value = host
      .borrow()
      .get_storage(athena_ctx.address(), &Bytes32Wrapper::from(key).into());

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
    let athena_ctx = ctx
      .rt
      .context
      .as_ref()
      .expect("Missing Athena runtime context");

    // marshal inputs
    let key = ctx.slice(arg1, BYTES32_LENGTH / 4);
    let value = ctx.slice(arg2, BYTES32_LENGTH / 4);

    // write value to host
    let host = ctx.rt.host.as_mut().expect("Missing host interface");
    let status_code = host.borrow_mut().set_storage(
      athena_ctx.address(),
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
    let mut host = ctx
      .rt
      .host
      .as_ref()
      .expect("Missing host interface")
      .borrow_mut();

    // get remaining gas
    // note: this does not factor in the cost of the current instruction
    let gas_left: u32 = ctx
      .rt
      .gas_left()
      .expect("Missing gas information")
      .try_into()
      .expect("gas arithmetic error");

    // note: the host is responsible for checking stack depth, not us

    // marshal inputs
    let address_words = ADDRESS_LENGTH / 4;
    let address = ctx.slice(arg1, address_words);
    let address = AddressWrapper::from(address);

    // read the input length from the next register
    let a2 = Register::X12;
    let len = ctx.rt.register(a2) as usize;

    // check byte alignment
    assert!(len % 4 == 0, "input is not byte-aligned");

    // `len` is denominated in number of bytes; we read words in chunks of four bytes
    // then convert into a standard bytearray.
    let input = if len > 0 {
      let input_slice = ctx.slice(arg2, len / 4);
      Some(
        input_slice
          .iter()
          .flat_map(|&num| num.to_le_bytes().to_vec())
          .collect(),
      )
    } else {
      None
    };

    // read the amount pointer from the next register as little-endian
    let a3 = Register::X13;
    let amount_ptr = ctx.rt.register(a3);
    let amount_slice = ctx.slice(amount_ptr, 2);
    let amount = u64::from(amount_slice[0]) | (u64::from(amount_slice[1]) << 32);

    // note: host is responsible for checking balance and stack depth

    // construct the outbound message
    let msg = AthenaMessage::new(
      MessageKind::Call,
      athena_ctx.depth() + 1,
      gas_left,
      address.into(),
      *athena_ctx.address(),
      input,
      amount,
      Vec::new(),
    );
    let res = host.call(msg);

    // calculate gas spent
    // TODO: should this be a panic or should it just return an out of gas error?
    // for now, it's a panic, since this should not happen.
    let gas_spent = gas_left
      .checked_sub(res.gas_left)
      .expect("host call spent more than available gas");
    ctx.rt.state.clk += gas_spent;

    Some(res.status_code as u32)
  }
}

pub struct SyscallHostGetBalance;

impl SyscallHostGetBalance {
  pub const fn new() -> Self {
    Self
  }
}

impl<T> Syscall<T> for SyscallHostGetBalance
where
  T: HostInterface,
{
  fn execute(&self, ctx: &mut SyscallContext<T>, arg1: u32, _arg2: u32) -> Option<u32> {
    let athena_ctx = ctx
      .rt
      .context
      .as_ref()
      .expect("Missing Athena runtime context");

    // get value from host
    let host = ctx.rt.host.as_mut().expect("Missing host interface");
    let balance = host.borrow_mut().get_balance(athena_ctx.address());
    let balance_high = (balance >> 32) as u32;
    let balance_low = balance as u32;
    let balance_slice = [balance_low, balance_high];

    log::info!("Get balance syscall returning: {}", balance);

    // return to caller
    ctx.mw_slice(arg1, &balance_slice);
    None
  }
}
