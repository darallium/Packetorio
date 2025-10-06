use crate::core::building::{Building, BuildingType};
use crate::core::dto::{BuildingId, Vec2i};
use crate::map_controller::World;
use godot::prelude::Vector2;
use std::collections::{HashMap, HashSet};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_world() {
        let world = World::new();
        assert_eq!(world.storage.iter().count(), 0);
    }

    #[test]
    fn test_place_and_get_building() {
        let mut world = World::new();
        let pos = Vec2i { x: 1, y: 2 };

        world.place_building(pos, BuildingType::Internet, 0);

        let buildings: Vec<_> = world.storage.iter().collect();
        assert_eq!(buildings.len(), 1);
        let building = buildings.first().unwrap();
        assert_eq!(building.position(), pos);
        assert_eq!(building.building_type(), BuildingType::Internet);
    }

    #[test]
    fn test_place_multiple_buildings() {
        let mut world = World::new();
        world.place_building(Vec2i { x: 0, y: 0 }, BuildingType::Internet, 0); // 2x2
        world.place_building(Vec2i { x: 3, y: 0 }, BuildingType::Datacenter, 0); // 2x2
        world.place_building(Vec2i { x: 6, y: 0 }, BuildingType::RecycleBin, 0); // 1x1

        assert_eq!(world.storage.iter().count(), 3);
    }

    #[test]
    fn test_remove_building() {
        let mut world = World::new();
        let pos = Vec2i { x: 3, y: 3 };
        world.place_building(pos, BuildingType::Internet, 0);
        assert_eq!(world.storage.iter().count(), 1);

        world.remove_building(&pos);
        assert_eq!(world.storage.iter().count(), 0);
    }

    // Note: move_building and rotate_building are not directly implemented on World.
    // These operations are handled by place_building and remove_building.
    // A higher-level controller would implement move/rotate.
    // We will test the underlying logic.

    #[test]
    fn test_building_placement_and_removal_for_move() {
        let mut world = World::new();
        let old_pos = Vec2i { x: 4, y: 4 };
        let new_pos = Vec2i { x: 5, y: 5 };

        // Simulate a move operation
        world.place_building(old_pos, BuildingType::Datacenter, 0);
        // let building_id = world.storage.iter().next().unwrap().id();

        world.remove_building(&old_pos);
        world.place_building(new_pos, BuildingType::Datacenter, 0);

        assert_eq!(world.storage.iter().count(), 1);
        let new_building = world.storage.iter().next().unwrap();
        assert_eq!(new_building.position(), new_pos);
        // Note: The ID will be different in this simple simulation.
        // A real move implementation would preserve the ID.
    }

    // load_map is on MapController, which we are not testing directly.
    // We can test the logic by creating a similar function for World if needed,
    // or by testing the building blocks.
    // For now, we focus on the core World logic.
}

use crate::core::dto::WorldEvent;
use crate::core::packet::{Packet, PacketLabel, Protocol};

fn create_test_packet() -> Packet {
    let mut packet = Packet::new(
        "192.168.1.1".to_string(),
        "192.168.1.2".to_string(),
        12345,
        80,
        Protocol::Tcp,
        64,
        vec![],
    );
    packet.label = PacketLabel::Correct;
    packet.payload = b"payload data".to_vec();
    packet
}

// Helper function to get a building by its position from the map
fn get_building_id_by_pos(world: &World, pos: Vec2i) -> Option<u64> {
    world.get_building_id_at(&pos)
}

#[test]
fn test_update_packet_movement_and_events() {
    let mut world = World::new();
    let internet_pos = Vec2i { x: 0, y: 0 }; // 2x2
    let conveyor_pos = Vec2i { x: 2, y: 0 }; // 1x1
    let datacenter_pos = Vec2i { x: 3, y: 0 }; // 2x2

    // 1. Setup
    world.place_building(internet_pos, BuildingType::Internet, 0); // Faces right
    world.place_building(conveyor_pos, BuildingType::Conveyor, 0); // Faces right
    world.place_building(datacenter_pos, BuildingType::Datacenter, 0); // Faces up

    let internet_id = get_building_id_by_pos(&world, internet_pos).unwrap();
    let conveyor_id = get_building_id_by_pos(&world, conveyor_pos).unwrap();
    let datacenter_id = get_building_id_by_pos(&world, datacenter_pos).unwrap();

    let test_packet = create_test_packet();
    if let Some(internet) = world
        .storage
        .get_mut(internet_id)
        .unwrap()
        .as_any_mut()
        .downcast_mut::<crate::core::buildings::internet::Internet>()
    {
        internet.add_packet(test_packet);
    }
    world.drain_events(); // Clear placement events

    // 2. Test Internet -> Conveyor transfer
    world.update(0.0);
    let events = world.drain_events();
    assert_eq!(events.len(), 1);
    match &events[0] {
        WorldEvent::PacketMoved { from_id, to_id, .. } => {
            assert_eq!(*from_id, internet_id);
            assert_eq!(*to_id, conveyor_id);
        }
        _ => panic!("Expected PacketMoved event"),
    }
    assert_eq!(
        world.storage.get(conveyor_id).unwrap().get_packets().len(),
        1
    );
    assert_eq!(
        world.storage.get(internet_id).unwrap().get_packets().len(),
        0
    );

    // 3. Test progress on Conveyor
    world.update(0.999); // Almost complete progress
    assert!(
        !world.storage.get(conveyor_id).unwrap().can_offload(),
        "Should not be ready to offload yet"
    );

    world.drain_events(); // Clear progress update events
    world.update(0.001); // Complete the progress, which should also trigger the transfer in the same update cycle

    // 4. Test Conveyor -> Datacenter transfer
    let events = world.drain_events();
    let packet_moved = events
        .iter()
        .find(|e| matches!(e, WorldEvent::PacketMoved { .. }));
    assert!(
        packet_moved.is_some(),
        "Expected PacketMoved event after progress completion"
    );

    if let Some(WorldEvent::PacketMoved { from_id, to_id, .. }) = packet_moved {
        assert_eq!(*from_id, conveyor_id);
        assert_eq!(*to_id, datacenter_id);
    }
    assert_eq!(
        world
            .storage
            .get(datacenter_id)
            .unwrap()
            .get_packets()
            .len(),
        1
    );
    assert_eq!(
        world.storage.get(conveyor_id).unwrap().get_packets().len(),
        0
    );
}

#[test]
fn test_filter_routing_round_robin_and_mismatch() {
    use crate::core::buildings::filters::ip_filter::{IpFilter, IpFilterConfig, IpFilterDirection};

    let mut world = World::new();

    let filter_pos = Vec2i { x: 1, y: 1 };
    world.place_building(filter_pos, BuildingType::IpFilter, 0);
    let front_bin_pos = Vec2i { x: 1, y: 0 };
    let left_bin_pos = Vec2i { x: 0, y: 1 };
    let right_bin_pos = Vec2i { x: 2, y: 1 };

    world.place_building(front_bin_pos, BuildingType::RecycleBin, 0);
    world.place_building(left_bin_pos, BuildingType::RecycleBin, 0);
    world.place_building(right_bin_pos, BuildingType::RecycleBin, 0);

    let filter_id = get_building_id_by_pos(&world, filter_pos).unwrap();
    let front_id = get_building_id_by_pos(&world, front_bin_pos).unwrap();
    let left_id = get_building_id_by_pos(&world, left_bin_pos).unwrap();
    let right_id = get_building_id_by_pos(&world, right_bin_pos).unwrap();

    {
        let filter = world
            .storage
            .get_mut(filter_id)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<IpFilter>()
            .unwrap();
        filter.set_config(IpFilterConfig {
            target_ip: "10.0.0.1".to_string(),
            direction: IpFilterDirection::Source,
        });
    }

    let mut delivery_counts: HashMap<BuildingId, usize> = HashMap::new();
    let source_pos = Vec2i {
        x: filter_pos.x,
        y: filter_pos.y + 1,
    };

    // Inject two matching packets; they should alternate between left and right outputs.
    for i in 0..2 {
        let mut packet = create_test_packet();
        packet.source_ip = "10.0.0.1".to_string();
        {
            let filter = world
                .storage
                .get_mut(filter_id)
                .unwrap()
                .as_any_mut()
                .downcast_mut::<IpFilter>()
                .unwrap();
            filter.accept(packet, source_pos);
        }

        world.update(0.0);
        let events = world.drain_events();
        let moved = events
            .into_iter()
            .find_map(|event| match event {
                WorldEvent::PacketMoved { to_id, .. } => Some(to_id),
                _ => None,
            })
            .expect("expected a packet move event");

        *delivery_counts.entry(moved).or_insert(0) += 1;

        // Ensure the filter buffer is clear before next iteration.
        let filter_packets = world.storage.get(filter_id).unwrap().get_packets();
        assert!(
            filter_packets.is_empty(),
            "filter should have offloaded packet {i}"
        );
    }

    assert_eq!(delivery_counts.get(&left_id), Some(&1));
    assert_eq!(delivery_counts.get(&right_id), Some(&1));

    // Inject a non-matching packet; it should travel to the front output.
    {
        let mut packet = create_test_packet();
        packet.source_ip = "192.168.0.99".to_string();
        let filter = world
            .storage
            .get_mut(filter_id)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<IpFilter>()
            .unwrap();
        filter.accept(packet, source_pos);
    }

    world.update(0.0);
    let events = world.drain_events();
    let moved = events
        .into_iter()
        .find_map(|event| match event {
            WorldEvent::PacketMoved { to_id, .. } => Some(to_id),
            _ => None,
        })
        .expect("expected a packet move event for mismatch");

    assert_eq!(moved, front_id);
}

#[test]
fn test_filter_routing_with_map_update() {
    use crate::core::buildings::filters::ip_filter::{IpFilter, IpFilterConfig, IpFilterDirection};

    let mut world = World::new();

    let conveyor_pos = Vec2i { x: 2, y: 2 };
    let filter_pos = Vec2i { x: 2, y: 1 };
    let front_bin_pos = Vec2i { x: 2, y: 0 };
    let left_bin_pos = Vec2i { x: 1, y: 1 };
    let right_bin_pos = Vec2i { x: 3, y: 1 };

    world.place_building(conveyor_pos, BuildingType::Conveyor, 3); // Facing north toward the filter
    world.place_building(filter_pos, BuildingType::IpFilter, 0);
    world.place_building(front_bin_pos, BuildingType::RecycleBin, 0);
    world.place_building(left_bin_pos, BuildingType::RecycleBin, 0);
    world.place_building(right_bin_pos, BuildingType::RecycleBin, 0);

    let conveyor_id = get_building_id_by_pos(&world, conveyor_pos).unwrap();
    let filter_id = get_building_id_by_pos(&world, filter_pos).unwrap();
    let front_id = get_building_id_by_pos(&world, front_bin_pos).unwrap();
    let left_id = get_building_id_by_pos(&world, left_bin_pos).unwrap();
    let right_id = get_building_id_by_pos(&world, right_bin_pos).unwrap();

    {
        let filter = world
            .storage
            .get_mut(filter_id)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<IpFilter>()
            .unwrap();
        filter.set_config(IpFilterConfig {
            target_ip: "10.0.0.1".to_string(),
            direction: IpFilterDirection::Source,
        });
    }

    let source_pos = Vec2i {
        x: conveyor_pos.x,
        y: conveyor_pos.y + 1,
    };

    let mut deliver_packet = |packet: Packet| -> BuildingId {
        world
            .storage
            .get_mut(conveyor_id)
            .unwrap()
            .accept(packet, source_pos);

        world.update(1.0);
        world.drain_events();
        world.update(0.0);

        let events = world.drain_events();
        events
            .into_iter()
            .find_map(|event| match event {
                WorldEvent::PacketMoved { from_id, to_id, .. } if from_id == filter_id => {
                    Some(to_id)
                }
                _ => None,
            })
            .expect("filter should forward packet")
    };

    let mut seen_sides = HashSet::new();
    for _ in 0..2 {
        let mut packet = create_test_packet();
        packet.source_ip = "10.0.0.1".to_string();
        let dest_id = deliver_packet(packet);
        assert!(dest_id == left_id || dest_id == right_id);
        seen_sides.insert(dest_id);
    }
    assert_eq!(
        seen_sides.len(),
        2,
        "filtered packets should alternate between side outputs"
    );

    let mut packet = create_test_packet();
    packet.source_ip = "172.16.0.5".to_string();
    let dest_id = deliver_packet(packet);
    assert_eq!(dest_id, front_id);
}

#[test]
fn test_rebuild_connections_simple() {
    let mut world = World::new();
    let pos1 = Vec2i { x: 0, y: 0 };
    let pos2 = Vec2i { x: 1, y: 0 };

    // Place conveyor that outputs to the adjacent tile and a receiver next to it
    world.place_building(pos1, BuildingType::Conveyor, 0); // Outputs to the east
    world.place_building(pos2, BuildingType::Conveyor, 0); // Accepts from the west

    let id1 = get_building_id_by_pos(&world, pos1).unwrap();
    let id2 = get_building_id_by_pos(&world, pos2).unwrap();

    // Check connections
    let edges_from_1 = world
        .get_output_connections(id1)
        .expect("expected connection");
    assert_eq!(edges_from_1.len(), 1);
    assert_eq!(edges_from_1[0].to_id, id2);

    assert!(world.get_output_connections(id2).is_none());

    // Remove one and check if connection is gone
    world.remove_building(&pos2);
    assert_eq!(world.get_output_connections(id1), None);
}

#[test]
fn test_conveyor_side_entry_position_steps() {
    use crate::core::buildings::conveyor::EntrySide;
    let tile_pos = Vec2i { x: 2, y: 2 };
    let tile_size = 16.0;

    let pos_left_entry = crate::map_controller::conveyor_packet_position(
        tile_pos,
        0,
        EntrySide::Left,
        0.0,
        tile_size,
    );
    assert_eq!(pos_left_entry, Vector2::new(40.0, 48.0));

    let pos_left_mid = crate::map_controller::conveyor_packet_position(
        tile_pos,
        0,
        EntrySide::Left,
        0.5,
        tile_size,
    );
    assert_eq!(pos_left_mid, Vector2::new(40.0, 40.0));

    let pos_left_second_leg = crate::map_controller::conveyor_packet_position(
        tile_pos,
        0,
        EntrySide::Left,
        0.75,
        tile_size,
    );
    assert_eq!(pos_left_second_leg, Vector2::new(44.0, 40.0));

    let pos_right_entry = crate::map_controller::conveyor_packet_position(
        tile_pos,
        0,
        EntrySide::Right,
        0.0,
        tile_size,
    );
    assert_eq!(pos_right_entry, Vector2::new(40.0, 32.0));

    let pos_right_mid = crate::map_controller::conveyor_packet_position(
        tile_pos,
        0,
        EntrySide::Right,
        0.5,
        tile_size,
    );
    assert_eq!(pos_right_mid, Vector2::new(40.0, 40.0));

    let pos_right_second_leg = crate::map_controller::conveyor_packet_position(
        tile_pos,
        0,
        EntrySide::Right,
        0.75,
        tile_size,
    );
    assert_eq!(pos_right_second_leg, Vector2::new(44.0, 40.0));
}
#[test]
fn test_world_reset_logic() {
    let mut world = World::new();
    world.place_building(Vec2i { x: 1, y: 1 }, BuildingType::Internet, 0);
    assert_eq!(world.storage.iter().count(), 1);

    world = World::new();
    assert_eq!(world.storage.iter().count(), 0);
}

#[test]
fn test_event_generation() {
    let mut world = World::new();
    let pos = Vec2i { x: 1, y: 1 };

    // BuildingPlaced event
    world.place_building(pos, BuildingType::Internet, 0);
    let events = world.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], WorldEvent::BuildingPlaced { .. }));

    // BuildingRemoved event
    world.remove_building(&pos);
    let events = world.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], WorldEvent::BuildingRemoved { .. }));
}

#[test]
fn test_place_multiblock_building() {
    let mut world = World::new();
    let pos = Vec2i { x: 2, y: 2 };
    world.place_building(pos, BuildingType::Datacenter, 0);

    let datacenter_id = get_building_id_by_pos(&world, pos).unwrap();

    // Check all 4 tiles are occupied by the same building
    assert_eq!(
        get_building_id_by_pos(&world, Vec2i { x: 2, y: 2 }),
        Some(datacenter_id)
    );
    assert_eq!(
        get_building_id_by_pos(&world, Vec2i { x: 3, y: 2 }),
        Some(datacenter_id)
    );
    assert_eq!(
        get_building_id_by_pos(&world, Vec2i { x: 2, y: 3 }),
        Some(datacenter_id)
    );
    assert_eq!(
        get_building_id_by_pos(&world, Vec2i { x: 3, y: 3 }),
        Some(datacenter_id)
    );

    // Check collision
    world.place_building(Vec2i { x: 3, y: 3 }, BuildingType::Conveyor, 0);
    assert_eq!(
        world.storage.iter().count(),
        1,
        "Should not be able to place on occupied tile"
    );
}

#[test]
fn test_remove_multiblock_building() {
    let mut world = World::new();
    let pos = Vec2i { x: 2, y: 2 };
    world.place_building(pos, BuildingType::Datacenter, 0);
    assert_eq!(world.storage.iter().count(), 1);

    world.remove_building(&Vec2i { x: 3, y: 3 }); // Remove by clicking any part of the building
    assert_eq!(world.storage.iter().count(), 0);

    // Check all 4 tiles are now empty
    assert_eq!(get_building_id_by_pos(&world, Vec2i { x: 2, y: 2 }), None);
    assert_eq!(get_building_id_by_pos(&world, Vec2i { x: 3, y: 2 }), None);
    assert_eq!(get_building_id_by_pos(&world, Vec2i { x: 2, y: 3 }), None);
    assert_eq!(get_building_id_by_pos(&world, Vec2i { x: 3, y: 3 }), None);
}

#[test]
fn test_place_ip_filter_with_config() {
    use crate::core::buildings::filters::ip_filter::{IpFilterConfig, IpFilterDirection};

    let mut world = World::new();
    let pos = Vec2i { x: 5, y: 5 };

    let config = IpFilterConfig {
        target_ip: "192.168.1.100".to_string(),
        direction: IpFilterDirection::Source,
    };

    world.place_ip_filter_with_config(pos, 0, config);

    let filter_id = get_building_id_by_pos(&world, pos).unwrap();
    let building = world.storage.get(filter_id).unwrap();

    assert_eq!(building.building_type(), BuildingType::IpFilter);
    assert_eq!(building.position(), pos);

    // テストパケットでフィルタリング動作確認
    let filter = building
        .as_any()
        .downcast_ref::<crate::core::buildings::filters::ip_filter::IpFilter>()
        .unwrap();

    let mut test_packet = create_test_packet();
    test_packet.source_ip = "192.168.1.100".to_string();
    assert!(filter.filter(&test_packet));

    test_packet.source_ip = "10.0.0.1".to_string();
    assert!(!filter.filter(&test_packet));
}

#[test]
fn test_ip_filter_set_config_via_building_storage() {
    use crate::core::buildings::filters::ip_filter::{IpFilterConfig, IpFilterDirection};

    let mut world = World::new();
    let pos = Vec2i { x: 3, y: 3 };

    // 設定なしでIPフィルターを設置
    world.place_building(pos, BuildingType::IpFilter, 0);

    let filter_id = get_building_id_by_pos(&world, pos).unwrap();

    // 初期状態では何も通さない
    {
        let building = world.storage.get(filter_id).unwrap();
        let filter = building
            .as_any()
            .downcast_ref::<crate::core::buildings::filters::ip_filter::IpFilter>()
            .unwrap();
        let test_packet = create_test_packet();
        assert!(!filter.filter(&test_packet));
    }

    // set_configを使って設定を追加
    {
        let building = world.storage.get_mut(filter_id).unwrap();
        let filter = building
            .as_any_mut()
            .downcast_mut::<crate::core::buildings::filters::ip_filter::IpFilter>()
            .unwrap();

        let config = IpFilterConfig {
            target_ip: "192.168.1.1".to_string(),
            direction: IpFilterDirection::Source,
        };
        filter.set_config(config);
    }

    // 設定後はフィルタリングが動作
    {
        let building = world.storage.get(filter_id).unwrap();
        let filter = building
            .as_any()
            .downcast_ref::<crate::core::buildings::filters::ip_filter::IpFilter>()
            .unwrap();
        let test_packet = create_test_packet(); // source_ip = "192.168.1.1"
        assert!(filter.filter(&test_packet));
    }
}

#[test]
fn test_place_length_filter_with_config() {
    use crate::core::buildings::filters::length_filter::{
        LengthFilterConfig, LengthFilterDirection,
    };

    let mut world = World::new();
    let pos = Vec2i { x: 5, y: 5 };

    let config = LengthFilterConfig {
        threshold: 512,
        direction: LengthFilterDirection::LessThan,
    };

    world.place_length_filter_with_config(pos, 0, config);

    let filter_id = get_building_id_by_pos(&world, pos).unwrap();
    let building = world.storage.get(filter_id).unwrap();

    assert_eq!(building.building_type(), BuildingType::LengthFilter);
    assert_eq!(building.position(), pos);

    // テストパケットでフィルタリング動作確認
    let filter = building
        .as_any()
        .downcast_ref::<crate::core::buildings::filters::length_filter::LengthFilter>()
        .unwrap();

    let mut test_packet = create_test_packet();
    test_packet.length = 256; // 512未満
    assert!(filter.filter(&test_packet));

    test_packet.length = 1024; // 512以上
    assert!(!filter.filter(&test_packet));
}

#[test]
fn test_length_filter_set_config_via_building_storage() {
    use crate::core::buildings::filters::length_filter::{
        LengthFilterConfig, LengthFilterDirection,
    };

    let mut world = World::new();
    let pos = Vec2i { x: 3, y: 3 };

    // 設定なしでLengthフィルターを設置
    world.place_building(pos, BuildingType::LengthFilter, 0);

    let filter_id = get_building_id_by_pos(&world, pos).unwrap();

    // 初期状態では何も通さない
    {
        let building = world.storage.get(filter_id).unwrap();
        let filter = building
            .as_any()
            .downcast_ref::<crate::core::buildings::filters::length_filter::LengthFilter>()
            .unwrap();
        let test_packet = create_test_packet();
        assert!(!filter.filter(&test_packet));
    }

    // set_configを使って設定を追加
    {
        let building = world.storage.get_mut(filter_id).unwrap();
        let filter = building
            .as_any_mut()
            .downcast_mut::<crate::core::buildings::filters::length_filter::LengthFilter>()
            .unwrap();

        let config = LengthFilterConfig {
            threshold: 64,
            direction: LengthFilterDirection::Exact,
        };
        filter.set_config(config);
    }

    // 設定後はフィルタリングが動作
    {
        let building = world.storage.get(filter_id).unwrap();
        let filter = building
            .as_any()
            .downcast_ref::<crate::core::buildings::filters::length_filter::LengthFilter>()
            .unwrap();
        let mut test_packet = create_test_packet();
        test_packet.length = 64; // 閾値と同じ
        assert!(filter.filter(&test_packet));
    }
}

#[test]
fn test_place_protocol_filter_with_config() {
    use crate::core::buildings::filters::protocol_filter::ProtocolFilterConfig;
    use crate::core::packet::Protocol;

    let mut world = World::new();
    let pos = Vec2i { x: 7, y: 7 };

    let config = ProtocolFilterConfig {
        protocol: Protocol::Tcp,
    };

    world.place_protocol_filter_with_config(pos, 0, config);

    let filter_id = get_building_id_by_pos(&world, pos).unwrap();
    let building = world.storage.get(filter_id).unwrap();

    assert_eq!(building.building_type(), BuildingType::ProtocolFilter);
    assert_eq!(building.position(), pos);

    // テストパケットでフィルタリング動作確認
    let filter = building
        .as_any()
        .downcast_ref::<crate::core::buildings::filters::protocol_filter::ProtocolFilter>()
        .unwrap();

    let test_packet = create_test_packet(); // protocol = Tcp
    assert!(filter.filter(&test_packet));

    let mut udp_packet = create_test_packet();
    udp_packet.protocol = Protocol::Udp;
    assert!(!filter.filter(&udp_packet));
}

#[test]
fn test_protocol_filter_set_config_via_building_storage() {
    use crate::core::buildings::filters::protocol_filter::ProtocolFilterConfig;
    use crate::core::packet::Protocol;

    let mut world = World::new();
    let pos = Vec2i { x: 4, y: 4 };

    // 設定なしでプロトコルフィルターを設置
    world.place_building(pos, BuildingType::ProtocolFilter, 0);

    let filter_id = get_building_id_by_pos(&world, pos).unwrap();

    // 初期状態では何も通さない
    {
        let building = world.storage.get(filter_id).unwrap();
        let filter = building
            .as_any()
            .downcast_ref::<crate::core::buildings::filters::protocol_filter::ProtocolFilter>()
            .unwrap();
        let test_packet = create_test_packet();
        assert!(!filter.filter(&test_packet));
    }

    // set_configを使って設定を追加
    {
        let building = world.storage.get_mut(filter_id).unwrap();
        let filter = building
            .as_any_mut()
            .downcast_mut::<crate::core::buildings::filters::protocol_filter::ProtocolFilter>()
            .unwrap();

        let config = ProtocolFilterConfig {
            protocol: Protocol::Tcp,
        };
        filter.set_config(config);
    }

    // 設定後はフィルタリングが動作
    {
        let building = world.storage.get(filter_id).unwrap();
        let filter = building
            .as_any()
            .downcast_ref::<crate::core::buildings::filters::protocol_filter::ProtocolFilter>()
            .unwrap();
        let test_packet = create_test_packet(); // protocol = Tcp
        assert!(filter.filter(&test_packet));
    }
}

#[test]
fn test_place_port_filter_with_config() {
    use crate::core::buildings::filters::port_filter::{PortFilterConfig, PortFilterDirection};

    let mut world = World::new();
    let pos = Vec2i { x: 5, y: 5 };

    let config = PortFilterConfig {
        target_port: 443,
        direction: PortFilterDirection::Destination,
    };

    world.place_port_filter_with_config(pos, 0, config);

    let filter_id = get_building_id_by_pos(&world, pos).unwrap();
    let building = world.storage.get(filter_id).unwrap();

    assert_eq!(building.building_type(), BuildingType::PortFilter);
    assert_eq!(building.position(), pos);

    // テストパケットでフィルタリング動作確認
    let filter = building
        .as_any()
        .downcast_ref::<crate::core::buildings::filters::port_filter::PortFilter>()
        .unwrap();

    let mut test_packet = create_test_packet();
    test_packet.dest_port = 443;
    assert!(filter.filter(&test_packet));

    test_packet.dest_port = 80;
    assert!(!filter.filter(&test_packet));
}

#[test]
fn test_port_filter_set_config_via_building_storage() {
    use crate::core::buildings::filters::port_filter::{PortFilterConfig, PortFilterDirection};

    let mut world = World::new();
    let pos = Vec2i { x: 3, y: 3 };

    // 設定なしでPortフィルターを設置
    world.place_building(pos, BuildingType::PortFilter, 0);

    let filter_id = get_building_id_by_pos(&world, pos).unwrap();

    // 初期状態では設定がないので何も通さない
    {
        let building = world.storage.get(filter_id).unwrap();
        let filter = building
            .as_any()
            .downcast_ref::<crate::core::buildings::filters::port_filter::PortFilter>()
            .unwrap();
        let test_packet = create_test_packet();
        assert!(!filter.filter(&test_packet));
    }

    // set_configを使って設定を追加
    {
        let building = world.storage.get_mut(filter_id).unwrap();
        let filter = building
            .as_any_mut()
            .downcast_mut::<crate::core::buildings::filters::port_filter::PortFilter>()
            .unwrap();

        let config = PortFilterConfig {
            target_port: 12345,
            direction: PortFilterDirection::Source,
        };
        filter.set_config(config);
    }

    // 設定後は新しいconfigでフィルタリングが動作
    {
        let building = world.storage.get(filter_id).unwrap();
        let filter = building
            .as_any()
            .downcast_ref::<crate::core::buildings::filters::port_filter::PortFilter>()
            .unwrap();
        let test_packet = create_test_packet(); // source_port = 12345
        assert!(filter.filter(&test_packet));

        let mut test_packet2 = create_test_packet();
        test_packet2.source_port = 54321;
        assert!(!filter.filter(&test_packet2));
    }
}

#[test]
fn test_place_content_filter_with_config() {
    use crate::core::buildings::filters::content_filter::ContentFilterConfig;

    let mut world = World::new();
    let pos = Vec2i { x: 5, y: 5 };

    let config = ContentFilterConfig {
        pattern: "malicious".to_string(),
    };

    world.place_content_filter_with_config(pos, 0, config);

    let filter_id = get_building_id_by_pos(&world, pos).unwrap();
    let building = world.storage.get(filter_id).unwrap();

    assert_eq!(building.building_type(), BuildingType::ContentFilter);
    assert_eq!(building.position(), pos);

    // テストパケットでフィルタリング動作確認
    let filter = building
        .as_any()
        .downcast_ref::<crate::core::buildings::filters::content_filter::ContentFilter>()
        .unwrap();

    let mut test_packet_match = create_test_packet();
    test_packet_match.payload = b"this is malicious content".to_vec();
    assert!(filter.filter(&test_packet_match));

    let mut test_packet_no_match = create_test_packet();
    test_packet_no_match.payload = b"safe content here".to_vec();
    assert!(!filter.filter(&test_packet_no_match));
}

#[test]
fn test_content_filter_set_config_via_building_storage() {
    use crate::core::buildings::filters::content_filter::ContentFilterConfig;

    let mut world = World::new();
    let pos = Vec2i { x: 3, y: 3 };

    // 設定なしでContentフィルターを設置
    world.place_building(pos, BuildingType::ContentFilter, 0);

    let filter_id = get_building_id_by_pos(&world, pos).unwrap();

    // 初期状態では設定がないので何も通さない
    {
        let building = world.storage.get(filter_id).unwrap();
        let filter = building
            .as_any()
            .downcast_ref::<crate::core::buildings::filters::content_filter::ContentFilter>()
            .unwrap();
        let test_packet = create_test_packet();
        assert!(!filter.filter(&test_packet));
    }

    // set_configを使って設定を追加
    {
        let building = world.storage.get_mut(filter_id).unwrap();
        let filter = building
            .as_any_mut()
            .downcast_mut::<crate::core::buildings::filters::content_filter::ContentFilter>()
            .unwrap();

        let config = ContentFilterConfig {
            pattern: r"p.*load".to_string(), // regex pattern
        };
        filter.set_config(config);
    }

    // 設定後はフィルタリングが動作
    {
        let building = world.storage.get(filter_id).unwrap();
        let filter = building
            .as_any()
            .downcast_ref::<crate::core::buildings::filters::content_filter::ContentFilter>()
            .unwrap();
        let test_packet = create_test_packet(); // payload contains "payload"
        assert!(filter.filter(&test_packet));

        let mut test_packet_no_match = create_test_packet();
        test_packet_no_match.payload = b"different data".to_vec();
        assert!(!filter.filter(&test_packet_no_match));
    }
}
