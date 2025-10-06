use crate::core::building::BuildingType;
use crate::core::dto::Vec2i;
use crate::core::packet::{Packet, PacketLabel, Protocol};
use crate::logic::packet_completion;
use crate::map_controller::World;

fn make_packet(
    source_ip: &str,
    dest_ip: &str,
    source_port: u16,
    dest_port: u16,
    protocol: Protocol,
    length: u32,
    label: PacketLabel,
) -> Packet {
    let mut packet = Packet::new(
        source_ip.to_string(),
        dest_ip.to_string(),
        source_port,
        dest_port,
        protocol,
        length,
        Vec::new(),
    );
    packet.label = label;
    packet
}

fn ids_for(world: &World) -> (u64, u64) {
    let mut datacenter_id = None;
    let mut recycle_id = None;
    for building in world.storage.iter() {
        match building.building_type() {
            BuildingType::Datacenter => datacenter_id = Some(building.id()),
            BuildingType::RecycleBin => recycle_id = Some(building.id()),
            _ => {}
        }
    }
    (
        datacenter_id.expect("Datacenter not found"),
        recycle_id.expect("Recycle bin not found"),
    )
}

#[test]
fn completed_packets_reports_once_all_delivered() {
    packet_completion::clear_planned_packets();

    let planned_packets = vec![
        make_packet(
            "10.0.0.1",
            "10.0.0.100",
            1234,
            80,
            Protocol::Tcp,
            128,
            PacketLabel::Correct,
        ),
        make_packet(
            "10.0.0.2",
            "10.0.0.200",
            4321,
            53,
            Protocol::Udp,
            256,
            PacketLabel::Incorrect,
        ),
    ];
    packet_completion::register_planned_core_packets(planned_packets.iter());

    let mut world = World::new();
    world.place_building(Vec2i { x: 0, y: 0 }, BuildingType::Datacenter, 0);
    world.place_building(Vec2i { x: 3, y: 0 }, BuildingType::RecycleBin, 0);

    assert!(packet_completion::completed_packet_count_for_test(&world).is_none());

    let (datacenter_id, recycle_id) = ids_for(&world);

    world
        .storage
        .get_mut(datacenter_id)
        .unwrap()
        .accept(planned_packets[0].clone(), Vec2i { x: 0, y: 0 });
    assert!(packet_completion::completed_packet_count_for_test(&world).is_none());

    world
        .storage
        .get_mut(recycle_id)
        .unwrap()
        .accept(planned_packets[1].clone(), Vec2i { x: 0, y: 0 });

    let completed = packet_completion::completed_packet_count_for_test(&world)
        .expect("Expected completion result after all packets delivered");
    assert_eq!(completed, 2);
}

#[test]
fn clear_planned_packets_resets_state() {
    packet_completion::register_planned_core_packets(
        [make_packet(
            "10.0.0.1",
            "10.0.0.100",
            1234,
            80,
            Protocol::Tcp,
            128,
            PacketLabel::Correct,
        )]
        .iter(),
    );
    packet_completion::clear_planned_packets();

    let world = World::new();
    assert!(packet_completion::completed_packet_count_for_test(&world).is_none());
}
