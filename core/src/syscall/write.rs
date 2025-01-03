use athena_interface::StatusCode;

use crate::runtime::{Outcome, Register, Syscall, SyscallContext, SyscallResult};

/// Write bytes to selected file descriptor.
/// Supported FDs:
/// - 1: stdout
/// - 2: stderr
/// - 3: public values
/// - 4: input stream (TODO:poszu check why it writes to the input stream)
///
/// FD 1 supports "cycle tracker". TODO(poszu): see if we need this, and add documentation or remove.
/// Note: data written to FD 1 & 2 must be a valid UTF-8 string.
pub(crate) struct SyscallWrite;

impl Syscall for SyscallWrite {
  fn execute(&self, ctx: &mut SyscallContext, fd: u32, write_buf: u32) -> SyscallResult {
    let nbytes = ctx.rt.register(Register::X12);
    let bytes = ctx.bytes(write_buf, nbytes as usize);
    let rt = &mut ctx.rt;
    match fd {
      1 => {
        let s = core::str::from_utf8(&bytes).or(Err(StatusCode::InvalidSyscallArgument))?;
        let flush_s = update_io_buf(ctx, fd, s);
        for line in flush_s {
          println!("stdout: {line}",);
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
        rt.state.public_values_stream.extend(bytes);
      }
      4 => {
        rt.state.input_stream.extend(bytes);
      }
      fd => {
        tracing::debug!(fd, "executing hook");
        match rt.execute_hook(fd, &bytes) {
          Ok(result) => {
            rt.state.input_stream.extend(result);
          }
          Err(err) => {
            tracing::debug!(fd, ?err, "hook failed");
            return Err(StatusCode::InvalidSyscallArgument);
          }
        }
      }
    }
    Ok(Outcome::Result(None))
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

#[cfg(test)]
mod tests {
  use athena_interface::StatusCode;

  use crate::{
    runtime::{hooks, Program, Runtime, Syscall, SyscallContext},
    utils::AthenaCoreOpts,
  };

  use super::SyscallWrite;

  #[test]
  fn invoking_nonexisting_hook_fails() {
    let mut runtime = Runtime::new(
      Program::new(vec![], 0, 0),
      None,
      AthenaCoreOpts::default(),
      None,
    );

    let result = SyscallWrite {}.execute(&mut SyscallContext { rt: &mut runtime }, 7, 0);
    assert_eq!(Err(StatusCode::InvalidSyscallArgument), result);
  }

  #[test]
  fn invoking_registered_hook() {
    let mut hook_mock = hooks::MockHook::new();
    let _ = hook_mock
      .expect_execute()
      .returning(|_, _| Ok(vec![1, 2, 3, 4, 5]));
    let mut runtime = Runtime::new(
      Program::new(vec![], 0, 0),
      None,
      AthenaCoreOpts::default(),
      None,
    );
    runtime.register_hook(7, Box::new(hook_mock)).unwrap();

    let result = SyscallWrite {}.execute(&mut SyscallContext { rt: &mut runtime }, 7, 0);
    result.unwrap();
    assert_eq!(vec![1, 2, 3, 4, 5], runtime.state.input_stream);
  }
}
