use crate::cprintln;
use crate::gdt;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_STACK_TABLE_INDEX);
        }

        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    cprintln!(DarkGray, "CPU EXCEPTION: Breakpoint\n {:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) -> ! {
    panic!(
        "CPU EXCEPTION: Double fault\n {:#?} \n Error Code: {}",
        stack_frame, err_code
    );
}

pub fn init_idt() {
    IDT.load();
}

#[test_case]
pub fn test_breakpoibt_interrupt() {
    x86_64::instructions::interrupts::int3();
}
