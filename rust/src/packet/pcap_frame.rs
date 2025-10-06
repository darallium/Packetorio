use godot::prelude::*;

use etherparse::{NetSlice, SlicedPacket, TransportSlice};

use crate::packet::Packet;

struct ParsedPacket {
    src_ip: String,
    dst_ip: String,
    src_port: u16,
    dst_port: u16,
    protocol: u8,
    payload: Vec<u8>,
}

#[derive(GodotClass)]
#[class(base=Resource)]
pub struct PcapFrame {
    #[base]
    base: Base<Resource>,

    #[var]
    timestamp_sec: i64,
    #[var]
    timestamp_usec: i64,
    #[var]
    orig_len: i64,
    #[var]
    data: PackedByteArray,
    #[var]
    payload: PackedByteArray,
}

#[godot_api]
impl PcapFrame {
    #[func]
    pub fn timestamp_sec(&self) -> i64 {
        self.timestamp_sec
    }
    #[func]
    pub fn timestamp_usec(&self) -> i64 {
        self.timestamp_usec
    }
    #[func]
    pub fn orig_len(&self) -> i64 {
        self.orig_len
    }
    #[func]
    pub fn data(&self) -> PackedByteArray {
        self.data.clone()
    }

    #[func]
    pub fn payload(&self) -> PackedByteArray {
        self.payload.clone()
    }

    #[func]
    pub fn to_packet(&self) -> Option<Gd<Packet>> {
        let data = self.data();
        if data.is_empty() {
            return None;
        }

        let bytes = data.to_vec();
        let parsed = parse_packet_from_bytes(&bytes)?;

        let raw_len = self.orig_len();
        let size = if raw_len <= 0 {
            0
        } else if raw_len > u32::MAX as i64 {
            u32::MAX
        } else {
            raw_len as u32
        };

        let timestamp = self.timestamp_sec() * 1_000_000 + self.timestamp_usec();

        let mut packet = Packet::from_parts(
            parsed.src_ip,
            parsed.dst_ip,
            parsed.src_port,
            parsed.dst_port,
            parsed.protocol,
            size,
            timestamp,
            0,
        );
        {
            let mut packet_mut = packet.bind_mut();
            packet_mut.set_payload_bytes(parsed.payload);
        }

        Some(packet)
    }

    pub(crate) fn refresh_payload(&mut self) {
        let bytes = self.data.to_vec();
        let payload = parse_packet_from_bytes(&bytes)
            .map(|packet| packet.payload)
            .unwrap_or_default();

        let mut packed = PackedByteArray::new();
        packed.resize(payload.len());
        if !payload.is_empty() {
            packed.as_mut_slice().copy_from_slice(&payload);
        }
        self.payload = packed;
    }
}

#[godot_api]
impl IResource for PcapFrame {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            timestamp_sec: 0,
            timestamp_usec: 0,
            orig_len: 0,
            data: PackedByteArray::new(),
            payload: PackedByteArray::new(),
        }
    }
}

fn parse_packet_from_bytes(bytes: &[u8]) -> Option<ParsedPacket> {
    let parsed = SlicedPacket::from_ethernet(bytes).ok()?;
    let SlicedPacket {
        link: _,
        link_exts: _,
        net,
        transport,
    } = parsed;

    let (src_ip, dst_ip, protocol, net_payload) = match net {
        Some(NetSlice::Ipv4(ipv4)) => {
            let header = ipv4.header();
            let src = header.source_addr();
            let dst = header.destination_addr();
            let protocol = ipv4.payload().ip_number.0;
            let payload = ipv4.payload().payload.to_vec();
            (src.to_string(), dst.to_string(), protocol, payload)
        }
        Some(NetSlice::Ipv6(ipv6)) => {
            let header = ipv6.header();
            let src = header.source_addr();
            let dst = header.destination_addr();
            let protocol = ipv6.payload().ip_number.0;
            let payload = ipv6.payload().payload.to_vec();
            (src.to_string(), dst.to_string(), protocol, payload)
        }
        _ => return None,
    };

    let (src_port, dst_port, transport_payload) = match transport {
        Some(TransportSlice::Tcp(tcp)) => (
            tcp.source_port(),
            tcp.destination_port(),
            tcp.payload().to_vec(),
        ),
        Some(TransportSlice::Udp(udp)) => (
            udp.source_port(),
            udp.destination_port(),
            udp.payload().to_vec(),
        ),
        Some(TransportSlice::Icmpv4(icmp)) => (0, 0, icmp.payload().to_vec()),
        Some(TransportSlice::Icmpv6(icmp)) => (0, 0, icmp.payload().to_vec()),
        None => (0, 0, Vec::new()),
    };

    let payload = if transport_payload.is_empty() {
        net_payload
    } else {
        transport_payload
    };

    Some(ParsedPacket {
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        protocol,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use etherparse::PacketBuilder;

    fn build_packet_bytes(payload: &[u8]) -> Vec<u8> {
        let builder = PacketBuilder::ethernet2([0, 1, 2, 3, 4, 5], [5, 4, 3, 2, 1, 0])
            .ipv4([192, 168, 1, 1], [192, 168, 1, 2], 20)
            .tcp(1234, 80, 1, 10);

        let mut serialized = Vec::with_capacity(builder.size(payload.len()));
        builder
            .write(&mut serialized, payload)
            .expect("failed to serialize packet");

        serialized
    }

    #[test]
    fn parse_packet_extracts_transport_payload() {
        let payload = b"payload bytes";
        let bytes = build_packet_bytes(payload);

        let parsed = parse_packet_from_bytes(&bytes).expect("packet");
        assert_eq!(parsed.src_ip, "192.168.1.1");
        assert_eq!(parsed.dst_ip, "192.168.1.2");
        assert_eq!(parsed.src_port, 1234);
        assert_eq!(parsed.dst_port, 80);
        assert_eq!(parsed.protocol, 6); // TCP
        assert_eq!(parsed.payload, payload);
    }

    #[test]
    fn parse_packet_falls_back_to_net_payload() {
        // Construct an IPv4 ICMP packet (no transport header recognized by helper).
        let builder = PacketBuilder::ethernet2([0, 1, 2, 3, 4, 5], [5, 4, 3, 2, 1, 0])
            .ipv4([10, 0, 0, 1], [10, 0, 0, 2], 20)
            .icmpv4_echo_request(0x1234, 1);

        let payload = b"icmp body";
        let mut serialized = Vec::with_capacity(builder.size(payload.len()));
        builder
            .write(&mut serialized, payload)
            .expect("failed to serialize packet");

        let parsed = parse_packet_from_bytes(&serialized).expect("packet");
        assert_eq!(parsed.src_ip, "10.0.0.1");
        assert_eq!(parsed.dst_ip, "10.0.0.2");
        assert_eq!(parsed.src_port, 0);
        assert_eq!(parsed.dst_port, 0);
        assert_eq!(parsed.protocol, 1); // ICMP
        assert_eq!(parsed.payload, payload);
    }
}
