#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(toy_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use toy_os::{allocator::init_heap, memory};
use x86_64::VirtAddr;

entry_point!(kernel_main);

#[no_mangle]
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    test_main();
    toy_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    toy_os::test_panic_handler(info)
}

#[test_case]
fn test_box() {
    let _v = Box::new(123);
}

#[test_case]
fn test_vector() {
    let n = 1000;
    let mut vec = vec![];
    for i in 0..n {
        vec.push(i);
    }
    let sum = n * (n - 1) / 2;
    assert_eq!(sum, vec.iter().sum());
}

#[test_case]
fn test_rc() {
    let rc = Rc::new(5);
    let rc_clone = rc.clone();
    assert_eq!(2, Rc::strong_count(&rc));
    drop(rc);
    assert_eq!(1, Rc::strong_count(&rc_clone));
}
