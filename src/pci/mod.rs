#![allow(unused)]

use crate::{
    cprintln,
    driver::{rtl8139_net_card::NetworkDriver, Driver},
    print, println,
};
use core::num;
use x86_64::instructions::port::Port;

struct Pci {
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
                    let mut pci_dev_data = PciDeviceData::new(self, bus, device, function);

                    let device = Self::get_device_dscriptor(&mut pci_dev_data);
                    if device.vendor_id == 0x0000 || device.vendor_id == 0xFFFF {
                        break;
                    }
                    cprintln!(
                        Magenta,
                        "vendor = [{:X}], device = [{:X}], header_type = [{:X}], port base = [{:X}] ",
                        device.vendor_id,
                        device.device_id,
                        device.header_type,
                        device.port_base
                    );
                    if device.vendor_id == 0x10EC && device.device_id == 0x8139 {
                        cprintln!(Green, "Yeayyyy! Found Network Card - RTL8139");
                        let pci_command = pci_dev_data.read_32(0x04);
                        println!(
                            "pci command data (before bus mastering) => {:b}",
                            pci_command
                        );
                        unsafe { pci_dev_data.write(0x04, (pci_command | 0x04)) };

                        let pci_command = pci_dev_data.read_32(0x04);
                        println!(
                            "pci command data (after bus mastering)  => {:b}",
                            pci_command
                        );

                        let mut driver = NetworkDriver::new(device.port_base);
                        driver.init();
                    }
                }
            }
        }
    }

    fn get_device_dscriptor(device_data: &mut PciDeviceData) -> DeviceDescriptor {
        //let mut device_data = PciDeviceData::new(self, bus, device, function);

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
            bus: device_data.bus,
            device: device_data.device,
            function: device_data.function,
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
        let mut dd = PciDeviceData::new(self, bus, device, 0);
        let r = dd.read_8(0x0E) & (1 << 7);
        r != 0
    }
}

pub struct PciDeviceData<'a> {
    pci: &'a mut Pci,
    bus: u8,
    device: u8,
    function: u8,
}

impl<'a> PciDeviceData<'a> {
    fn new(pci: &'a mut Pci, bus: u8, device: u8, function: u8) -> Self {
        PciDeviceData {
            pci,
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
    pub unsafe fn read(&mut self, offset: u8) -> u32 {
        let command = Self::build_command(self.bus, self.device, self.function, offset);
        self.pci.command_port.write(command);
        let data = self.pci.data_port.read();
        data >> (8 * (offset & 3))
    }

    pub fn read_32(&mut self, offset: u8) -> u32 {
        unsafe { self.read(offset) }
    }
    pub fn read_16(&mut self, offset: u8) -> u16 {
        (unsafe { self.read(offset) }) as u16
    }
    pub fn read_8(&mut self, offset: u8) -> u8 {
        (unsafe { self.read(offset) }) as u8
    }

    pub unsafe fn write(&mut self, offset: u8, value: u32) {
        let command = Self::build_command(self.bus, self.device, self.function, offset);
        self.pci.command_port.write(command);
        self.pci.data_port.write(value);
    }
}

pub fn scan_pci() {
    Pci::new().scan_devices();
}
