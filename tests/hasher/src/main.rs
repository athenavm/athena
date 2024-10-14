#![cfg_attr(target_os = "zkvm", no_main)]

#[cfg(target_os = "zkvm")]
athena_vm::entrypoint!(main);

fn main() {
  {
    let mut buf = [0; 64];

    for (i, x) in buf.iter_mut().enumerate() {
      *x = i as u8;
    }
    println!("{buf:?}");
  }
  let buf = [0; 64];
  println!("{buf:?}");
}
