use godot::classes::ProjectSettings;
use godot::prelude::*;
use pcap_file::pcap::{PcapHeader, PcapReader};
use std::fs::File;

use crate::{packet::PcapCapture, packet::PcapFrame, packet::Traffic};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct PcapLoader {
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for PcapLoader {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl PcapLoader {
    #[func]
    pub fn load_pcap(&self, path: GString) -> Option<Gd<PcapCapture>> {
        let path = path.to_string();
        let abs: GString = ProjectSettings::singleton().globalize_path(path.as_str());
        godot_print!("load_pcap: req='{}' abs='{}'", path, abs);

        let file = File::open(abs.to_string()).ok()?;
        let mut reader = PcapReader::new(file).ok()?;

        let header: PcapHeader = reader.header();
        let mut capture = PcapCapture::new_gd();
        {
            let mut cap_mut = capture.bind_mut();
            cap_mut.set_linktype(u32::from(header.datalink));
            cap_mut.set_snaplen(header.snaplen);
        }

        while let Some(pkt_res) = reader.next_packet() {
            let pkt = match pkt_res {
                Ok(p) => p,
                Err(_) => continue,
            };

            let mut data = PackedByteArray::new();
            data.resize(pkt.data.len());
            data.as_mut_slice().copy_from_slice(&pkt.data);

            let mut frame = PcapFrame::new_gd();
            {
                let mut frame_mut = frame.bind_mut();
                let ts = pkt.timestamp;
                frame_mut.set_timestamp_sec(ts.as_secs() as i64);
                frame_mut.set_timestamp_usec(ts.subsec_micros() as i64);
                frame_mut.set_orig_len(pkt.orig_len as i64);
                frame_mut.set_data(data);
                frame_mut.refresh_payload();
            }
            {
                let mut cap_mut = capture.bind_mut();
                cap_mut.push_frame(frame);
            }
        }

        Some(capture)
    }

    #[func]
    pub fn load_traffic(&self, path: GString) -> Option<Gd<Traffic>> {
        let capture = self.load_pcap(path.clone())?;
        let traffic = {
            let capture_ref = capture.bind();
            capture_ref.to_traffic()
        };
        Some(traffic)
    }
}
