#![allow(unused)]
use x86_64::structures::idt::InterruptStackFrame;

use self::rtl8139::RTL8139;
use super::Driver;
use crate::{cprint, cprintln, interrupts::set_interrupt_handler_fn, memory, pci::PciDevice};

pub mod rtl8139;

pub struct NetRTL8138 {
    driver: RTL8139,
    pci_device: PciDevice,
}

impl NetRTL8138 {
    pub fn mac(&self) -> [u8; 6] {
        self.driver.mac()
    }
}

impl Driver for NetRTL8138 {
    fn new(pci_device: PciDevice) -> Self {
        // pci bus mastering
        let pci_command = pci_device.command();
        unsafe { pci_device.write(0x04, (pci_command | 0x04) as u32) };

        // instantiate driver
        NetRTL8138 {
            driver: unsafe {
                RTL8139::preload_unchecked(pci_device.base_register(0), |address| {
                    memory::virtual_to_physical_addr(address).unwrap()
                })
            },
            pci_device,
        }
    }

    fn init(&mut self) {
        self.driver.load();
        let irq = self.pci_device.interrupt_line();
        cprintln!(Green, "mac address => {:?}", self.driver.mac());
        cprintln!(Green, "Interrupt line for NIC => {}", irq);
        set_interrupt_handler_fn(irq as usize, nic_data_rcvd_interrupt_handler);
        cprintln!(Green, "Interrupt handeler set for NIC  ");
    }
}

extern "x86-interrupt" fn nic_data_rcvd_interrupt_handler(_stack_frame: InterruptStackFrame) {
    cprintln!(Green, "NIC Date received");
    RTL8139::on_interrupt();
}
