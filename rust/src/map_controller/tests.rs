use super::MapController;
use crate::core::packet::{Packet as CorePacket, PacketLabel, Protocol};

#[test]
fn packet_to_dictionary_includes_expected_fields() {
    let mut packet = CorePacket::new(
        "10.0.0.1".to_string(),
        "10.0.0.2".to_string(),
        8080,
        443,
        Protocol::Tcp,
        512,
        b"hello".to_vec(),
    );
    packet.label = PacketLabel::Incorrect;
    packet.progress = 0.42;

    let row = MapController::packet_to_export_row_for_test(&packet, 42);

    assert_eq!(row.building_id, 42);
    assert_eq!(row.source_ip, "10.0.0.1");
    assert_eq!(row.dest_ip, "10.0.0.2");
    assert_eq!(row.source_port, 8080);
    assert_eq!(row.dest_port, 443);
    assert_eq!(row.protocol, Protocol::Tcp);
    assert_eq!(row.length, 512);
    assert_eq!(row.label, PacketLabel::Incorrect);
    assert!((row.progress - 0.42).abs() < f32::EPSILON);
    assert_eq!(row.payload_string, packet.payload_to_string());
}
