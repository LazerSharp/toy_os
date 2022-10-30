use crate::cprintln;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    cprintln!(DarkGray, "CPU EXCEPTION: Breakpoint\n {:#?}", stack_frame);
}

pub fn init_idt() {
    IDT.load();
}

#[test_case]
pub fn test_breakpoibt_interrupt() {
    x86_64::instructions::interrupts::int3();
}
