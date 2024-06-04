//! A simple program to be run inside the Athena VM.

#![no_main]
athena_vm::entrypoint!(main);

pub fn main() {
    // NOTE: values of n larger than 186 will overflow the u128 type,
    // resulting in output that doesn't match fibonacci sequence.
    let n = athena_vm::io::read::<u32>();
    let mut a: u128 = 0;
    let mut b: u128 = 1;
    let mut sum: u128;
    for _ in 1..n {
        sum = a + b;
        a = b;
        b = sum;
    }

    athena_vm::io::write(&a);
    athena_vm::io::write(&b);
}
