use crate::core::building::{Building, BuildingAction, BuildingType};
use crate::core::dto::{BuildingId, Vec2i};
use crate::core::packet::Packet;

const CONVEYOR_SPEED: f32 = 1.0; // 1タイル/秒

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntrySide {
    Back,
    Left,
    Right,
}

pub struct Conveyor {
    id: BuildingId,
    pos: Vec2i,
    rot: i32,
    buffer: Option<(Packet, EntrySide)>,
}

impl Conveyor {
    pub fn new(id: BuildingId, pos: Vec2i, rot: i32) -> Self {
        Self {
            id,
            pos,
            rot,
            buffer: None,
        }
    }

    pub fn get_front_pos(&self) -> Vec2i {
        let (dx, dy) = match self.rot.rem_euclid(4) {
            0 => (1, 0),  // East
            1 => (0, 1),  // South
            2 => (-1, 0), // West
            3 => (0, -1), // North
            _ => (1, 0),
        };
        Vec2i {
            x: self.pos.x + dx,
            y: self.pos.y + dy,
        }
    }

    fn entry_side_from_delta(&self, dx: i32, dy: i32) -> Option<EntrySide> {
        if dx == 0 && dy == 0 {
            return None;
        }

        let front_vec = {
            let front = self.get_front_pos();
            Vec2i {
                x: front.x - self.pos.x,
                y: front.y - self.pos.y,
            }
        };

        let right_vec = Vec2i {
            x: front_vec.y,
            y: -front_vec.x,
        };
        let left_vec = Vec2i {
            x: -front_vec.y,
            y: front_vec.x,
        };
        let back_vec = Vec2i {
            x: -front_vec.x,
            y: -front_vec.y,
        };

        let delta = Vec2i { x: dx, y: dy };

        if delta == back_vec {
            Some(EntrySide::Back)
        } else if delta == left_vec {
            Some(EntrySide::Left)
        } else if delta == right_vec {
            Some(EntrySide::Right)
        } else {
            None
        }
    }

    pub fn buffer_state(&self) -> Option<(Packet, EntrySide)> {
        self.buffer
            .as_ref()
            .map(|(packet, side)| (packet.clone(), *side))
    }
}

impl Building for Conveyor {
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
        BuildingType::Conveyor
    }
    fn get_size(&self) -> Vec2i {
        Vec2i { x: 1, y: 1 }
    }
    fn get_output_poses(&self) -> Vec<Vec2i> {
        vec![self.get_front_pos()]
    }
    fn get_input_poses(&self) -> Vec<Vec2i> {
        let front = self.get_front_pos();

        let candidates = [
            Vec2i {
                x: self.pos.x - 1,
                y: self.pos.y,
            },
            Vec2i {
                x: self.pos.x + 1,
                y: self.pos.y,
            },
            Vec2i {
                x: self.pos.x,
                y: self.pos.y - 1,
            },
            Vec2i {
                x: self.pos.x,
                y: self.pos.y + 1,
            },
        ];

        candidates.into_iter().filter(|pos| *pos != front).collect()
    }

    fn update(&mut self, delta: f32) {
        if let Some((packet, _)) = &mut self.buffer {
            packet.progress += CONVEYOR_SPEED * delta;
            if packet.progress > 1.0 {
                packet.progress = 1.0;
            }
        }
    }

    fn can_offload(&self) -> bool {
        matches!(&self.buffer, Some((p, _)) if p.progress >= 1.0)
    }

    fn can_accept(&self, _packet: &Packet, source_pos: Vec2i) -> bool {
        if self.buffer.is_some() {
            return false;
        }
        source_pos != self.get_front_pos()
    }

    fn offload(&mut self) -> Packet {
        self.buffer.take().expect("Offload called without packet").0
    }

    fn accept(&mut self, mut packet: Packet, source_pos: Vec2i) -> BuildingAction {
        let source_dx = source_pos.x - self.pos.x;
        let source_dy = source_pos.y - self.pos.y;

        let entry_side = self
            .entry_side_from_delta(source_dx, source_dy)
            .unwrap_or(EntrySide::Back);

        let is_lateral = matches!(entry_side, EntrySide::Left | EntrySide::Right);

        packet.progress = if is_lateral { 0.5 } else { 0.0 };
        self.buffer = Some((packet, entry_side));
        BuildingAction::None
    }

    fn get_progress(&self) -> f32 {
        self.buffer.as_ref().map_or(0.0, |(p, _)| p.progress)
    }

    fn get_packets(&self) -> Vec<Packet> {
        self.buffer.iter().map(|(p, _)| p.clone()).collect()
    }

    fn get_packet_progresses(&self) -> Vec<f32> {
        self.buffer.iter().map(|(p, _)| p.progress).collect()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dto::Vec2i;
    use crate::core::packet::{Packet, Protocol};

    fn create_test_packet() -> Packet {
        Packet::new(
            "192.168.1.1".to_string(),
            "192.168.1.2".to_string(),
            12345,
            80,
            Protocol::Tcp,
            64,
            vec![],
        )
    }

    #[test]
    fn conveyor_packet_lifecycle() {
        let mut conveyor = Conveyor::new(1, Vec2i { x: 0, y: 1 }, 0);
        let source_pos = Vec2i { x: -1, y: 1 };
        let packet = create_test_packet();

        assert!(conveyor.can_accept(&packet, source_pos));
        conveyor.accept(packet.clone(), source_pos);
        assert!(!conveyor.can_accept(&packet, source_pos));

        assert_eq!(conveyor.get_packets().len(), 1);
        assert_eq!(conveyor.get_progress(), 0.0);
        assert!(!conveyor.can_offload());

        conveyor.update(0.5);
        assert_eq!(conveyor.get_progress(), 0.5);
        assert!(!conveyor.can_offload());

        conveyor.update(0.5);
        assert_eq!(conveyor.get_progress(), 1.0);
        assert!(conveyor.can_offload());

        let offloaded_packet = conveyor.offload();
        assert_eq!(offloaded_packet.source_ip, "192.168.1.1");
        assert!(conveyor.buffer.is_none());
        assert!(conveyor.get_packets().is_empty());
    }
}
