use crate::core::building::{Building, BuildingAction, BuildingType};
use crate::core::dto::{BuildingId, Vec2i};
use crate::core::packet::{Packet, PacketLabel, Protocol};
use crate::packet::Traffic;
use godot::prelude::*;

pub struct Internet {
    id: BuildingId,
    pos: Vec2i,
    rot: i32,
    packets: Vec<Packet>,
    traffic: Option<Gd<Traffic>>,
    time: f32,
    next_packet_index: usize,
}

impl Internet {
    pub fn new(id: BuildingId, pos: Vec2i, rot: i32) -> Self {
        Self {
            id,
            pos,
            rot,
            packets: Vec::new(),
            traffic: None,
            time: 0.0,
            next_packet_index: 0,
        }
    }

    pub fn set_traffic(&mut self, traffic: Gd<crate::packet::Traffic>) {
        self.traffic = Some(traffic);
        self.time = 0.0;
        self.next_packet_index = 0;
    }

    #[cfg(test)]
    pub fn add_packet(&mut self, packet: Packet) {
        self.packets.push(packet);
    }
}

impl Building for Internet {
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
        BuildingType::Internet
    }
    fn get_size(&self) -> Vec2i {
        Vec2i { x: 2, y: 2 }
    }
    fn get_output_poses(&self) -> Vec<Vec2i> {
        let mut poses = Vec::new();
        let size = self.get_size();
        // Top and Bottom
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
        // Left and Right
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
    fn get_input_poses(&self) -> Vec<Vec2i> {
        Vec::new()
    }
    fn update(&mut self, delta: f32) {
        self.time += delta;
        if let Some(traffic) = &self.traffic {
            let traffic = traffic.bind();
            let traffic_packets = traffic.packets();

            while self.next_packet_index < traffic_packets.len() {
                if let Some(packet_resource) = traffic_packets.get(self.next_packet_index) {
                    let packet_resource = packet_resource.bind();
                    // Packet timestamps are stored as microseconds relative to the capture start.
                    let packet_time = (packet_resource.get_timestamp() as f64) / 1_000_000.0;
                    if (self.time as f64) >= packet_time {
                        // This is a simplified conversion.
                        // A more robust solution would handle different packet types.
                        let protocol = match packet_resource.get_protocol() {
                            6 => Protocol::Tcp,
                            17 => Protocol::Udp,
                            _ => Protocol::Unknown,
                        };
                        let mut new_packet = Packet::new(
                            packet_resource.get_src_ip().to_string(),
                            packet_resource.get_dst_ip().to_string(),
                            packet_resource.get_src_port() as u16,
                            packet_resource.get_dst_port() as u16,
                            protocol,
                            packet_resource.get_packet_size() as u32,
                            vec![], // ペイロードは今のところ空
                        );
                        new_packet.label = PacketLabel::from_raw(packet_resource.get_label());
                        self.packets.push(new_packet);
                        self.next_packet_index += 1;
                    } else {
                        break; // Packets are sorted by time, so we can stop.
                    }
                } else {
                    break;
                }
            }
        }
    }
    fn can_offload(&self) -> bool {
        !self.packets.is_empty()
    }
    fn can_accept(&self, _packet: &Packet, _source_pos: Vec2i) -> bool {
        false
    }
    fn offload(&mut self) -> Packet {
        self.packets.remove(0)
    }
    fn accept(&mut self, _packet: Packet, _source_pos: Vec2i) -> BuildingAction {
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
