#[cfg(test)]
mod tests {
  use super::super::vm::VM; // Import the `vm` module correctly.

  #[test]
  fn test_vm_creation() {
    let vm = VM::new(1024);
    assert_eq!(vm.memory.load(0), 0);
  }
}
