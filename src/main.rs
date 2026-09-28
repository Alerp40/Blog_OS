#![no_std] //dont link rust std lib
#![no_main] // disable rust entry points

mod vga_buffer;
static HELLO: &[u8] = b"Hello World!";

use core::panic::PanicInfo;

// kernel entry point
#[unsafe(no_mangle)] // dont mangle name of func
pub extern "C" fn _start() -> ! {
    vga_buffer::print_someting();
    // dont mangle name of func
    loop {}
}

// funtion is called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
