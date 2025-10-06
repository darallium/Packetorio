use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum PacketLabel {
    Correct,
    Incorrect,
    #[default]
    Unknown,
}

impl PacketLabel {
    pub fn from_raw(value: i64) -> Self {
        match value {
            1 => PacketLabel::Correct,
            -1 => PacketLabel::Incorrect,
            _ => PacketLabel::Unknown,
        }
    }

    pub fn to_raw(self) -> i64 {
        match self {
            PacketLabel::Correct => 1,
            PacketLabel::Incorrect => -1,
            PacketLabel::Unknown => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Protocol {
    Tcp,
    Udp,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub source_ip: String,
    pub dest_ip: String,
    pub source_port: u16,
    pub dest_port: u16,
    pub protocol: Protocol,
    pub length: u32,
    pub payload: Vec<u8>,
    pub progress: f32,
    pub label: PacketLabel,
}

impl Packet {
    /// Return the payload as an ASCII string with non-ASCII bytes escaped as `\xHH`.
    pub fn payload_to_string(&self) -> String {
        encode_payload_bytes(&self.payload)
    }

    pub fn new(
        source_ip: String,
        dest_ip: String,
        source_port: u16,
        dest_port: u16,
        protocol: Protocol,
        length: u32,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            source_ip,
            dest_ip,
            source_port,
            dest_port,
            protocol,
            length,
            payload,
            progress: 0.0,
            label: PacketLabel::default(),
        }
    }
}

/// Encode raw bytes into an ASCII string, escaping non-printable bytes as `\xHH`.
pub fn encode_payload_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut out = String::with_capacity(bytes.len());
    for &byte in bytes {
        match byte {
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            b'\0' => out.push_str("\\0"),
            0x20..=0x7E => out.push(byte as char),
            _ => {
                out.push_str("\\x");
                out.push(char::from(HEX[(byte >> 4) as usize]));
                out.push(char::from(HEX[(byte & 0x0F) as usize]));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::encode_payload_bytes;

    #[test]
    fn encode_payload_bytes_printable_and_binary() {
        let data = b"Hi\\x";
        assert_eq!(encode_payload_bytes(data), "Hi\\\\x");

        let data = [0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x00, 0xFF];
        assert_eq!(encode_payload_bytes(&data), "Hello\\0\\xff");

        let data = b"Line1\nLine2\t";
        assert_eq!(encode_payload_bytes(data), "Line1\\nLine2\\t");
    }

    #[test]
    fn packet_payload_to_string_uses_encoder() {
        use super::{Packet, Protocol};

        let packet = Packet::new(
            "s".into(),
            "d".into(),
            1,
            2,
            Protocol::Tcp,
            0,
            vec![0x41, 0x00, 0x0A],
        );
        assert_eq!(packet.payload_to_string(), "A\\0\\n");
    }
}
