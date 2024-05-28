pub struct Memory {
  data: Vec<u8>,
}

impl Memory {
  pub fn new(size: usize) -> Self {
      Memory {
          data: vec![0; size],
      }
  }

  pub fn load(&self, address: usize) -> u8 {
      self.data[address]
  }

  pub fn store(&mut self, address: usize, value: u8) {
      self.data[address] = value;
  }
}
