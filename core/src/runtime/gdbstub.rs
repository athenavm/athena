use athena_interface::StatusCode;
use gdbstub::{
  arch::{Arch, RegId, Registers},
  common::Signal,
  conn::{Connection, ConnectionExt},
  stub::{run_blocking, DisconnectReason, GdbStub, SingleThreadStopReason},
  target::{
    ext::{
      base::{
        single_register_access::SingleRegisterAccess,
        singlethread::{
          SingleThreadBase, SingleThreadResume, SingleThreadResumeOps, SingleThreadSingleStep,
        },
        BaseOps,
      },
      breakpoints::{Breakpoints, BreakpointsOps, SwBreakpoint},
    },
    Target,
  },
};

use super::{Event, ExecutionError, Register, Runtime};

impl Target for Runtime<'_> {
  type Arch = Riscv32e;

  type Error = String;

  #[inline(always)]
  fn base_ops(&mut self) -> gdbstub::target::ext::base::BaseOps<'_, Self::Arch, Self::Error> {
    BaseOps::SingleThread(self)
  }

  #[inline(always)]
  fn support_breakpoints(&mut self) -> Option<BreakpointsOps<Self>> {
    Some(self)
  }
}

impl Breakpoints for Runtime<'_> {
  fn support_sw_breakpoint(
    &mut self,
  ) -> Option<gdbstub::target::ext::breakpoints::SwBreakpointOps<'_, Self>> {
    Some(self)
  }
}

impl SingleThreadSingleStep for Runtime<'_> {
  fn step(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
    tracing::trace!("single stepping");
    let _ = self.execute_cycle();
    Ok(())
  }
}

impl SingleThreadResume for Runtime<'_> {
  fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
    tracing::trace!("resuming");
    Ok(())
  }

  fn support_single_step(
    &mut self,
  ) -> Option<gdbstub::target::ext::base::singlethread::SingleThreadSingleStepOps<'_, Self>> {
    Some(self)
  }
}

impl SwBreakpoint for Runtime<'_> {
  fn add_sw_breakpoint(
    &mut self,
    addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
    _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind,
  ) -> gdbstub::target::TargetResult<bool, Self> {
    tracing::trace!("adding sw breakpoint at {:#x}", addr);
    self.breakpoints.insert(addr as u32);
    Ok(true)
  }

  fn remove_sw_breakpoint(
    &mut self,
    addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
    _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind,
  ) -> gdbstub::target::TargetResult<bool, Self> {
    tracing::trace!("removing sw breakpoint at {:#x}", addr);
    self.breakpoints.remove(&(addr as u32));
    Ok(true)
  }
}

impl SingleThreadBase for Runtime<'_> {
  fn read_registers(
    &mut self,
    regs: &mut <Self::Arch as gdbstub::arch::Arch>::Registers,
  ) -> gdbstub::target::TargetResult<(), Self> {
    tracing::trace!("reading registers");
    regs.x.copy_from_slice(&self.registers()[..16]);
    regs.pc = self.state.pc;
    Ok(())
  }

  fn write_registers(
    &mut self,
    regs: &<Self::Arch as gdbstub::arch::Arch>::Registers,
  ) -> gdbstub::target::TargetResult<(), Self> {
    tracing::trace!("writing registers: {regs:?}");
    for (reg, value) in regs.x.iter().enumerate() {
      self.rw(Register::from_u32(reg as u32), *value)
    }

    Ok(())
  }

  fn read_addrs(
    &mut self,
    start_addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
    mut data: &mut [u8],
  ) -> gdbstub::target::TargetResult<usize, Self> {
    tracing::trace!(
      "reading memory {start_addr:#x} - {:#x}",
      start_addr as usize + data.len()
    );
    if start_addr == 0 {
      return Err(gdbstub::target::TargetError::NonFatal);
    }
    let mut n = 0;
    //FIXME: implement optimally by reading u32 words
    // for addr in start_addr..(start_addr + data.len() as u32 / 4) {
    //   let value = self.word(addr * 4);
    //   data[..4].copy_from_slice(&value.to_le_bytes());
    //   n += 4;
    //   data = data[4..].as_mut();
    // }

    while !data.is_empty() {
      data[0] = self.byte(start_addr + n as u32);
      n += 1;
      data = data[1..].as_mut();
    }

    Ok(n)
  }

  fn write_addrs(
    &mut self,
    start_addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
    data: &[u8],
  ) -> gdbstub::target::TargetResult<(), Self> {
    tracing::trace!(
      "writing memory {start_addr:#x} - {:#x}",
      start_addr + data.len() as u32
    );
    assert!(data.len() % 4 == 0);
    assert!(start_addr % 4 == 0);
    for (offset, value) in data.chunks(4).enumerate() {
      let addr = start_addr + offset as u32 * 4;
      let value = u32::from_le_bytes(value.try_into().unwrap());
      self.mw(addr, value);
    }
    Ok(())
  }

  fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
    Some(self)
  }

  fn support_single_register_access(
    &mut self,
  ) -> Option<
    gdbstub::target::ext::base::single_register_access::SingleRegisterAccessOps<'_, (), Self>,
  > {
    Some(self)
  }
}

impl SingleRegisterAccess<()> for Runtime<'_> {
  fn read_register(
    &mut self,
    _tid: (),
    reg_id: <Self::Arch as Arch>::RegId,
    buf: &mut [u8],
  ) -> gdbstub::target::TargetResult<usize, Self> {
    match reg_id {
      RiscvRegId::Gpr(id) => {
        let value = self.registers()[id as usize];
        buf.copy_from_slice(&value.to_le_bytes());
        Ok(4)
      }
      RiscvRegId::Pc => {
        buf.copy_from_slice(&self.state.pc.to_le_bytes());
        Ok(4)
      }
    }
  }

  fn write_register(
    &mut self,
    _tid: (),
    reg_id: <Self::Arch as Arch>::RegId,
    val: &[u8],
  ) -> gdbstub::target::TargetResult<(), Self> {
    match reg_id {
      RiscvRegId::Gpr(id) => {
        let value = u32::from_le_bytes(val.try_into().unwrap());
        self.rw(Register::from_u32(id as u32), value);
      }
      RiscvRegId::Pc => {
        let value = u32::from_le_bytes(val.try_into().unwrap());
        self.state.pc = value;
      }
    }
    Ok(())
  }
}

enum RunEvent {
  IncomingData,
  Event(Event),
  ExecutionError(ExecutionError),
}

struct GdbBlockingEventLoop<'h> {
  _h: std::marker::PhantomData<Runtime<'h>>,
}

impl GdbBlockingEventLoop<'_> {
  fn execute_with_poll(
    runtime: &mut Runtime,
    mut poll_incoming_data: impl FnMut() -> bool,
  ) -> RunEvent {
    let mut cycles = 0;
    loop {
      if cycles % 2 == 0 && poll_incoming_data() {
        break RunEvent::IncomingData;
      }
      cycles += 1;

      match runtime.execute_cycle() {
        Ok(Some(event)) => return RunEvent::Event(event),
        Ok(None) => continue,
        Err(err) => return RunEvent::ExecutionError(err),
      };
    }
  }
}

// The `run_blocking::BlockingEventLoop` groups together various callbacks
// the `GdbStub::run_blocking` event loop requires you to implement.
impl<'h> run_blocking::BlockingEventLoop for GdbBlockingEventLoop<'h> {
  type Target = Runtime<'h>;
  type Connection = Box<dyn ConnectionExt<Error = std::io::Error>>;

  // or MultiThreadStopReason on multi threaded targets
  type StopReason = SingleThreadStopReason<u32>;

  // Invoked immediately after the target's `resume` method has been
  // called. The implementation should block until either the target
  // reports a stop reason, or if new data was sent over the connection.
  fn wait_for_stop_reason(
    target: &mut Runtime<'h>,
    conn: &mut Self::Connection,
  ) -> Result<
    run_blocking::Event<SingleThreadStopReason<u32>>,
    run_blocking::WaitForStopReasonError<
      <Self::Target as Target>::Error,
      <Self::Connection as Connection>::Error,
    >,
  > {
    let poll_incoming_data = || conn.peek().map(|b| b.is_some()).unwrap_or(true);

    match Self::execute_with_poll(target, poll_incoming_data) {
      RunEvent::IncomingData => {
        let byte = conn
          .read()
          .map_err(run_blocking::WaitForStopReasonError::Connection)?;
        Ok(run_blocking::Event::IncomingData(byte))
      }
      RunEvent::Event(event) => {
        tracing::trace!("received event {event:?}");

        let stop_reason = match event {
          Event::DoneStep => SingleThreadStopReason::DoneStep,
          Event::Halted => SingleThreadStopReason::Exited(0),
          Event::Break => SingleThreadStopReason::SwBreak(()),
        };

        Ok(run_blocking::Event::TargetStopped(stop_reason))
      }
      RunEvent::ExecutionError(err) => {
        tracing::debug!("received execution error {err:?}");

        let stop_reason = match err {
          ExecutionError::SyscallFailed(code) => SingleThreadStopReason::Exited(code as u8),
          ExecutionError::OutOfGas() => SingleThreadStopReason::Exited(StatusCode::OutOfGas as u8),
          ExecutionError::HaltWithNonZeroExitCode(code) => {
            SingleThreadStopReason::Exited(code as u8)
          }
          ExecutionError::InvalidMemoryAccess(_, _) => {
            SingleThreadStopReason::Terminated(Signal::EXC_BAD_ACCESS)
          }
          ExecutionError::UnsupportedSyscall(_) => {
            SingleThreadStopReason::Terminated(Signal::SIGSYS)
          }
          ExecutionError::Breakpoint() => SingleThreadStopReason::SwBreak(()),
          ExecutionError::Unimplemented() => SingleThreadStopReason::Terminated(Signal::SIGILL),
          ExecutionError::UnknownSymbol() => SingleThreadStopReason::Terminated(Signal::SIGABRT),
        };
        Ok(run_blocking::Event::TargetStopped(stop_reason))
      }
    }
  }

  // Invoked when the GDB client sends a Ctrl-C interrupt.
  fn on_interrupt(
    _target: &mut Runtime<'h>,
  ) -> Result<Option<SingleThreadStopReason<u32>>, <Runtime<'h> as Target>::Error> {
    // Because this emulator runs as part of the GDB stub loop, there isn't any
    // special action that needs to be taken to interrupt the underlying target. It
    // is implicitly paused whenever the stub isn't within the
    // `wait_for_stop_reason` callback.
    Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT)))
  }
}

pub fn gdb_event_loop_thread<'h>(
  debugger: GdbStub<Runtime<'h>, Box<dyn ConnectionExt<Error = std::io::Error>>>,
  runtime: &mut Runtime<'h>,
) -> Result<Option<u32>, ExecutionError> {
  match debugger.run_blocking::<GdbBlockingEventLoop<'h>>(runtime) {
    Ok(disconnect_reason) => match disconnect_reason {
      DisconnectReason::Disconnect => {
        tracing::info!("GDB client disconnected. Running to completion...");
        loop {
          match runtime.execute_cycle() {
            Ok(Some(crate::runtime::Event::Halted)) => break,
            Err(err) => return Err(err),
            _ => continue,
          }
        }
      }
      DisconnectReason::TargetExited(code) => {
        tracing::info!("Target exited with code {}!", code);
        if code != 0 {
          return Err(ExecutionError::HaltWithNonZeroExitCode(code as u32));
        }
      }
      DisconnectReason::TargetTerminated(sig) => {
        tracing::info!("Target terminated with signal {}!", sig);
        match sig {
          Signal::SIGILL => return Err(ExecutionError::Unimplemented()),
          Signal::EXC_BAD_ACCESS => {
            return Err(ExecutionError::InvalidMemoryAccess(
              crate::runtime::Opcode::UNIMP,
              0,
            ))
          }
          Signal::SIGSYS => return Err(ExecutionError::UnsupportedSyscall(0)),
          _ => panic!("Unexpected signal: {sig}"),
        }
      }
      DisconnectReason::Kill => panic!("GDB sent a kill command!"),
    },
    Err(e) => {
      panic!("Error: {e}");
    }
  }
  runtime.postprocess();

  // Calculate remaining gas. If we spent too much gas, an error would already have been thrown and
  // we would never reach this code, hence the assertion.
  Ok(
    runtime
      .gas_left()
      .map(|gas_left| u32::try_from(gas_left).expect("Gas conversion error")),
  )
}

pub fn run_under_gdb(
  runtime: &mut Runtime,
  listener: std::net::TcpListener,
  symbol: Option<&str>,
) -> Result<Option<u32>, ExecutionError> {
  if let Some(symbol) = symbol {
    runtime.jump_to_symbol(symbol)?;
  }
  let (stream, _) = listener.accept().unwrap();
  let conn = Box::new(stream);
  let gdb = GdbStub::new(conn as _);
  gdb_event_loop_thread(gdb, runtime)
}

pub enum Riscv32e {}

impl Arch for Riscv32e {
  type Usize = u32;
  type Registers = RiscvCoreRegs;
  type BreakpointKind = usize;
  type RegId = RiscvRegId;

  fn target_description_xml() -> Option<&'static str> {
    Some(include_str!("rv32e.xml"))
  }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RiscvCoreRegs {
  /// General purpose registers (x0-x15)
  pub x: [u32; 16],
  /// Program counter
  pub pc: u32,
}

impl Registers for RiscvCoreRegs {
  type ProgramCounter = u32;

  fn pc(&self) -> Self::ProgramCounter {
    self.pc
  }

  fn gdb_serialize(&self, mut write_byte: impl FnMut(Option<u8>)) {
    for reg in self.x {
      for byte in reg.to_le_bytes() {
        write_byte(Some(byte));
      }
    }
    for byte in self.pc.to_le_bytes() {
      write_byte(Some(byte));
    }
  }

  fn gdb_deserialize(&mut self, bytes: &[u8]) -> Result<(), ()> {
    if bytes.len() % 4 != 0 {
      return Err(());
    }

    let mut regs = bytes
      .chunks_exact(4)
      .map(|c| u32::from_le_bytes(c.try_into().unwrap()));

    for reg in self.x.iter_mut() {
      *reg = regs.next().ok_or(())?
    }
    self.pc = regs.next().ok_or(())?;

    if regs.next().is_some() {
      return Err(());
    }

    Ok(())
  }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum RiscvRegId {
  /// General Purpose Register (x0-x15).
  Gpr(u8),
  /// Program Counter.
  Pc,
}

impl RegId for RiscvRegId {
  fn from_raw_id(id: usize) -> Option<(Self, Option<std::num::NonZeroUsize>)> {
    match id {
      0..=15 => Some((RiscvRegId::Gpr(id as u8), None)),
      16 => Some((RiscvRegId::Pc, None)),
      _ => None,
    }
  }
}
