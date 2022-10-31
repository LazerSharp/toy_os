#![no_std]
#![no_main]

use core::panic::PanicInfo;
use toy_os::{exit_qemu, sprint, sprintln, QemuExitCode};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sprintln!("[ok]");
    exit_qemu(QemuExitCode::Success);
    toy_os::hlt_loop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    sprintln!("[test did not panic]");
    exit_qemu(QemuExitCode::Failure);
    toy_os::hlt_loop();
}

fn should_fail() {
    sprint!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}
