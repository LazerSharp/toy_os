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
    println!("Hello {}{}", "there", "!");

    toy_os::init();

    // x86_64::instructions::interrupts::int3();

    // println!("I am alive!");

    // trigger a page fault
    // unsafe {
    //     *(0xdeadbeef as *mut u64) = 42;
    // };

    // #[allow(unconditional_recursion)]
    // fn stack_overflow() {
    //     stack_overflow(); // for each recursion, the return address is pushed
    // }

    // trigger a stack overflow
    //stack_overflow();

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

    // loop {
    //     for _ in 1..1000 {}
    //     cprint!(LightBlue, "-");
    // }
    toy_os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(panic: &PanicInfo) -> ! {
    use toy_os::cprintln;
    cprintln!(Red, "Panic! msg: {}", panic);
    toy_os::hlt_loop();
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
