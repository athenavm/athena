use std::io::Write;

use super::{Instruction, Runtime};

impl Runtime<'_> {
  #[inline]
  pub fn log(&mut self, instruction: &Instruction) {
    // Write the current program counter to the trace buffer for the cycle tracer.
    if let Some(ref mut buf) = self.trace_buf {
      if !self.unconstrained {
        buf.write_all(&u32::to_be_bytes(self.state.pc)).unwrap();
      }
    }

    tracing::trace!(
        clk = self.state.global_clk,
        pc = format_args!("0x{:x}", self.state.pc),
        instruction = ?instruction,
        registers = ?self.state.regs,
    );

    if !self.unconstrained && self.state.global_clk % 10_000_000 == 0 {
      tracing::trace!(
        clk = self.state.clk,
        global_clk = self.state.global_clk,
        pc = self.state.pc,
      );
    }
  }
}
