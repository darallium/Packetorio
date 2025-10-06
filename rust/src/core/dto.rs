use crate::core::building::BuildingType;
use crate::core::packet::Packet;
use godot::prelude::Vector2i;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl From<Vector2i> for Vec2i {
    fn from(v: Vector2i) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Vec2i> for Vector2i {
    fn from(v: Vec2i) -> Self {
        Self { x: v.x, y: v.y }
    }
}

pub type BuildingId = u64;

#[derive(Debug, Clone)]
pub enum WorldEvent {
    BuildingPlaced {
        id: BuildingId,
        pos: Vec2i,
        building_type: BuildingType,
        rotation: i32,
    },
    BuildingRemoved {
        id: BuildingId,
        pos: Vec2i,
    },
    BuildingProgressUpdated {
        id: BuildingId,
        progress: f32,
    },
    PacketMoved {
        packet: Packet,
        from_id: BuildingId,
        to_id: BuildingId,
        progress_start: f32,
    },
}
