use godot::prelude::*;

use super::Packet;

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct Traffic {
    #[base]
    base: Base<Resource>,

    #[var]
    packets: Array<Gd<Packet>>,
}

#[godot_api]
impl IResource for Traffic {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            packets: Array::new(),
        }
    }
}

#[godot_api]
impl Traffic {
    #[func]
    pub fn packets(&self) -> Array<Gd<Packet>> {
        self.packets.clone()
    }

    #[func]
    pub fn packet_count(&self) -> i64 {
        self.packets.len() as i64
    }

    #[func]
    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    #[func]
    pub fn get_packet(&self, index: i32) -> Option<Gd<Packet>> {
        self.packets.get(index as usize)
    }

    #[func]
    pub fn push_packet(&mut self, packet: Gd<Packet>) {
        self.packets.push(&packet);
    }

    #[func]
    pub fn clear(&mut self) {
        self.packets.clear();
    }
}
