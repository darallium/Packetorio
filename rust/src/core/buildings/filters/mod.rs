use crate::core::packet::Packet;

pub mod content_filter;
pub mod ip_filter;
pub mod length_filter;
pub mod port_filter;
pub mod protocol_filter;

pub trait Filter {
    fn filter(&self, packet: &Packet) -> bool;
}
