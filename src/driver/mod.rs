pub mod rtl8139_net_card;

pub trait Driver {
    fn init(&mut self);
}
