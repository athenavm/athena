#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let a: u32 = 5;
    let b: u32 = 3;
    let sum = add(a, b);

    // We need a way to output `sum` for verification
    // Assuming a function `output` exists to demonstrate
    // output(sum);

    loop {}
}

fn add(x: u32, y: u32) -> u32 {
    x + y
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
