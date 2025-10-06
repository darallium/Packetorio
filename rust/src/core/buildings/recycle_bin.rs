use crate::core::building::{Building, BuildingAction, BuildingType};
use crate::core::dto::{BuildingId, Vec2i};
use crate::core::packet::Packet;

pub struct RecycleBin {
    id: BuildingId,
    pos: Vec2i,
    rot: i32,
    packets: Vec<Packet>,
}

impl RecycleBin {
    pub fn new(id: BuildingId, pos: Vec2i, rot: i32) -> Self {
        Self {
            id,
            pos,
            rot,
            packets: Vec::new(),
        }
    }
}

impl Building for RecycleBin {
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
        BuildingType::RecycleBin
    }
    fn get_size(&self) -> Vec2i {
        Vec2i { x: 1, y: 1 }
    }
    fn get_output_poses(&self) -> Vec<Vec2i> {
        vec![]
    }
    fn get_input_poses(&self) -> Vec<Vec2i> {
        vec![
            Vec2i {
                x: self.pos.x,
                y: self.pos.y - 1,
            },
            Vec2i {
                x: self.pos.x + 1,
                y: self.pos.y,
            },
            Vec2i {
                x: self.pos.x,
                y: self.pos.y + 1,
            },
            Vec2i {
                x: self.pos.x - 1,
                y: self.pos.y,
            },
        ]
    }
    fn update(&mut self, _delta: f32) {}
    fn can_offload(&self) -> bool {
        false
    }
    fn can_accept(&self, _packet: &Packet, _source_pos: Vec2i) -> bool {
        true
    }
    fn offload(&mut self) -> Packet {
        panic!("Cannot offload from recycle bin")
    }
    fn accept(&mut self, packet: Packet, _source_pos: Vec2i) -> BuildingAction {
        self.packets.push(packet);
        BuildingAction::None
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
