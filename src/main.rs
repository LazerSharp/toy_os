#![no_std]
#![no_main]

// Custom test framework

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

//////

mod vga_buffer;
use core::panic::PanicInfo;

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Runing tests {}", tests.len());
    for test in tests {
        test()
    }
}

#[test_case]
fn test1() {
    println!("Execucuting test1");
    assert_eq!(2, 1 + 1);
    cprintln!(Green, "[OK]");
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

#[panic_handler]
fn panic(panic: &PanicInfo) -> ! {
    //bg!(Yellow);
    fg!(Red);
    println!("Panic! msg: {}", panic);
    loop {}
}



