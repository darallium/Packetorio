use crate::core::building::{Building, BuildingAction, BuildingType};
use crate::core::dto::{BuildingId, Vec2i};
use crate::core::packet::Packet;

pub struct Junction {
    id: BuildingId,
    pos: Vec2i,
    rot: i32,
    buffer: Option<(Packet, Vec2i)>,
}

impl Junction {
    pub fn new(id: BuildingId, pos: Vec2i, rot: i32) -> Self {
        Self {
            id,
            pos,
            rot,
            buffer: None,
        }
    }

    fn neighbor_offsets(&self) -> [(i32, i32); 4] {
        let rotation = self.rot.rem_euclid(4);
        match rotation {
            0 => [(0, -1), (1, 0), (0, 1), (-1, 0)],
            1 => [(1, 0), (0, 1), (-1, 0), (0, -1)],
            2 => [(0, 1), (-1, 0), (0, -1), (1, 0)],
            3 => [(-1, 0), (0, -1), (1, 0), (0, 1)],
            _ => [(0, -1), (1, 0), (0, 1), (-1, 0)],
        }
    }

    pub fn pending_output_pos(&self) -> Option<Vec2i> {
        self.buffer.as_ref().map(|(_, source_pos)| Vec2i {
            x: self.pos.x + (self.pos.x - source_pos.x),
            y: self.pos.y + (self.pos.y - source_pos.y),
        })
    }
}

impl Building for Junction {
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
        BuildingType::Junction
    }
    fn get_size(&self) -> Vec2i {
        Vec2i { x: 1, y: 1 }
    }
    fn get_output_poses(&self) -> Vec<Vec2i> {
        self.neighbor_offsets()
            .into_iter()
            .map(|(dx, dy)| Vec2i {
                x: self.pos.x + dx,
                y: self.pos.y + dy,
            })
            .collect()
    }
    fn get_input_poses(&self) -> Vec<Vec2i> {
        self.get_output_poses()
    }
    fn update(&mut self, _delta: f32) {}
    fn can_offload(&self) -> bool {
        self.buffer.is_some()
    }
    fn can_accept(&self, _packet: &Packet, _source_pos: Vec2i) -> bool {
        self.buffer.is_none()
    }
    fn offload(&mut self) -> Packet {
        self.buffer.take().expect("Offload called without packet").0
    }
    fn accept(&mut self, packet: Packet, source_pos: Vec2i) -> BuildingAction {
        self.buffer = Some((packet, source_pos));
        BuildingAction::None
    }
    fn get_progress(&self) -> f32 {
        0.0
    }
    fn get_packets(&self) -> Vec<Packet> {
        self.buffer.iter().map(|(p, _)| p.clone()).collect()
    }
    fn get_packet_progresses(&self) -> Vec<f32> {
        self.buffer.iter().map(|_| 0.0).collect()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
