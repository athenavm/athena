#[cfg(test)]
mod tests {
  use athena::vm::{VM, interpreter::Instruction}; // Import the `vm` module correctly.

  #[test]
  fn test_vm_creation() {
    let vm = VM::new(1024);
    assert_eq!(vm.memory.load(0), 0);
  }

  #[test]
  fn test_add_instruction() {
      let mut vm = VM::new(1024);
      vm.registers[1] = 2;
      vm.registers[2] = 3;
      vm.execute_instruction(Instruction::Add { rd: 0, rs1: 1, rs2: 2 });
      assert_eq!(vm.registers[0], 5);
  }

  // Add more tests for other instructions and operations
}
