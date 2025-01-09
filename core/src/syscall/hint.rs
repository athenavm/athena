use athena_interface::StatusCode;

use crate::runtime::{Outcome, Syscall, SyscallContext, SyscallResult};

/// SyscallHintLen returns the length of the next slice in the hint input stream.
pub(crate) struct SyscallHintLen;

impl Syscall for SyscallHintLen {
  fn execute(&self, ctx: &mut SyscallContext, _: u32, _: u32) -> SyscallResult {
    let len = if ctx.rt.state.input_stream_ptr >= ctx.rt.state.input_stream.len() {
      0
    } else {
      ctx.rt.state.input_stream.len() - ctx.rt.state.input_stream_ptr
    };
    tracing::debug!(
      ptr = ctx.rt.state.input_stream_ptr,
      total = ctx.rt.state.input_stream.len(),
      len,
      "hinted remaning data in the input stream"
    );
    Ok(Outcome::Result(Some(len as u32)))
  }
}

/// SyscallHintRead returns the length of the next slice in the hint input stream.
pub(crate) struct SyscallHintRead;

impl Syscall for SyscallHintRead {
  fn execute(&self, ctx: &mut SyscallContext, ptr: u32, len: u32) -> SyscallResult {
    let data = ctx.rt.state.input_stream[ctx.rt.state.input_stream_ptr..].to_vec();
    if len as usize > data.len() {
      tracing::debug!(
        ptr = ctx.rt.state.input_stream_ptr,
        total = ctx.rt.state.input_stream.len(),
        available = data.len(),
        len,
        "failed reading stdin due to insufficient input data",
      );
      return Err(StatusCode::InsufficientInput);
    }
    let mut data = &data[..len as usize];
    let mut address = ptr;

    // Handle unaligned start
    if address % 4 != 0 {
      let aligned_addr = address & !3; // Round down to aligned address
      let offset = (address % 4) as usize;
      let bytes_to_write = std::cmp::min(4 - offset, data.len());
      tracing::debug!(
        address,
        aligned_addr,
        offset,
        bytes_to_write,
        "hint read address not aligned to 4 bytes"
      );

      let mut word_bytes = ctx.rt.mr(aligned_addr).to_le_bytes();
      tracing::debug!(word = hex::encode(word_bytes), "read existing word");

      word_bytes[offset..offset + bytes_to_write].copy_from_slice(&data[..bytes_to_write]);

      ctx.rt.mw(aligned_addr, u32::from_le_bytes(word_bytes));
      tracing::debug!(word = hex::encode(word_bytes), "written updated word");

      address = aligned_addr + 4;
      data = &data[bytes_to_write..];
    }

    // Iterate through the remaining data in 4-byte chunks
    let mut chunks = data.chunks_exact(4);
    for chunk in &mut chunks {
      // unwrap() won't panic, which is guaranteed by chunks()
      let word = u32::from_le_bytes(chunk.try_into().unwrap());
      ctx.rt.mw(address, word);
      address += 4;
    }
    // In case the vec is not a multiple of 4, right-pad with 0s. This is fine because we
    // are assuming the word is uninitialized, so filling it with 0s makes sense.
    let remainder = chunks.remainder();
    if !remainder.is_empty() {
      let mut word_array = [0u8; 4];
      let len = remainder.len();
      word_array[..len].copy_from_slice(remainder);
      ctx.rt.mw(address, u32::from_le_bytes(word_array));
    }
    tracing::debug!(
      from = ptr,
      to = address as usize + remainder.len(),
      read = len,
      "HintRead syscall finished"
    );
    tracing::trace!(data = hex::encode(data));
    ctx.rt.state.input_stream_ptr += len as usize;
    Ok(Outcome::Result(Some(len)))
  }
}

#[cfg(test)]
mod tests {
  use athena_interface::StatusCode;

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
    ctx.rt.write_stdin_slice(&data[0]);
    ctx.rt.write_stdin_slice(&data[1]);

    let result = syscall.execute(&mut ctx, 0, 0).unwrap();
    assert_eq!(
      Outcome::Result(Some((data[0].len() + data[1].len()) as u32)),
      result
    );
  }

  #[test]
  fn hint_read() {
    let mut rt = Runtime::new(Program::default(), None, AthenaCoreOpts::default(), None);
    let mut ctx = SyscallContext::new(&mut rt);
    let syscall = super::SyscallHintRead {};

    // no inputs
    let result = syscall.execute(&mut ctx, 0, 10);
    assert_eq!(Err(StatusCode::InsufficientInput), result);

    let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    ctx.rt.write_stdin(&data);

    // can't read more than available
    let result = syscall.execute(&mut ctx, 0, data.len() as u32 + 1);
    assert_eq!(Err(StatusCode::InsufficientInput), result);

    // read only up to `len`
    let len = 3;
    let result = syscall.execute(&mut ctx, 0, len as u32);
    assert_eq!(Ok(Outcome::Result(Some(len as u32))), result);
    assert_eq!(&data[..len], ctx.bytes(0, len).as_slice());

    // read the rest
    let address = len;
    let len = data.len() - len;
    let result = syscall.execute(&mut ctx, address as u32, len as u32);
    assert_eq!(Ok(Outcome::Result(Some(len as u32))), result);
    assert_eq!(data, ctx.bytes(0, data.len()).as_slice());
  }
}
