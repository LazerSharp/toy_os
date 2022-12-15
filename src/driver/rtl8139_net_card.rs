pub struct NetworkDriver {
    base_addr: u16,
    rx_buffer: Vec<u8>,
}

use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::ops::Range;
use x86_64::{
    structures::port::{PortRead, PortWrite},
    VirtAddr,
};

const MAC_PORT_RANGE: Range<u16> = 0..6;
const COMMAND_PORT: u16 = 0x37;
const RBSTART: u16 = 0x30;
const CONFIG_1: u16 = 0x52;
/////

const RX_BUF_LEN_IDX: usize = 2; /* 0==8K, 1==16K, 2==32K, 3==64K */
const RX_BUF_LEN: usize = 1024 << RX_BUF_LEN_IDX;
const RX_BUF_PAD: usize = 16; /* see 11th and 12th bit of RCR: 0x44 */
const RX_BUF_WRAP_PAD: usize = 256; /* spare padding to handle pkt wrap */
const RX_BUF_TOT_LEN: usize = RX_BUF_LEN + RX_BUF_PAD + RX_BUF_WRAP_PAD;

use crate::{
    cprint, cprintln, memory, print, println,
    util::{read_port, write_port},
};

impl NetworkDriver {
    pub fn new(base_addr: u16) -> Self {
        NetworkDriver {
            base_addr,
            rx_buffer: vec![0u8; RX_BUF_TOT_LEN],
        }
    }
}

impl NetworkDriver {
    pub fn mac_address(&self) -> [u8; 6] {
        let mac: [u8; 6] = MAC_PORT_RANGE
            .into_iter()
            //.map(|i| Port::<u8>::new(self.base_addr + i))
            .map(|offset| self.read_offset(offset))
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

    fn write_offset<T: PortWrite>(&self, port_offset: u16, data: T) {
        write_port(self.base_addr + port_offset, data)
    }

    fn read_offset<T: PortRead>(&self, port_offset: u16) -> T {
        read_port(self.base_addr + port_offset)
    }

    fn reset(&self) {
        self.write_offset(COMMAND_PORT, 0x10u8);
        while (self.read_offset::<u8>(COMMAND_PORT) & 0x10) != 0 {}
    }

    fn init_recv_buff(&self) {
        let rx_buff_address = self.rx_buffer.as_ptr() as u64;
        cprintln!(
            LightGray,
            "Allocated virtual location: {:x}",
            (rx_buff_address as u32)
        );
        let physical_rx_addr = memory::virtual_to_physical_addr(VirtAddr::new(rx_buff_address));
        if let Some(physical_rx_addr) = physical_rx_addr {
            let addr = physical_rx_addr.as_u64();
            cprintln!(
                Yellow,
                "Allocated physical memory location: {:x}",
                (addr as u32)
            );
            if (addr & 0xFFFF_FFFF_0000_0000) != 0 {
                panic!("Adress not in 32 bit range :(")
            }
            self.write_offset(RBSTART, addr as u32);
        } else {
            panic!("Unable to setup receieve buffer. Page is not mapped!!!")
        }
    }
    /// set Interrupt Mask Register
    fn set_imr(&self) {
        self.write_offset(0x3C, 0x0005u16);
    }

    fn turn_on(&self) {
        self.write_offset(CONFIG_1, 0x00u8);
    }
}

impl super::Driver for NetworkDriver {
    fn init(&mut self) {
        cprintln!(Green, "Initializing Network card");
        self.turn_on();
        self.reset();
        self.init_recv_buff();
        self.set_imr();

        self.print_mac();
        cprintln!(Green, "Initializing Network card - Done!");
    }
}
