#![no_std]
#![no_main]
// Custom test framework
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

//////

mod serial;
mod vga_buffer;

use core::panic::PanicInfo;

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    sprintln!("Runing {} tests", tests.len());
    for test in tests {
        test()
    }
    exit_qemu(QemuExitCode::Success);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    //vga_buffer::_print_something("Hello from VGA! \nHow are you? Wörld!");
    println!("Hello {}{}", "there", "!");
    //set_bg_color(Color::Brown);
    // bg!(Brown);
    // fg!(White);
    // println!("How are you?");
    // //fg!();bg!(); // rest to default
    // println!("Hi Aarsi!");
    #[cfg(test)]
    test_main();

    loop {
        panic!("I am worried :(");
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(panic: &PanicInfo) -> ! {
    //bg!(Yellow);
    fg!(Red);
    println!("Panic! msg: {}", panic);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(panic: &PanicInfo) -> ! {
    sprintln!("[FAILED] \n error: {}", panic);
    exit_qemu(QemuExitCode::Failure);
    loop {}
}

#[test_case]
fn test1() {
    sprint!("Execucuting test1... ");
    assert_eq!(2, 1 + 1);
    sprintln!("[OK]");
}

#[test_case]
fn test2() {
    sprint!("Execucuting test2... ");
    assert_eq!(3, 2 + 1);
    sprintln!("[OK]");
}
