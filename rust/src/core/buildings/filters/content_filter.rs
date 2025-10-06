use crate::core::building::{Building, BuildingAction, BuildingType};
use crate::core::dto::{BuildingId, Vec2i};
use crate::core::packet::Packet;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFilterConfig {
    pub pattern: String,
}

pub struct ContentFilter {
    id: BuildingId,
    pos: Vec2i,
    rot: i32,
    buffer: Option<Packet>,
    pub config: Option<ContentFilterConfig>,
    compiled_regex: Option<Regex>,
}

impl ContentFilter {
    pub fn new(id: BuildingId, pos: Vec2i, rot: i32) -> Self {
        Self {
            id,
            pos,
            rot,
            buffer: None,
            config: None,
            compiled_regex: None,
        }
    }

    pub fn new_with_config(
        id: BuildingId,
        pos: Vec2i,
        rot: i32,
        config: ContentFilterConfig,
    ) -> Self {
        let compiled_regex = Regex::new(&config.pattern).ok();
        Self {
            id,
            pos,
            rot,
            buffer: None,
            config: Some(config),
            compiled_regex,
        }
    }

    pub fn set_config(&mut self, config: ContentFilterConfig) {
        self.compiled_regex = Regex::new(&config.pattern).ok();
        self.config = Some(config);
    }

    pub fn filter(&self, packet: &Packet) -> bool {
        if let (Some(_config), Some(regex)) = (&self.config, &self.compiled_regex) {
            // Try to convert payload to string for regex matching
            if let Ok(payload_str) = str::from_utf8(&packet.payload) {
                regex.is_match(payload_str)
            } else {
                false
            }
        } else {
            false // 設定がない場合は何も通さない
        }
    }
}

impl Building for ContentFilter {
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
        BuildingType::ContentFilter
    }
    fn get_size(&self) -> Vec2i {
        Vec2i { x: 1, y: 1 }
    }
    fn get_output_poses(&self) -> Vec<Vec2i> {
        let rotation = self.rot.rem_euclid(4);
        let offsets = match rotation {
            0 => [(0, -1), (1, 0), (-1, 0)],
            1 => [(1, 0), (0, 1), (0, -1)],
            2 => [(0, 1), (-1, 0), (1, 0)],
            3 => [(-1, 0), (0, -1), (0, 1)],
            _ => [(0, -1), (1, 0), (-1, 0)],
        };

        offsets
            .into_iter()
            .map(|(dx, dy)| Vec2i {
                x: self.pos.x + dx,
                y: self.pos.y + dy,
            })
            .collect()
    }
    fn get_input_poses(&self) -> Vec<Vec2i> {
        let (dx, dy) = match self.rot.rem_euclid(4) {
            0 => (0, 1),
            1 => (-1, 0),
            2 => (0, -1),
            3 => (1, 0),
            _ => (0, 1),
        };

        vec![Vec2i {
            x: self.pos.x + dx,
            y: self.pos.y + dy,
        }]
    }
    fn update(&mut self, _delta: f32) {}
    fn can_offload(&self) -> bool {
        self.buffer.is_some()
    }
    fn can_accept(&self, _packet: &Packet, _source_pos: Vec2i) -> bool {
        !self.buffer.is_some()
    }
    fn offload(&mut self) -> Packet {
        self.buffer.take().expect("Offload called without packet")
    }
    fn accept(&mut self, packet: Packet, _source_pos: Vec2i) -> BuildingAction {
        self.buffer = Some(packet);
        BuildingAction::None
    }
    fn get_progress(&self) -> f32 {
        0.0
    }
    fn get_packets(&self) -> Vec<Packet> {
        self.buffer.iter().cloned().collect()
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
