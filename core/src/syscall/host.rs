use std::cmp::min;

use crate::runtime::{Outcome, Register, Syscall, SyscallContext, SyscallResult};
use athena_interface::{
  Address, AthenaMessage, Bytes32Wrapper, MessageKind, StatusCode, BYTES32_LENGTH,
};

pub(crate) struct SyscallHostRead;

impl Syscall for SyscallHostRead {
  fn execute(&self, ctx: &mut SyscallContext, arg1: u32, _: u32) -> SyscallResult {
    let athena_ctx = ctx
      .rt
      .context
      .as_ref()
      .expect("Missing Athena runtime context");

    // marshal inputs
    let key = ctx.slice(arg1, BYTES32_LENGTH / 4);

    // read value from host
    let host = ctx.rt.host.as_mut().expect("Missing host interface");
    let value = host.get_storage(athena_ctx.address(), &Bytes32Wrapper::from(key).into());

    // set return value
    let value_vec: Vec<u32> = Bytes32Wrapper::new(value).into();
    ctx.mw_slice(arg1, value_vec.as_slice());
    Ok(Outcome::Result(None))
  }
}

pub(crate) struct SyscallHostWrite;

impl Syscall for SyscallHostWrite {
  fn execute(&self, ctx: &mut SyscallContext, arg1: u32, arg2: u32) -> SyscallResult {
    let athena_ctx = ctx
      .rt
      .context
      .as_ref()
      .expect("Missing Athena runtime context");

    // marshal inputs
    let key = ctx.slice(arg1, BYTES32_LENGTH / 4);
    let value = ctx.slice(arg2, BYTES32_LENGTH / 4);

    // write value to host
    let host = ctx.rt.host.as_deref_mut().expect("Missing host interface");
    let status_code = host.set_storage(
      athena_ctx.address(),
      &Bytes32Wrapper::from(key).into(),
      &Bytes32Wrapper::from(value).into(),
    );

    Ok(Outcome::Result(Some(status_code as u32)))
  }
}

/// SyscallHostCall performs a host call, calling other programs.
/// Inputs:
///  - a0 (arg1): address to call
///  - a1 (arg2): pointer to payload containing method selector and input to the called program
///  - a2 (x12): length of input (bytes)
///  - a3 (x13): address to write the result to
///  - a4 (x14): length of result buffer (bytes)
///  - a5 (x15): address to read the amount from (2 words, 8 bytes)
pub(crate) struct SyscallHostCall;

impl Syscall for SyscallHostCall {
  fn execute(&self, ctx: &mut SyscallContext, address_ptr: u32, payload_ptr: u32) -> SyscallResult {
    // make sure we have a runtime context
    let athena_ctx = ctx
      .rt
      .context
      .as_ref()
      .expect("Missing Athena runtime context");
    // get remaining gas
    // note: this does not factor in the cost of the current instruction
    let gas_left: u32 = ctx
      .rt
      .gas_left()
      .expect("Missing gas information")
      .try_into()
      .expect("gas arithmetic error");

    // note: the host is responsible for checking stack depth, not us

    let address = Address::from(ctx.array(address_ptr));

    // read the input length from the next register
    let len = ctx.rt.register(Register::X12) as usize;

    // `len` is denominated in number of bytes; we read words in chunks of four bytes
    // then convert into a byte array.
    let input = if len > 0 {
      Some(ctx.bytes(payload_ptr, len))
    } else {
      None
    };

    let amount_ptr = ctx.rt.register(Register::X15);
    let amount = ctx.dword(amount_ptr);

    // note: host is responsible for checking balance and stack depth

    // construct the outbound message
    let msg = AthenaMessage::new(
      MessageKind::Call,
      athena_ctx.depth() + 1,
      gas_left,
      address,
      *athena_ctx.address(),
      input,
      amount,
      Vec::new(),
    );
    let host = ctx.rt.host.as_deref_mut().expect("Missing host interface");
    let res = host.call(msg);

    let output_size = res
      .output
      .map_or(0, |output| self.copy_result(ctx, &output));

    // calculate gas spent
    // TODO: should this be a panic or should it just return an out of gas error?
    // for now, it's a panic, since this should not happen.
    let gas_spent = gas_left
      .checked_sub(res.gas_left)
      .expect("host call spent more than available gas");
    ctx.rt.state.clk += gas_spent;

    match res.status_code {
      StatusCode::Success => Ok(Outcome::Result(Some(output_size))),
      status => {
        tracing::debug!("host system call failed with status code '{status}'");
        Err(status)
      }
    }
  }
}

impl SyscallHostCall {
  fn copy_result(&self, ctx: &mut SyscallContext, output: &[u8]) -> u32 {
    let mut output_ptr = ctx.rt.register(Register::X13);
    if output_ptr == 0 {
      tracing::debug!("no output buffer allocated - not copying the result");
      return 0;
    }
    let output_buf_len = ctx.rt.register(Register::X14);
    if output.len() as u32 > output_buf_len {
      tracing::warn!(
        output_buf_len,
        output_len = output.len(),
        "output buffer too small to hold the entire result - truncating"
      )
    }
    let output_size = min(output.len() as u32, output_buf_len);

    if output_ptr != 0 && output_size != 0 {
      let mut chunks = output[..output_size as usize].chunks_exact(4);
      for c in chunks.by_ref() {
        let v = u32::from_le_bytes(c.try_into().unwrap());
        ctx.rt.mw(output_ptr, v);
        output_ptr += 4;
      }

      if !chunks.remainder().is_empty() {
        let mut value = [0u8; 4];
        value.copy_from_slice(chunks.remainder());
        ctx.rt.mw(output_ptr, u32::from_le_bytes(value));
      }
    };

    output_size
  }
}

pub(crate) struct SyscallHostGetBalance;

impl Syscall for SyscallHostGetBalance {
  fn execute(&self, ctx: &mut SyscallContext, arg1: u32, _: u32) -> SyscallResult {
    let athena_ctx = ctx
      .rt
      .context
      .as_ref()
      .expect("Missing Athena runtime context");

    // get value from host
    let host = ctx.rt.host.as_deref_mut().expect("Missing host interface");
    let balance = host.get_balance(athena_ctx.address());
    let balance_high = (balance >> 32) as u32;
    let balance_low = balance as u32;
    let balance_slice = [balance_low, balance_high];

    tracing::debug!("get balance syscall returning: {}", balance);

    // return to caller
    ctx.mw_slice(arg1, &balance_slice);
    Ok(Outcome::Result(None))
  }
}

pub(crate) struct SyscallHostSpawn;

impl Syscall for SyscallHostSpawn {
  fn execute(&self, ctx: &mut SyscallContext, address: u32, len: u32) -> SyscallResult {
    let blob = ctx.bytes(address, len as usize);

    let host = ctx.rt.host.as_deref_mut().expect("Missing host interface");
    let address = host.spawn(blob);

    let out_addr = ctx
      .rt
      .rr(Register::X12, crate::runtime::MemoryAccessPosition::A);

    for (idx, c) in address.as_ref().chunks_exact(4).enumerate() {
      let v = u32::from_le_bytes(c.try_into().unwrap());
      ctx.rt.mw(out_addr + idx as u32 * 4, v);
    }

    Ok(Outcome::Result(None))
  }
}

/// System call to deploy a new program
pub struct SyscallHostDeploy;

impl Syscall for SyscallHostDeploy {
  fn execute(&self, ctx: &mut SyscallContext, address: u32, len: u32) -> SyscallResult {
    let blob = ctx.bytes(address, len as usize);

    let host = ctx.rt.host.as_deref_mut().expect("Missing host interface");
    let address = match host.deploy(blob) {
      Ok(addr) => addr,
      Err(err) => {
        tracing::debug!("deploy failed: {err}");
        return Err(StatusCode::Failure);
      }
    };
    tracing::debug!(%address, "deploy succeeded");

    let out_addr = ctx
      .rt
      .rr(Register::X12, crate::runtime::MemoryAccessPosition::A);

    for (idx, c) in address.as_ref().chunks_exact(4).enumerate() {
      let v = u32::from_le_bytes(c.try_into().unwrap());
      ctx.rt.mw(out_addr + idx as u32 * 4, v);
    }

    Ok(Outcome::Result(None))
  }
}

#[cfg(test)]
mod tests {
  use athena_interface::{Address, AthenaContext, ExecutionResult, MockHostInterface};

  use crate::{
    runtime::{self, Program},
    utils::{with_max_gas, AthenaCoreOpts},
  };

  use super::*;

  #[test]
  fn call_increments_depth() {
    let mut host = MockHostInterface::new();
    host
      .expect_call()
      .withf(|m| m.depth == 1)
      .returning(|_| ExecutionResult::new(StatusCode::Success, 0, None));

    let context = AthenaContext::new(Address::default(), Address::default(), 0);
    let mut runtime = runtime::Runtime::new(
      Program::new(vec![], 0, 0),
      Some(&mut host),
      AthenaCoreOpts::default().with_options(vec![with_max_gas(10)]),
      Some(context),
    );

    let result = SyscallHostCall {}.execute(&mut SyscallContext { rt: &mut runtime }, 0, 0);
    result.unwrap();
  }
}
