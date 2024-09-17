use athena_interface::StatusCode;

use crate::{
  runtime::{Register, Syscall, SyscallContext, SyscallResult},
  utils::num_to_comma_separated,
};

pub struct SyscallWrite;

/// Write bytes to selected file descriptor.
/// Supported FDs:
/// - 1: stdout
/// - 2: stderr
/// - 3: public values
/// - 4: input stream (TODO:poszu check why it writes to the input stream)
///
/// FD 1 supports "cycle tracker". TODO(poszu): see if we need this, and add documentation or remove.
/// Note: data written to FD 1 & 2 must be a valid UTF-8 string.
impl Syscall for SyscallWrite {
  fn execute(
    &self,
    ctx: &mut SyscallContext,
    fd: u32,
    write_buf: u32,
  ) -> Result<SyscallResult, StatusCode> {
    let rt = &mut ctx.rt;
    let nbytes = rt.register(Register::X12);
    // Read nbytes from memory starting at write_buf.
    let bytes = (0..nbytes)
      .map(|i| rt.byte(write_buf + i))
      .collect::<Vec<u8>>();
    match fd {
      1 => {
        let s = core::str::from_utf8(&bytes).or(Err(StatusCode::InvalidSyscallArgument))?;
        if s.contains("cycle-tracker-start:") {
          let fn_name = s
            .split("cycle-tracker-start:")
            .last()
            .ok_or(StatusCode::InvalidSyscallArgument)?
            .trim_end()
            .trim_start();
          let depth = rt.cycle_tracker.len() as u32;
          rt.cycle_tracker
            .insert(fn_name.to_string(), (rt.state.global_clk, depth));
          let padding = (0..depth).map(|_| "│ ").collect::<String>();
          log::debug!("{}┌╴{}", padding, fn_name);
        } else if s.contains("cycle-tracker-end:") {
          let fn_name = s
            .split("cycle-tracker-end:")
            .last()
            .ok_or(StatusCode::InvalidSyscallArgument)?
            .trim_end()
            .trim_start();
          let (start, depth) = rt.cycle_tracker.remove(fn_name).unwrap_or((0, 0));
          // Leftpad by 2 spaces for each depth.
          let padding = (0..depth).map(|_| "│ ").collect::<String>();
          log::info!(
            "{}└╴{} cycles",
            padding,
            num_to_comma_separated(rt.state.global_clk - start as u64)
          );
        } else {
          let flush_s = update_io_buf(ctx, fd, s);
          for line in flush_s {
            println!("stdout: {line}",);
          }
        }
      }
      2 => {
        let s = core::str::from_utf8(&bytes).or(Err(StatusCode::InvalidSyscallArgument))?;
        let flush_s = update_io_buf(ctx, fd, s);
        for line in flush_s {
          println!("stderr: {line}");
        }
      }
      3 => {
        rt.state.public_values_stream.extend_from_slice(&bytes);
      }
      4 => {
        rt.state.input_stream.push(bytes);
      }
      fd => {
        log::debug!("syscall write called with invalid fd: {fd}");
        return Err(StatusCode::InvalidSyscallArgument);
      }
    }
    Ok(SyscallResult::Result(None))
  }
}

fn update_io_buf(ctx: &mut SyscallContext, fd: u32, s: &str) -> Vec<String> {
  let rt = &mut ctx.rt;
  let entry = rt.io_buf.entry(fd).or_default();
  entry.push_str(s);
  if entry.contains('\n') {
    // Return lines except for the last from buf.
    let prev_buf = std::mem::take(entry);
    let mut lines = prev_buf.split('\n').collect::<Vec<&str>>();
    let last = lines.pop().unwrap_or("");
    *entry = last.to_string();
    lines
      .into_iter()
      .map(|line| line.to_string())
      .collect::<Vec<String>>()
  } else {
    vec![]
  }
}
