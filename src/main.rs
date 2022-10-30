#![no_std]
#![no_main]
// Custom test framework
#![feature(custom_test_frameworks)]
#![test_runner(toy_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use toy_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    toy_os::init();

    println!("Hello {}{}", "there", "!");

    x86_64::instructions::interrupts::int3();

    //vga_buffer::_print_something("Hello from VGA! \nHow are you? Wörld!");

    //set_bg_color(Color::Brown);
    // bg!(Brown);
    // fg!(White);
    // println!("How are you?");
    // //fg!();bg!(); // rest to default
    // println!("Hi Aarsi!");
    #[cfg(test)]
    test_main();

    println!("I am still alive!");

    loop {
        //panic!("I am worried :(");
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(panic: &PanicInfo) -> ! {
    use toy_os::{bg, fg};
    bg!(Yellow);
    fg!(Red);
    println!("Panic! msg: {}", panic);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(panic: &PanicInfo) -> ! {
    toy_os::test_panic_handler(panic);
}

#[test_case]
fn test1() {
    assert_eq!(2, 1 + 1);
}

#[test_case]
fn test2() {
    assert_eq!(3, 2 + 1);
}
