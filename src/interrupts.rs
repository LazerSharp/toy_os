use crate::cprint;
use crate::cprintln;
use crate::gdt;
use crate::hlt_loop;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::PageFaultErrorCode;
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
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_intr_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_intr_handler);
        idt
    };
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    cprintln!(DarkGray, "CPU EXCEPTION: Page Fault!");
    cprintln!(DarkGray, "Adress accessed: {:?}", Cr2::read());
    cprintln!(DarkGray, "Stack frame: \n {:#?}", stack_frame);
    cprintln!(DarkGray, "Error code: {:?}", error_code);
    hlt_loop();
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

// Hardware interrupts

extern "x86-interrupt" fn timer_intr_handler(_stack_frame: InterruptStackFrame) {
    cprint!(LightGreen, ".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8())
    };
}

extern "x86-interrupt" fn keyboard_intr_handler(_stack_frame: InterruptStackFrame) {
    //use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    //use spin::Mutex;
    use x86_64::instructions::port::Port;

    // lazy_static! {
    //     static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
    //         Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
    //     );
    // }
    //let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };

    crate::task::keyboard::add_scancode(scancode);

    // if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
    //     if let Some(key) = keyboard.process_keyevent(key_event) {
    //         match key {
    //             DecodedKey::Unicode(character) => cprint!(LightCyan, "{}", character),
    //             DecodedKey::RawKey(key) => cprint!(Red, "{:?}", key),
    //         }
    //     }
    // }

    // cprint!(Red, "{}", scancode);

    //cprint!(Red, "k");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8())
    };
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_hw_int() {
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

#[test_case]
pub fn test_breakpoibt_interrupt() {
    x86_64::instructions::interrupts::int3();
}
