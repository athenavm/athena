// #![no_std]
#![no_main]
athena_vm::entrypoint!(main);

// #[no_mangle]
// pub extern "C" fn _start() -> ! {
//     let a: u32 = 5;
//     let b: u32 = 3;
//     let sum = add(a, b);

//     // We need a way to output `sum` for verification
//     // Assuming a function `output` exists to demonstrate
//     // output(sum);
// }

pub fn main() {
    let a: u32 = 5;
    let b: u32 = 3;
    let _sum = add(a, b);
    println!("Hello, world! a + b is {}", _sum);

    // We need a way to output `sum` for verification
    // Assuming a function `output` exists to demonstrate
    // output(sum);
}

fn add(x: u32, y: u32) -> u32 {
    x + y
}

// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
//     unsafe {
//       core::arch::asm!("unimp", options(noreturn));
//     }
// }
