#[derive(Debug, Clone, Copy)]
pub struct AthenaCoreOpts {
  max_gas: u32,
}

impl AthenaCoreOpts {
  pub fn new() -> Self {
    Self::default()
  }

  // Method to apply options
  pub fn with_options(mut self, opts: impl IntoIterator<Item = impl FnOnce(&mut Self)>) -> Self {
    for opt in opts {
      opt(&mut self);
    }
    self
  }

  pub fn max_gas(&self) -> u32 {
    self.max_gas
  }
}

impl Default for AthenaCoreOpts {
  fn default() -> Self {
    Self { max_gas: 0 }
  }
}

// Functional option for gas_metering
pub fn with_max_gas(value: u32) -> impl FnOnce(&mut AthenaCoreOpts) {
  move |opts: &mut AthenaCoreOpts| {
    opts.max_gas = value;
  }
}
