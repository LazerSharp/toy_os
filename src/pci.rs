#![allow(unused)]

use crate::{
    cprintln,
    driver::{net::NetRTL8138, register_pci_device, Driver},
    print, println,
};
use core::{num, panic};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

lazy_static! {
    static ref PCI: Mutex<Pci> = Mutex::new(Pci::new());
}
struct Pci {
    data_port: Port<u32>,
    command_port: Port<u32>, // address port
}

#[derive(Debug)]
pub struct DeviceDescriptor {
    device_data: PciDevice,

    vendor_id: u16,
    device_id: u16,

    class_id: u8,
    subclass_id: u8,
    interface_id: u8,
    revision_id: u8,

    header_type: u8,
    port_base: u16,
}

impl Pci {
    fn new() -> Self {
        Pci {
            command_port: Port::new(0xcf8),
            data_port: Port::new(0xcfc),
        }
    }

    fn scan_devices(&mut self) {
        cprintln!(Yellow, "scan pci called");
        for bus in 0u8..8 {
            for device in 0u8..32 {
                let num_func = if self.device_has_function(bus, device) {
                    8
                } else {
                    1
                };
                for function in 0u8..num_func {
                    let device = PciDevice::new(bus, device, function);

                    //let device = Self::get_device_dscriptor(pci_dev_data);
                    if device.vendor_id() == 0x0000 || device.vendor_id() == 0xFFFF {
                        break;
                    }
                    cprintln!(
                        Magenta,
                        "vendor = [{:X}], device = [{:X}], header_type = [{:X}] ",
                        device.vendor_id(),
                        device.device_id(),
                        device.header_type()
                    );
                    if register_pci_device(device) {
                        cprintln!(
                            LightRed,
                            "Device registered for [device_id = {:x}] ",
                            device.device_id()
                        )
                    }
                }
            }
        }
    }

    fn get_device_dscriptor(device_data: PciDevice) -> DeviceDescriptor {
        let dev_vendor_details = device_data.read_32(0x00); //unsafe { self.read(bus, device, function, 0x00) };
        let vendor_id = dev_vendor_details as u16;
        let device_id = (dev_vendor_details >> 16) as u16;

        let dev_class_details = device_data.read_32(0x08); // unsafe { self.read(bus, device, function, 0x00) };
        let revision_id = dev_class_details as u8;
        let interface_id = (dev_class_details >> 8) as u8;
        let subclass_id = (dev_class_details >> 16) as u8;
        let class_id = (dev_class_details >> 24) as u8;

        let bar0 = device_data.read_32(0x10); //unsafe { self.read(bus, device, function, 0x10) };

        DeviceDescriptor {
            device_data,
            vendor_id,
            device_id,
            header_type: device_data.read_8(0x0E), //unsafe { self.read(bus, device, function, 0x0E) } as u8,
            revision_id,
            interface_id,
            class_id,
            subclass_id,
            port_base: if (bar0 & 0x1) == 1 {
                (bar0 & (!0x3)) as u16
            } else {
                0u16
            },
        }
    }

    fn device_has_function(&mut self, bus: u8, device: u8) -> bool {
        let mut dd = PciDevice::new(bus, device, 0);
        let r = dd.read_8(0x0E) & (1 << 7);
        r != 0
    }
}

/// Device offsets
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum DeviceOffset {
    VendorId = 0x0,
    DeviceId = 0x2,
    Command = 0x4,
    Status = 0x6,
    RevisionId = 0x8,
    ProgrammingInterface = 0x9,
    Subclass = 0xA,
    Class = 0xB,
    BaseRegister0 = 0x10,
    HeaderType = 0xE,
    InteruptLine = 0x3C,
}

impl DeviceOffset {
    pub fn port(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    bus: u8,
    device: u8,
    function: u8,
}

impl PciDevice {
    fn new(bus: u8, device: u8, function: u8) -> Self {
        PciDevice {
            bus,
            device,
            function,
        }
    }

    fn build_command(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        (1u32 << 31)
            | ((bus as u32 & 0xFF) << 16)
            | ((device as u32 & 0x1F) << 11)
            | ((function as u32 & 0x07) << 8)
            | (offset as u32 & 0xFC)
    }
    pub unsafe fn read(&self, offset: u8) -> u32 {
        let mut pci = PCI.lock();
        let command = Self::build_command(self.bus, self.device, self.function, offset);
        pci.command_port.write(command);
        let data = pci.data_port.read();
        data >> (8 * (offset & 3))
    }

    pub fn vendor_details(&self) -> u32 {
        self.read_32(0x00)
    }

    pub fn read_32(&self, offset: u8) -> u32 {
        unsafe { self.read(offset) }
    }
    pub fn read_16(&self, offset: u8) -> u16 {
        (unsafe { self.read(offset) }) as u16
    }
    pub fn read_8(&self, offset: u8) -> u8 {
        (unsafe { self.read(offset) }) as u8
    }

    pub unsafe fn write(&self, offset: u8, value: u32) {
        let mut pci = PCI.lock();
        let command = Self::build_command(self.bus, self.device, self.function, offset);
        pci.command_port.write(command);
        pci.data_port.write(value);
    }

    pub fn vendor_id(&self) -> u16 {
        self.read_16(DeviceOffset::VendorId.port())
    }

    pub fn device_id(&self) -> u16 {
        self.read_16(DeviceOffset::DeviceId.port())
    }

    pub fn command(&self) -> u16 {
        self.read_16(DeviceOffset::Command.port())
    }

    pub fn status(&self) -> u16 {
        self.read_16(DeviceOffset::Status.port())
    }

    pub fn reavision_id(&self) -> u8 {
        self.read_8(DeviceOffset::RevisionId.port())
    }

    pub fn prog_interface(&self) -> u8 {
        self.read_8(DeviceOffset::ProgrammingInterface.port())
    }

    pub fn class(&self) -> u8 {
        self.read_8(DeviceOffset::Class.port())
    }

    pub fn sub_class(&self) -> u8 {
        self.read_8(DeviceOffset::Subclass.port())
    }

    pub fn header_type(&self) -> u8 {
        self.read_8(DeviceOffset::HeaderType.port())
    }

    pub fn interrupt_line(&self) -> u8 {
        self.read_8(DeviceOffset::InteruptLine.port())
    }

    pub fn base_register(&self, num: u8) -> u16 {
        if num > 5 {
            panic!("Invalid base register number!")
        }
        let base_address = self.read_32(DeviceOffset::BaseRegister0.port() + num * 32);

        let bar_type = base_address & 0x1;
        // make sure it is io type
        if bar_type != 1 {
            cprintln!(Red, "BAR{} = {}, type = {}", num, base_address, bar_type);
            panic!("Memory Base register. I dont know how to handle");
        }
        (base_address & (!0x3)) as u16
    }
}

pub fn scan_pci() {
    Pci::new().scan_devices();
}
