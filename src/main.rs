#![no_std]
#![no_main]
// Custom test framework
#![feature(custom_test_frameworks)]
#![test_runner(toy_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use toy_os::memory;
use toy_os::{cprintln, println};
use x86_64::structures::paging::{Page, Translate};
use x86_64::{registers::control::Cr3, VirtAddr};

entry_point!(kernel_main);

//#[no_mangle]
//pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello {}{}", "there", "!");
    toy_os::init();

    // read page table

    let (level_4_page_table, _) = Cr3::read();

    cprintln!(
        Pink,
        "Level 4 page table starts at {:?}",
        level_4_page_table.start_address()
    );

    cprintln!(
        LightGray,
        "physical memory offset : {:?}",
        VirtAddr::new(boot_info.physical_memory_offset)
    );

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         cprintln!(LightGreen, "L4 Entry {} -> {:?}", i, entry);
    //     }
    // }

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        0xb8000 + boot_info.physical_memory_offset,
        // some code page
        0x201008,
        0x401008 + boot_info.physical_memory_offset,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    // new: initialize a mapper
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    for addr in addresses {
        let vaddr = VirtAddr::new(addr);
        cprintln!(Brown, "{:?} -> {:?}", vaddr, mapper.translate_addr(vaddr));
        // let paddr = unsafe { virttual_to_physical_addr(vaddr, phys_mem_offset) };
        // cprintln!(Green, "{:?} -> {:?}", vaddr, paddr);
    }

    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

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
