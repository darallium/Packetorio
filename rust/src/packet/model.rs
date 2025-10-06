use godot::prelude::*;

use crate::core::packet::encode_payload_bytes;

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct Packet {
    #[base]
    base: Base<Resource>,

    #[var]
    src_ip: GString,
    #[var]
    dst_ip: GString,
    #[var]
    src_port: i64,
    #[var]
    dst_port: i64,
    #[var]
    protocol: i64,
    #[var]
    packet_size: i64,
    #[var]
    timestamp: i64,
    #[var]
    label: i64,
    payload: Vec<u8>,
}

#[godot_api]
impl IResource for Packet {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            src_ip: GString::default(),
            dst_ip: GString::default(),
            src_port: 0,
            dst_port: 0,
            protocol: 0,
            packet_size: 0,
            timestamp: 0,
            label: 0,
            payload: Vec::new(),
        }
    }
}

impl Packet {
    pub(crate) fn from_parts(
        src_ip: String,
        dst_ip: String,
        src_port: u16,
        dst_port: u16,
        protocol: u8,
        packet_size: u32,
        timestamp: i64,
        label: i64,
    ) -> Gd<Packet> {
        let mut packet = Packet::new_gd();
        {
            let mut packet_mut = packet.bind_mut();
            packet_mut.src_ip = src_ip.into();
            packet_mut.dst_ip = dst_ip.into();
            packet_mut.src_port = src_port as i64;
            packet_mut.dst_port = dst_port as i64;
            packet_mut.protocol = protocol as i64;
            packet_mut.packet_size = packet_size as i64;
            packet_mut.timestamp = timestamp;
            packet_mut.label = label;
            packet_mut.payload = Vec::new();
        }
        packet
    }

    pub(crate) fn set_payload_bytes(&mut self, payload: Vec<u8>) {
        self.payload = payload;
    }

    pub(crate) fn payload_to_string(&self) -> String {
        encode_payload_bytes(&self.payload)
    }
}

#[godot_api]
impl Packet {
    #[func]
    pub fn get_payload_string(&self) -> GString {
        self.payload_to_string().into()
    }
}
