pub struct NetworkDriver {
    mac_ports: [Port<u8>; 6],
}
use core::convert::TryInto;

use alloc::vec::Vec;
use x86_64::instructions::port::Port;

use crate::{cprint, cprintln, driver::Driver, print, println};

impl NetworkDriver {
    pub fn new(base_addr: u16) -> Self {
        let mac_ports: [Port<u8>; 6] = (0..6)
            .into_iter()
            .map(|i| Port::<u8>::new(base_addr + i))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        NetworkDriver { mac_ports }
    }
}

impl NetworkDriver {
    pub fn mac_address(&mut self) -> [u8; 6] {
        let mac: [u8; 6] = self
            .mac_ports
            .iter_mut()
            .map(|port| unsafe { port.read() })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        mac
    }

    pub fn print_mac(&mut self) {
        print!("Mac ");
        let address = self.mac_address();
        cprint!(Brown, "{:02x}", address[0]);
        for byte in &address[1..] {
            cprint!(Brown, ":{:02x}", byte)
        }
        println!();
    }
}

impl super::Driver for NetworkDriver {
    fn init(&mut self) {
        cprintln!(Green, "Initializing Network card");
        self.print_mac();
    }
}

fn _test() {
    let mut net_driver = NetworkDriver::new(0x50);
    net_driver.init();
}
