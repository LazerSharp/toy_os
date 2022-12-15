use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use spin::Mutex;

use crate::{driver::net::NetRTL8138, pci::PciDevice};

//pub mod rtl8139_net_card;
pub mod net;
pub trait Driver {
    fn init(&mut self);
    fn new(pci_device: PciDevice) -> Self;
}

static NIC_DRIVER: OnceCell<Arc<Mutex<NetRTL8138>>> = OnceCell::uninit();

pub fn register_pci_device(pci_device: PciDevice) -> bool {
    match (pci_device.vendor_id(), pci_device.device_id()) {
        (0x10EC, 0x8139) => {
            // RTL8139 Network card
            //cprintln!(Green, "Yeayyyy! Found Network Card - RTL8139");
            let mut driver = NetRTL8138::new(pci_device);
            driver.init();
            //cprintln!(Green, "driver initiated");
            NIC_DRIVER
                .try_init_once(|| Arc::new(Mutex::new(driver)))
                .expect("NIC should be called once");
            true
        }
        _ => false,
    }
}

pub fn nic() -> Arc<Mutex<NetRTL8138>> {
    let driver = NIC_DRIVER.try_get();
    let driver = driver.expect("NIC not initialized yet");
    Arc::clone(driver)
}
