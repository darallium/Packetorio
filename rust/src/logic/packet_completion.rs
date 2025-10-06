use std::collections::HashMap;
use std::sync::Mutex;

use godot::prelude::*;

use crate::core::building::BuildingType;
use crate::core::dto::BuildingId;
use crate::core::packet::{Packet as CorePacket, PacketLabel, Protocol};
use crate::logic::building_storage::BuildingStorage;
use crate::map_controller::World;
use crate::packet::{Packet as TrafficPacket, Traffic};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PacketKey {
    source_ip: String,
    dest_ip: String,
    source_port: u16,
    dest_port: u16,
    protocol: Protocol,
    length: u32,
    label: PacketLabel,
}

impl PacketKey {
    fn from_core(packet: &CorePacket) -> Self {
        Self {
            source_ip: packet.source_ip.clone(),
            dest_ip: packet.dest_ip.clone(),
            source_port: packet.source_port,
            dest_port: packet.dest_port,
            protocol: packet.protocol.clone(),
            length: packet.length,
            label: packet.label,
        }
    }

    fn from_resource(packet: &Gd<TrafficPacket>) -> Option<Self> {
        let packet = packet.bind();
        let protocol = match packet.get_protocol() {
            6 => Protocol::Tcp,
            17 => Protocol::Udp,
            _ => Protocol::Unknown,
        };
        let label = PacketLabel::from_raw(packet.get_label());
        Some(Self {
            source_ip: packet.get_src_ip().to_string(),
            dest_ip: packet.get_dst_ip().to_string(),
            source_port: packet.get_src_port() as u16,
            dest_port: packet.get_dst_port() as u16,
            protocol,
            length: packet.get_packet_size() as u32,
            label,
        })
    }
}

#[derive(Clone, Debug)]
struct PacketReport {
    building_type: BuildingType,
    building_id: BuildingId,
    packet: CorePacket,
}

static PLANNED_PACKETS: Mutex<Option<Vec<PacketKey>>> = Mutex::new(None);

pub fn register_planned_traffic(traffic: &Gd<Traffic>) {
    let traffic = traffic.bind();
    let packets = traffic.packets();

    let mut planned = Vec::with_capacity(packets.len());
    for idx in 0..packets.len() {
        if let Some(packet) = packets.get(idx) {
            if let Some(key) = PacketKey::from_resource(&packet) {
                planned.push(key);
            }
        }
    }

    set_planned_packet_keys(planned);
}

pub fn clear_planned_packets() {
    let mut guard = PLANNED_PACKETS
        .lock()
        .expect("PLANNED_PACKETS mutex poisoned in clear_planned_packets");
    *guard = None;
}

fn set_planned_packet_keys(keys: Vec<PacketKey>) {
    let mut guard = PLANNED_PACKETS
        .lock()
        .expect("PLANNED_PACKETS mutex poisoned in set_planned_packet_keys");
    *guard = Some(keys);
}

#[cfg(test)]
pub fn register_planned_core_packets<'a, I>(packets: I)
where
    I: IntoIterator<Item = &'a CorePacket>,
{
    let planned = packets
        .into_iter()
        .map(PacketKey::from_core)
        .collect::<Vec<_>>();
    set_planned_packet_keys(planned);
}

pub fn completed_packets_variant(world: &World) -> Variant {
    match check_all_packets_transferred(world) {
        Some(reports) => {
            let mut array = VariantArray::new();
            for report in reports {
                let variant = packet_report_to_variant(report);
                array.push(&variant);
            }
            array.to_variant()
        }
        None => Variant::nil(),
    }
}

#[cfg(test)]
pub fn completed_packet_count_for_test(world: &World) -> Option<usize> {
    check_all_packets_transferred(world).map(|reports| reports.len())
}

fn check_all_packets_transferred(world: &World) -> Option<Vec<PacketReport>> {
    let planned_keys = {
        let guard = PLANNED_PACKETS
            .lock()
            .expect("PLANNED_PACKETS mutex poisoned in check_all_packets_transferred");
        guard.clone()?
    };

    let reports = collect_delivered_packets(&world.storage);
    if reports.is_empty() && planned_keys.is_empty() {
        return Some(reports);
    }

    let delivered_counts = to_counts(
        reports
            .iter()
            .map(|report| PacketKey::from_core(&report.packet)),
    );
    let planned_counts = to_counts(planned_keys.into_iter());

    if delivered_counts == planned_counts {
        Some(reports)
    } else {
        None
    }
}

fn collect_delivered_packets(storage: &BuildingStorage) -> Vec<PacketReport> {
    let mut reports = Vec::new();
    for building in storage.iter() {
        match building.building_type() {
            BuildingType::Datacenter | BuildingType::RecycleBin => {
                let building_type = building.building_type();
                let building_id = building.id();
                for packet in building.get_packets() {
                    reports.push(PacketReport {
                        building_type,
                        building_id,
                        packet,
                    });
                }
            }
            _ => {}
        }
    }
    reports
}

fn to_counts<I>(iter: I) -> HashMap<PacketKey, usize>
where
    I: IntoIterator<Item = PacketKey>,
{
    let mut counts = HashMap::new();
    for key in iter {
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

fn packet_report_to_variant(report: PacketReport) -> Variant {
    let mut dict = Dictionary::new();
    dict.set("building_id", report.building_id.to_variant());
    dict.set("building_type", (report.building_type as i32).to_variant());
    dict.set("source_ip", report.packet.source_ip.to_variant());
    dict.set("dest_ip", report.packet.dest_ip.to_variant());
    dict.set(
        "source_port",
        (report.packet.source_port as i32).to_variant(),
    );
    dict.set("dest_port", (report.packet.dest_port as i32).to_variant());
    dict.set("protocol", (report.packet.protocol as i32).to_variant());
    dict.set("length", (report.packet.length as i32).to_variant());
    dict.set("label", report.packet.label.to_raw().to_variant());
    dict.to_variant()
}
