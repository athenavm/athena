use super::Base;

/// A register stores a 32-bit value used by operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
  X0 = 0,
  X1 = 1,
  X2 = 2,
  X3 = 3,
  X4 = 4,
  X5 = 5,
  X6 = 6,
  X7 = 7,
  X8 = 8,
  X9 = 9,
  X10 = 10,
  X11 = 11,
  X12 = 12,
  X13 = 13,
  X14 = 14,
  X15 = 15,
}
impl Register {
  // FIXME: remove all uses
  pub(crate) fn from_u32(value: u32) -> Self {
    Self::try_from(value).unwrap()
  }
}

impl TryFrom<u32> for Register {
  type Error = &'static str;
  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(Register::X0),
      1 => Ok(Register::X1),
      2 => Ok(Register::X2),
      3 => Ok(Register::X3),
      4 => Ok(Register::X4),
      5 => Ok(Register::X5),
      6 => Ok(Register::X6),
      7 => Ok(Register::X7),
      8 => Ok(Register::X8),
      9 => Ok(Register::X9),
      10 => Ok(Register::X10),
      11 => Ok(Register::X11),
      12 => Ok(Register::X12),
      13 => Ok(Register::X13),
      14 => Ok(Register::X14),
      15 => Ok(Register::X15),
      _ => Err("register out of bounds"),
    }
  }
}

#[derive(Debug, Clone)]
pub(crate) struct Registers {
  register_space: Vec<u32>,
}

impl Registers {
  pub(crate) fn new(base: Base) -> Self {
    let reg_count = match base {
      Base::RV32E => 16,
    };
    Self {
      register_space: (0..reg_count).map(|_| 0).collect(),
    }
  }

  pub(crate) fn write(&mut self, reg: Register, value: u32) {
    if reg == Register::X0 {
      return;
    }
    self.register_space[reg as usize] = value;
  }

  pub(crate) fn read(&self, reg: Register) -> u32 {
    self.register_space[reg as usize]
  }

  pub(crate) fn all(&self) -> &[u32] {
    &self.register_space
  }
}

#[cfg(test)]
mod tests {
  use crate::runtime::{Base, Register, Registers};

  #[test]
  fn cannot_overwrite_x0() {
    let mut regs = Registers::new(Base::RV32E);
    assert_eq!(0, regs.read(Register::X0));

    regs.write(Register::X0, 1234);
    assert_eq!(0, regs.read(Register::X0));
  }

  #[test]
  fn regs_start_zeroed() {
    let regs = Registers::new(Base::RV32E);
    assert_eq!([0; 16].as_slice(), regs.all());
  }

  #[test]
  fn write_read() {
    let mut regs = Registers::new(Base::RV32E);
    for reg in 1..16 {
      let value = reg + 100;
      let reg = Register::try_from(reg).unwrap();
      regs.write(reg, value);
      assert_eq!(value, regs.read(reg));
    }
  }
}
