use x86_64::{
    instructions::port::{PortReadOnly, PortWriteOnly},
    structures::port::{PortRead, PortWrite},
};

pub fn read_port<T: PortRead>(port: u16) -> T {
    let mut port = PortReadOnly::<T>::new(port);
    unsafe { port.read() }
}

pub fn write_port<T: PortWrite>(port: u16, data: T) {
    let mut port = PortWriteOnly::<T>::new(port);
    unsafe { port.write(data) }
}
