use crate::cprint;
use crate::cprintln;
use crate::gdt;
use crate::hlt_loop;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::HandlerFunc;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: Mutex<InterruptDescriptorTable> = {
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
        idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(mouse_intr_handler);
        Mutex::new(idt)
    };
}

pub fn init_idt() {
    unsafe {
        IDT.lock().load_unsafe();
    }
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

// Hardware interrupts

extern "x86-interrupt" fn timer_intr_handler(_stack_frame: InterruptStackFrame) {
    //cprint!(LightGreen, ".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8())
    };
}

extern "x86-interrupt" fn mouse_intr_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let _m: u8 = unsafe { port.read() };

    cprint!(Pink, "MOUSE");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8())
    };
}

extern "x86-interrupt" fn keyboard_intr_handler(_stack_frame: InterruptStackFrame) {
    //use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    //use spin::Mutex;
    use x86_64::instructions::port::Port;
    cprint!(Green, "KEY_PRESSED");
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
    Keyboard = PIC_1_OFFSET + 1,
    Timer = PIC_1_OFFSET,
    Mouse = PIC_1_OFFSET + 12,
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
    unsafe {
        let mut pics = PICS.lock();
        pics.write_masks(0x00, 0x00);
        pics.initialize();
        for mask in pics.read_masks() {
            cprintln!(LightGray, "{:8b}", mask);
        }
    }
    x86_64::instructions::interrupts::enable();
}

pub fn set_interrupt_handler_fn(irq: usize, handler: HandlerFunc) {
    let mut idt = IDT.lock();
    idt[PIC_1_OFFSET as usize + irq].set_handler_fn(handler);
}

#[test_case]
pub fn test_breakpoibt_interrupt() {
    x86_64::instructions::interrupts::int3();
}
