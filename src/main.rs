#![no_std]
#![no_main]

// #[macro_use]
// extern crate lazy_static;

mod vga_buffer;

use core::panic::PanicInfo;



#[no_mangle]
pub extern "C" fn _start() -> ! {
    //vga_buffer::_print_something("Hello from VGA! \nHow are you? Wörld!");
    println!("Hello {}{}", "there", "!");
    //set_bg_color(Color::Brown);
    bg!(Brown);
    fg!(White);
    println!("How are you?");
    //fg!();bg!(); // rest to default
    println!("Hi Aarsi!");

    loop {}
}

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {}
}



