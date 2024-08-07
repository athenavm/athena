//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the VM.
#![no_main]
athena_vm::entrypoint!(main);

pub fn main() {
  let n = 10;
  // Compute the n'th fibonacci number, using normal Rust code.
  let mut a = 0;
  let mut b = 1;
  for _ in 0..n {
    let mut c = a + b;
    c %= 7919; // Modulus to prevent overflow.
    a = b;
    b = c;
  }
  athena_vm::io::write(&a);
  athena_vm::io::write(&b);
}
