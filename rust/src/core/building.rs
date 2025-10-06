use crate::core::dto::Vec2i;
use crate::core::packet::Packet;
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    Internet,
    Datacenter,
    Conveyor,
    IpFilter,
    PortFilter,
    LengthFilter,
    ProtocolFilter,
    ContentFilter,
    Junction,
    RecycleBin,
}

#[derive(Debug, Clone, Copy)]
pub enum BuildingAction {
    None,
    AddScore(u32),
    SubScore(u32),
}

use crate::core::dto::BuildingId;

pub trait Building {
    fn id(&self) -> BuildingId;
    fn position(&self) -> Vec2i;
    fn rotation(&self) -> i32;
    fn building_type(&self) -> BuildingType;
    fn get_size(&self) -> Vec2i;
    fn get_output_poses(&self) -> Vec<Vec2i>;
    fn get_input_poses(&self) -> Vec<Vec2i>;
    fn update(&mut self, delta: f32);
    fn can_offload(&self) -> bool;
    fn can_accept(&self, packet: &Packet, source_pos: Vec2i) -> bool;
    fn offload(&mut self) -> Packet;
    fn accept(&mut self, packet: Packet, source_pos: Vec2i) -> BuildingAction;
    fn get_progress(&self) -> f32;
    fn get_packets(&self) -> Vec<Packet>;
    fn get_packet_progresses(&self) -> Vec<f32>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
