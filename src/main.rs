#![no_std] //dont link rust std lib
#![no_main] // disable rust entry points

mod vga_buffer;
static HELLO: &[u8] = b"Hello World!";

use core::panic::PanicInfo;

// kernel entry point
#[unsafe(no_mangle)] // dont mangle name of func
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!"); //no need to import macro as it lives in the root namespace
    panic!();

    loop {}
}

// funtion is called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
