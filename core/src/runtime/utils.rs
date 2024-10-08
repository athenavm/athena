use std::io::Write;

use super::{Instruction, Runtime};
use crate::runtime::Register;

pub const fn align(addr: u32) -> u32 {
  addr - addr % 4
}

impl<'h> Runtime<'h> {
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
        x0 = self.register(Register::X0),
        x1 = self.register(Register::X1),
        x2 = self.register(Register::X2),
        x3 = self.register(Register::X3),
        x4 = self.register(Register::X4),
        x5 = self.register(Register::X5),
        x6 = self.register(Register::X6),
        x7 = self.register(Register::X7),
        x8 = self.register(Register::X8),
        x9 = self.register(Register::X9),
        x10 = self.register(Register::X10),
        x11 = self.register(Register::X11),
        x12 = self.register(Register::X12),
        x13 = self.register(Register::X13),
        x14 = self.register(Register::X14),
        x15 = self.register(Register::X15),
        x16 = self.register(Register::X16),
        x17 = self.register(Register::X17),
        x18 = self.register(Register::X18),
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
