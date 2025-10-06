use godot::prelude::*;

use crate::packet::PcapFrame;
use crate::packet::normalize_timestamp;
use crate::packet::{Packet, Traffic};

#[derive(GodotClass)]
#[class(base=Resource)]
pub struct PcapCapture {
    #[base]
    base: Base<Resource>,

    #[var]
    linktype: u32,
    #[var]
    snaplen: u32,
    #[var]
    frames: Array<Gd<PcapFrame>>,
}

#[godot_api]
impl PcapCapture {
    #[func]
    pub fn linktype(&self) -> u32 {
        self.linktype
    }
    #[func]
    pub fn snaplen(&self) -> u32 {
        self.snaplen
    }
    #[func]
    pub fn frames(&self) -> Array<Gd<PcapFrame>> {
        self.frames.clone()
    }
    #[func]
    pub fn frame_count(&self) -> i64 {
        self.frames.len() as i64
    }

    #[func]
    pub fn get_frame(&self, index: i32) -> Option<Gd<PcapFrame>> {
        self.frames.get(index as usize)
    }

    #[func]
    pub fn get_packet(&self, index: i32) -> Option<Gd<Packet>> {
        let frame = self.get_frame(index)?;
        let frame_ref = frame.bind();
        frame_ref.to_packet()
    }

    #[func]
    pub fn to_traffic(&self) -> Gd<Traffic> {
        let mut packets_vec: Vec<Gd<Packet>> = self
            .frames
            .iter_shared()
            .filter_map(|frame| {
                let frame_ref = frame.bind();
                frame_ref.to_packet()
            })
            .collect();

        packets_vec.sort_by(|a, b| {
            let a_ts = a.bind().get_timestamp();
            let b_ts = b.bind().get_timestamp();
            a_ts.cmp(&b_ts)
        });

        let baseline = packets_vec
            .first()
            .map(|packet| packet.bind().get_timestamp());

        let mut packets = Array::<Gd<Packet>>::new();
        for mut packet in packets_vec.into_iter() {
            {
                let mut packet_mut = packet.bind_mut();
                let timestamp = packet_mut.get_timestamp();
                packet_mut.set_timestamp(normalize_timestamp(timestamp, baseline));
            }
            packets.push(&packet);
        }

        let mut traffic = Traffic::new_gd();
        {
            let mut traffic_mut = traffic.bind_mut();
            traffic_mut.set_packets(packets);
        }
        traffic
    }

    #[func]
    pub fn push_frame(&mut self, frame: Gd<PcapFrame>) {
        self.frames.push(&frame);
    }
}

#[godot_api]
impl IResource for PcapCapture {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            linktype: 0,
            snaplen: 0,
            frames: Array::new(),
        }
    }
}
