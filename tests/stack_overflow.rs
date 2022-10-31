#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use toy_os::{exit_qemu, QemuExitCode};
use toy_os::{sprint, sprintln};
use x86_64::structures::idt::InterruptStackFrame;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    sprint!("stack_overflow::stack_overflow...\t");

    toy_os::gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();
    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    toy_os::test_panic_handler(info)
}

use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(toy_os::gdt::DOUBLE_FAULT_STACK_TABLE_INDEX);
        }

        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    sprintln!("[ok]");
    exit_qemu(QemuExitCode::Success);
    toy_os::hlt_loop();
}
