//! Utilities for loading packet captures described in JSON into a `Traffic` resource.
//!
//! The loader accepts either a top-level array or an object containing a `packets` array. Each
//! entry must contain the following keys:
//!
//! - `src_ip` / `dst_ip` – IPv4/IPv6 textual addresses.
//! - `src_port` / `dst_port` – TCP/UDP port numbers (use `0` when unavailable).
//! - `protocol` – IP protocol number (e.g. `6` for TCP, `17` for UDP).
//! - `size` – Packet size in bytes.
//! - `timestamp` (optional) – Microseconds since the capture start (the loader normalizes the
//!   minimum timestamp to `0`).
//! - `label` (optional) – Either `"correct"`, `"incorrect"`, or `"unknown"`; defaults to
//!   `"unknown"`.
//! - `payload` (optional) – Payload bytes encoded as a string. ASCII characters may appear
//!   directly; other bytes must be written as Python-style escapes (`\xHH`).
//!
//! #### Example
//!
//! ```json
//! {
//!   "packets": [
//!     {
//!       "src_ip": "192.168.0.1",
//!       "dst_ip": "10.0.0.5",
//!       "src_port": 443,
//!       "dst_port": 60234,
//!       "protocol": 6,
//!       "size": 1500,
//!       "timestamp": 2000,
//!       "label": "correct",
//!       "payload": "GET / HTTP/1.1\\r\\nHost: example.com\\r\\n\\r\\n"
//!     }
//!   ]
//! }
//! ```
use godot::classes::{Json, ResourceLoader};
use godot::prelude::*;

use super::{Packet, Traffic, normalize_timestamp};
use crate::core::packet::PacketLabel;

struct PacketEntry {
    src_ip: String,
    dst_ip: String,
    src_port: u16,
    dst_port: u16,
    protocol: u8,
    size: u32,
    timestamp: Option<i64>,
    label: Option<PacketLabelValue>,
    payload: Option<String>,
}

#[derive(Copy, Clone)]
enum PacketLabelValue {
    Correct,
    Incorrect,
    Unknown,
}

impl From<PacketLabelValue> for PacketLabel {
    fn from(value: PacketLabelValue) -> Self {
        match value {
            PacketLabelValue::Correct => PacketLabel::Correct,
            PacketLabelValue::Incorrect => PacketLabel::Incorrect,
            PacketLabelValue::Unknown => PacketLabel::Unknown,
        }
    }
}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct JsonLoader {
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for JsonLoader {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl JsonLoader {
    #[func]
    /// Load `Traffic` from a JSON file with the packet schema documented at the top of this
    /// module.
    pub fn load_traffic(&self, path: GString) -> Option<Gd<Traffic>> {
        let mut loader = ResourceLoader::singleton();
        let Some(resource) = loader.load(&path) else {
            godot_error!("Failed to load resource");
            return None;
        };
        let Ok(json) = resource.try_cast::<Json>() else {
            godot_error!("Resource is not a JSON file");
            return None;
        };

        let data = json.get_data();
        let mut entries = match parse_packets_from_variant(&data) {
            Ok(entries) => entries,
            Err(err) => {
                godot_error!("Failed to parse JSON traffic: {}", err);
                return None;
            }
        };
        entries.sort_by_key(|entry| entry.timestamp.unwrap_or(0));
        let baseline = entries.iter().filter_map(|entry| entry.timestamp).min();

        let mut packets = Array::<Gd<Packet>>::new();
        for entry in entries.into_iter() {
            let PacketEntry {
                src_ip,
                dst_ip,
                src_port,
                dst_port,
                protocol,
                size,
                timestamp,
                label,
                payload,
            } = entry;

            let raw = timestamp.unwrap_or(0);
            let normalized = normalize_timestamp(raw, baseline);
            let label = label
                .map(PacketLabel::from)
                .unwrap_or(PacketLabel::Unknown)
                .to_raw();

            let payload_bytes = match payload {
                Some(payload_str) => decode_payload(&payload_str)?,
                None => Vec::new(),
            };

            let mut packet = Packet::from_parts(
                src_ip, dst_ip, src_port, dst_port, protocol, size, normalized, label,
            );
            {
                let mut packet_mut = packet.bind_mut();
                packet_mut.set_payload_bytes(payload_bytes);
            }
            packets.push(&packet);
        }

        let mut traffic = Traffic::new_gd();
        {
            let mut traffic_mut = traffic.bind_mut();
            traffic_mut.set_packets(packets);
        }

        Some(traffic)
    }
}

fn parse_packets_from_variant(data: &Variant) -> Result<Vec<PacketEntry>, String> {
    if let Ok(dict) = data.try_to::<Dictionary>() {
        if let Some(packets_variant) = dict.get("packets") {
            let array = packets_variant
                .try_to::<VariantArray>()
                .map_err(|_| String::from("'packets' は配列である必要があります"))?;
            parse_packet_array(&array)
        } else {
            Err(String::from("JSONルートに 'packets' 配列が見つかりません"))
        }
    } else if let Ok(array) = data.try_to::<VariantArray>() {
        parse_packet_array(&array)
    } else {
        Err(String::from(
            "JSONルートは配列、または 'packets' 配列を持つ辞書である必要があります",
        ))
    }
}

fn parse_packet_array(array: &VariantArray) -> Result<Vec<PacketEntry>, String> {
    let mut packets = Vec::with_capacity(array.len());
    for (index, entry_variant) in array.iter_shared().enumerate() {
        let entry_dict = entry_variant
            .try_to::<Dictionary>()
            .map_err(|_| format!("インデックス {} の要素が辞書ではありません", index))?;
        let packet = parse_packet_entry(&entry_dict)
            .map_err(|err| format!("インデックス {} のパケット: {}", index, err))?;
        packets.push(packet);
    }
    Ok(packets)
}

fn parse_packet_entry(dict: &Dictionary) -> Result<PacketEntry, String> {
    let src_ip = dict
        .get("src_ip")
        .and_then(|v| variant_to_string(&v))
        .ok_or_else(|| String::from("'src_ip' が存在しない、または文字列ではありません"))?;
    let dst_ip = dict
        .get("dst_ip")
        .and_then(|v| variant_to_string(&v))
        .ok_or_else(|| String::from("'dst_ip' が存在しない、または文字列ではありません"))?;
    let src_port = dict
        .get("src_port")
        .and_then(|v| variant_to_u16(&v))
        .ok_or_else(|| String::from("'src_port' が存在しない、または範囲外です"))?;
    let dst_port = dict
        .get("dst_port")
        .and_then(|v| variant_to_u16(&v))
        .ok_or_else(|| String::from("'dst_port' が存在しない、または範囲外です"))?;
    let protocol = dict
        .get("protocol")
        .and_then(|v| variant_to_u8(&v))
        .ok_or_else(|| String::from("'protocol' が存在しない、または範囲外です"))?;
    let size = dict
        .get("size")
        .and_then(|v| variant_to_u32(&v))
        .ok_or_else(|| String::from("'size' が存在しない、または範囲外です"))?;

    let timestamp = match dict.get("timestamp") {
        Some(value) => Some(
            variant_to_i64(&value)
                .ok_or_else(|| String::from("'timestamp' が整数値ではありません"))?,
        ),
        None => None,
    };

    let label = match dict.get("label") {
        Some(value) => {
            let label_str = variant_to_string(&value)
                .ok_or_else(|| String::from("'label' が文字列ではありません"))?;
            Some(
                parse_label_value(&label_str)
                    .ok_or_else(|| format!("未知のラベル値 '{}'", label_str))?,
            )
        }
        None => None,
    };

    let payload = match dict.get("payload") {
        Some(value) => Some(
            variant_to_string(&value)
                .ok_or_else(|| String::from("'payload' が文字列ではありません"))?,
        ),
        None => None,
    };

    Ok(PacketEntry {
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        protocol,
        size,
        timestamp,
        label,
        payload,
    })
}

fn variant_to_string(value: &Variant) -> Option<String> {
    if let Ok(gstr) = value.try_to::<GString>() {
        return Some(gstr.to_string());
    }
    if let Ok(rust_string) = value.try_to::<String>() {
        return Some(rust_string);
    }
    None
}

fn variant_to_i64(value: &Variant) -> Option<i64> {
    if let Ok(int_value) = value.try_to::<i64>() {
        return Some(int_value);
    }
    if let Ok(int_value) = value.try_to::<i32>() {
        return Some(int_value as i64);
    }
    if let Ok(float_value) = value.try_to::<f64>()
        && float_value.is_finite()
    {
        return Some(float_value.round() as i64);
    }
    variant_to_string(value)
        .and_then(|text| text.parse::<f64>().ok())
        .and_then(|float_value| {
            if float_value.is_finite() {
                Some(float_value.round() as i64)
            } else {
                None
            }
        })
}

fn variant_to_u32(value: &Variant) -> Option<u32> {
    variant_to_i64(value).and_then(|int_value| {
        if (0..=u32::MAX as i64).contains(&int_value) {
            Some(int_value as u32)
        } else {
            None
        }
    })
}

fn variant_to_u16(value: &Variant) -> Option<u16> {
    variant_to_i64(value).and_then(|int_value| {
        if (0..=u16::MAX as i64).contains(&int_value) {
            Some(int_value as u16)
        } else {
            None
        }
    })
}

fn variant_to_u8(value: &Variant) -> Option<u8> {
    variant_to_i64(value).and_then(|int_value| {
        if (0..=u8::MAX as i64).contains(&int_value) {
            Some(int_value as u8)
        } else {
            None
        }
    })
}

fn parse_label_value(text: &str) -> Option<PacketLabelValue> {
    if text.eq_ignore_ascii_case("correct") {
        Some(PacketLabelValue::Correct)
    } else if text.eq_ignore_ascii_case("incorrect") {
        Some(PacketLabelValue::Incorrect)
    } else if text.eq_ignore_ascii_case("unknown") {
        Some(PacketLabelValue::Unknown)
    } else {
        None
    }
}

fn decode_payload(encoded: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(encoded.len());
    let mut iter = encoded.as_bytes().iter().copied();

    while let Some(byte) = iter.next() {
        if byte == b'\\' {
            let Some(next) = iter.next() else {
                return None;
            };

            if next == b'x' || next == b'X' {
                let high = iter.next()?;
                let low = iter.next()?;
                let high_val = hex_value(high)?;
                let low_val = hex_value(low)?;
                bytes.push((high_val << 4) | low_val);
            } else {
                bytes.push(match next {
                    b'\\' => b'\\',
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b't' => b'\t',
                    b'0' => b'\0',
                    other => other,
                });
            }
        } else {
            bytes.push(byte);
        }
    }

    Some(bytes)
}

fn hex_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_label_value_conversion() {
        assert_eq!(
            PacketLabel::from(PacketLabelValue::Correct),
            PacketLabel::Correct
        );
        assert_eq!(
            PacketLabel::from(PacketLabelValue::Incorrect),
            PacketLabel::Incorrect
        );
        assert_eq!(
            PacketLabel::from(PacketLabelValue::Unknown),
            PacketLabel::Unknown
        );
    }

    #[test]
    fn decode_payload_supports_mixed_ascii_and_hex() {
        let encoded = "Hello\\x20World\\x21";
        let expected = b"Hello World!".to_vec();
        assert_eq!(decode_payload(encoded), Some(expected));
    }

    #[test]
    fn decode_payload_handles_escape_sequences() {
        let encoded = "Line1\\nLine2";
        assert_eq!(decode_payload(encoded), Some(b"Line1\nLine2".to_vec()));
    }

    #[test]
    fn decode_payload_rejects_invalid_hex() {
        assert_eq!(decode_payload("\\xZZ"), None);
        assert_eq!(decode_payload("\\x1"), None);
    }
}
