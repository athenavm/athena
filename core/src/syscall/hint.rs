use athena_interface::StatusCode;

use crate::runtime::{Outcome, Syscall, SyscallContext, SyscallResult};

/// SyscallHintLen returns the length of the next slice in the hint input stream.
pub(crate) struct SyscallHintLen;

impl Syscall for SyscallHintLen {
  fn execute(&self, ctx: &mut SyscallContext, _: u32, _: u32) -> SyscallResult {
    if ctx.rt.state.input_stream_ptr >= ctx.rt.state.input_stream.len() {
      tracing::debug!(
        "no more data to read in stdin: input_stream_ptr={}, input_stream_len={}",
        ctx.rt.state.input_stream_ptr,
        ctx.rt.state.input_stream.len(),
      );
      return Ok(Outcome::Result(Some(0)));
    }
    Ok(Outcome::Result(Some(
      ctx.rt.state.input_stream[ctx.rt.state.input_stream_ptr].len() as u32,
    )))
  }
}

/// SyscallHintRead returns the length of the next slice in the hint input stream.
pub(crate) struct SyscallHintRead;

impl Syscall for SyscallHintRead {
  fn execute(&self, ctx: &mut SyscallContext, ptr: u32, len: u32) -> SyscallResult {
    if ctx.rt.state.input_stream_ptr >= ctx.rt.state.input_stream.len() {
      tracing::debug!(
             "failed reading stdin due to insufficient input data: input_stream_ptr={}, input_stream_len={}",
              ctx.rt.state.input_stream_ptr,
              ctx.rt.state.input_stream.len()
      );
      return Err(StatusCode::InsufficientInput);
    }
    let vec = &ctx.rt.state.input_stream[ctx.rt.state.input_stream_ptr];
    ctx.rt.state.input_stream_ptr += 1;
    assert!(
      !ctx.rt.unconstrained,
      "hint read should not be used in a unconstrained block"
    );
    if vec.len() != len as usize {
      tracing::debug!(
        "hint input stream read length mismatch: expected={}, actual={}",
        len,
        vec.len()
      );
      return Err(StatusCode::InvalidSyscallArgument);
    }
    if ptr % 4 != 0 {
      tracing::debug!("hint read address not aligned to 4 bytes");
      return Err(StatusCode::InvalidSyscallArgument);
    }
    // Iterate through the vec in 4-byte chunks
    for i in (0..len).step_by(4) {
      // Get each byte in the chunk
      let b1 = vec[i as usize];
      // In case the vec is not a multiple of 4, right-pad with 0s. This is fine because we
      // are assuming the word is uninitialized, so filling it with 0s makes sense.
      let b2 = vec.get(i as usize + 1).copied().unwrap_or(0);
      let b3 = vec.get(i as usize + 2).copied().unwrap_or(0);
      let b4 = vec.get(i as usize + 3).copied().unwrap_or(0);
      let word = u32::from_le_bytes([b1, b2, b3, b4]);

      // Save the data into runtime state so the runtime will use the desired data instead of
      // 0 when first reading/writing from this address.
      ctx
        .rt
        .state
        .uninitialized_memory
        .entry(ptr + i)
        .and_modify(|_| panic!("hint read address is initialized already"))
        .or_insert(word);
    }
    Ok(Outcome::Result(None))
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    runtime::{Outcome, Program, Runtime, Syscall, SyscallContext},
    utils::AthenaCoreOpts,
  };

  #[test]
  fn hint_len_syscall() {
    let mut rt = Runtime::new(Program::default(), None, AthenaCoreOpts::default(), None);
    let mut ctx = SyscallContext::new(&mut rt);
    let syscall = super::SyscallHintLen {};

    // no inputs
    let result = syscall.execute(&mut ctx, 0, 0).unwrap();
    assert_eq!(Outcome::Result(Some(0)), result);

    // with inputs
    let data = [vec![1, 2, 3, 4, 5], vec![6, 7]];
    ctx.rt.write_vecs(&data);

    let result = syscall.execute(&mut ctx, 0, 0).unwrap();
    assert_eq!(Outcome::Result(Some(data[0].len() as u32)), result);
  }
}
