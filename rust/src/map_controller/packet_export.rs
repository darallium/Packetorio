use godot::prelude::*;

use super::World;
use crate::core::building::BuildingType;
use crate::core::dto::BuildingId;
use crate::core::packet::{Packet as CorePacket, PacketLabel, Protocol};

#[derive(Debug, Clone, PartialEq)]
pub struct PacketView {
    pub building_id: BuildingId,
    pub source_ip: String,
    pub dest_ip: String,
    pub source_port: u16,
    pub dest_port: u16,
    pub protocol: Protocol,
    pub length: u32,
    pub label: PacketLabel,
    pub progress: f32,
    pub payload: String,
}

impl PacketView {
    pub fn from_packet(packet: &CorePacket, building_id: BuildingId) -> Self {
        Self {
            building_id,
            source_ip: packet.source_ip.clone(),
            dest_ip: packet.dest_ip.clone(),
            source_port: packet.source_port,
            dest_port: packet.dest_port,
            protocol: packet.protocol.clone(),
            length: packet.length,
            label: packet.label,
            progress: packet.progress,
            payload: packet.payload_to_string(),
        }
    }

    pub fn into_dictionary(self) -> Dictionary {
        let mut dict = Dictionary::new();
        dict.set("building_id", self.building_id.to_variant());
        dict.set("source_ip", self.source_ip.to_variant());
        dict.set("dest_ip", self.dest_ip.to_variant());
        dict.set("source_port", (self.source_port as i64).to_variant());
        dict.set("dest_port", (self.dest_port as i64).to_variant());
        dict.set("protocol", (self.protocol as i32).to_variant());
        dict.set("length", (self.length as i64).to_variant());
        dict.set("label", self.label.to_raw().to_variant());
        dict.set("progress", self.progress.to_variant());
        dict.set("payload", self.payload.to_variant());
        dict
    }

    pub fn into_variant(self) -> Variant {
        self.into_dictionary().to_variant()
    }
}

pub(super) fn datacenter_packets(world: &World) -> VariantArray {
    world
        .storage
        .iter()
        .filter(|building| building.building_type() == BuildingType::Datacenter)
        .flat_map(|building| {
            let building_id = building.id();
            building
                .get_packets()
                .into_iter()
                .map(move |packet| PacketView::from_packet(&packet, building_id))
        })
        .map(PacketView::into_variant)
        .collect::<VariantArray>()
}

pub(super) fn recyclebin_packets(world: &World) -> VariantArray {
    world
        .storage
        .iter()
        .filter(|building| building.building_type() == BuildingType::RecycleBin)
        .flat_map(|building| {
            let building_id = building.id();
            building
                .get_packets()
                .into_iter()
                .map(move |packet| PacketView::from_packet(&packet, building_id))
        })
        .map(PacketView::into_variant)
        .collect::<VariantArray>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::building::BuildingType;
    use crate::core::dto::Vec2i;

        fn sample_packet() -> CorePacket {
        CorePacket::new(
            "10.0.0.1".to_string(),
            "10.0.0.2".to_string(),
            1000,
            2000,
            Protocol::Tcp,
            1500,
            b"payload".to_vec(),
        )
    }

            fn inject_packet(world: &mut World, building_id: BuildingId, packet: CorePacket) {
        if let Some(building) = world.storage.get_mut(building_id) {
            let _ = building.accept(packet, Vec2i {x:0, y:0});
        }
    }

    #[test]
    #[cfg(not(test))]
    fn test_get_datacenter_packets() {
        let mut world = World::new();
        world.place_building(Vec2i {x:0, y:0}, BuildingType::Datacenter, 0);

        let datacenter_id = world
            .storage
            .iter()
            .find(|b| b.building_type() == BuildingType::Datacenter)
            .unwrap()
            .id();

        let mut packet = sample_packet();
        packet.label = PacketLabel::Correct;
        packet.progress = 1.0;

        inject_packet(&mut world, datacenter_id, packet);

        let exported = datacenter_packets(&world);
        assert_eq!(exported.len(), 1);

        let dict = exported
            .get(0)
            .expect("Entry must has Elements")
            .try_to::<Dictionary>()
            .expect("Entry should be a Dictionary");

                assert_eq!(
            dict.get("building_id").unwrap().try_to::<u64>().unwrap(),
            datacenter_id
        );
        assert_eq!(
            dict.get("source_ip").unwrap().to_string(),
            "10.0.0.1"
        );
        assert_eq!(
            dict.get("dest_ip").unwrap().to_string(),
            "10.0.0.2"
        );
        assert!((dict.get("progress").unwrap().try_to::<f32>().unwrap() - 1.0).abs() < f32::EPSILON);
        assert_eq!(
            dict.get("payload_string").unwrap().to_string(),
            "payload"
        );
    }

    #[test]
    #[cfg(not(test))]
    fn test_get_recyclebin_packets() {
        let mut world = World::new();
        world.place_building(Vec2i {x:0,y:0}, BuildingType::RecycleBin, 0);

        let recycle_id = world
            .storage
            .iter()
            .find(|b| b.building_type() == BuildingType::RecycleBin)
            .unwrap()
            .id();

        let mut packet = sample_packet();
        packet.label = PacketLabel::Incorrect;

        inject_packet(&mut world, recycle_id, packet);

        let exported = recyclebin_packets(&world);
        assert_eq!(exported.len(), 1);

        let dict = exported
            .get(0)
            .expect("Entry must has Elements")
            .try_to::<Dictionary>()
            .expect("Entry should be a Dictionary");

        assert_eq!(
            dict.get("building_id").unwrap().try_to::<u64>().unwrap(),
            recycle_id
        );
        assert_eq!(
            dict.get("label").unwrap().try_to::<i64>().unwrap(),
            PacketLabel::Incorrect.to_raw()
        );
    }
}


