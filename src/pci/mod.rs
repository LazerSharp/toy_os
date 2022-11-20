#![allow(unused)]

use core::num;

use x86_64::instructions::port::Port;

use crate::{cprintln, print};

pub struct Pci {
    data_port: Port<u32>,
    command_port: Port<u32>, // address port
}

#[derive(Debug)]
pub struct DeviceDescriptor {
    bus: u8,
    device: u8,
    function: u8,
    vendor_id: u16,
    device_id: u16,
}

impl Pci {
    pub fn new() -> Self {
        Pci {
            command_port: Port::new(0xcf8),
            data_port: Port::new(0xcfc),
        }
    }

    fn build_command(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        (1u32 << 31)
            | ((bus as u32 & 0xFF) << 16)
            | ((device as u32 & 0x1F) << 11)
            | ((function as u32 & 0x07) << 8)
            | (offset as u32 & 0xFC)
    }

    unsafe fn read(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> u16 {
        let command = Self::build_command(bus, device, function, offset);
        self.command_port.write(command);
        let data = self.data_port.read();
        (data >> (8 * (offset & 2))) as u16
    }

    unsafe fn write(&mut self, bus: u8, device: u8, function: u8, offset: u8, value: u32) {
        let command = Self::build_command(bus, device, function, offset);
        self.command_port.write(command);
        self.data_port.write(value);
    }

    fn scan_devices(&mut self) {
        for bus in 0u8..8 {
            for device in 0u8..32 {
                let num_func = if self.device_has_function(bus, device) {
                    8
                } else {
                    1
                };
                for function in 0u8..num_func {
                    let device = self.get_device_dscriptor(bus, device, function);
                    if (device.vendor_id == 0x0000 || device.vendor_id == 0xFFFF) {
                        break;
                    }
                    cprintln!(
                        Magenta,
                        "vendor = [{:x}], device = [{:x}] ",
                        device.vendor_id,
                        device.device_id
                    );
                }
            }
        }
    }

    fn get_device_dscriptor(&mut self, bus: u8, device: u8, function: u8) -> DeviceDescriptor {
        DeviceDescriptor {
            bus,
            device,
            function,
            vendor_id: unsafe { self.read(bus, device, function, 0x00) },
            device_id: unsafe { self.read(bus, device, function, 0x02) },
        }
    }

    fn device_has_function(&mut self, bus: u8, device: u8) -> bool {
        let r = unsafe { self.read(bus, device, 0, 0x0E) } & (1 << 7);
        !(r == 1)
    }
}

pub fn scan_pci() {
    Pci::new().scan_devices();
}
