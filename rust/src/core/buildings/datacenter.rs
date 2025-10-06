use godot::obj::Gd;

use crate::core::building::{Building, BuildingAction, BuildingType};
use crate::core::dto::{BuildingId, Vec2i};
use crate::core::packet::{Packet, PacketLabel};
use crate::packet::Traffic;

pub struct Datacenter {
    id: BuildingId,
    pos: Vec2i,
    rot: i32,
    packets: Vec<Packet>,
    traffic: Option<Gd<Traffic>>,
}

impl Datacenter {
    pub fn new(id: BuildingId, pos: Vec2i, rot: i32) -> Self {
        Self {
            id,
            pos,
            rot,
            packets: Vec::new(),
            traffic: None,
        }
    }

    pub fn set_traffic(&mut self, traffic: Gd<Traffic>) {
        self.traffic = Some(traffic);
    }
}

impl Building for Datacenter {
    fn id(&self) -> BuildingId {
        self.id
    }
    fn position(&self) -> Vec2i {
        self.pos
    }
    fn rotation(&self) -> i32 {
        self.rot
    }
    fn building_type(&self) -> BuildingType {
        BuildingType::Datacenter
    }
    fn get_size(&self) -> Vec2i {
        Vec2i { x: 2, y: 2 }
    }
    fn get_output_poses(&self) -> Vec<Vec2i> {
        vec![]
    }
    fn get_input_poses(&self) -> Vec<Vec2i> {
        let size = self.get_size();
        let mut poses = Vec::new();

        for x_offset in 0..size.x {
            poses.push(Vec2i {
                x: self.pos.x + x_offset,
                y: self.pos.y - 1,
            });
            poses.push(Vec2i {
                x: self.pos.x + x_offset,
                y: self.pos.y + size.y,
            });
        }

        for y_offset in 0..size.y {
            poses.push(Vec2i {
                x: self.pos.x - 1,
                y: self.pos.y + y_offset,
            });
            poses.push(Vec2i {
                x: self.pos.x + size.x,
                y: self.pos.y + y_offset,
            });
        }

        poses
    }
    fn update(&mut self, _delta: f32) {}
    fn can_offload(&self) -> bool {
        false
    }
    fn can_accept(&self, _packet: &Packet, _source_pos: Vec2i) -> bool {
        true
    }
    fn offload(&mut self) -> Packet {
        panic!("Cannot offload from datacenter")
    }
    fn accept(&mut self, packet: Packet, _source_pos: Vec2i) -> BuildingAction {
        let outcome = match packet.label {
            PacketLabel::Correct => BuildingAction::AddScore(10),
            PacketLabel::Incorrect => BuildingAction::SubScore(10),
            PacketLabel::Unknown => BuildingAction::None,
        };
        self.packets.push(packet);
        outcome
    }
    fn get_progress(&self) -> f32 {
        0.0
    }
    fn get_packets(&self) -> Vec<Packet> {
        self.packets.clone()
    }
    fn get_packet_progresses(&self) -> Vec<f32> {
        vec![]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
