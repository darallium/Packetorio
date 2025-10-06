use godot::classes::{INode, Json, Node, ResourceLoader};
use godot::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::building::BuildingType;
use crate::core::building_defs::building;
use crate::core::dto::Vec2i as CoreVec2i;
use crate::logic::building_map::BuildingMap;
use crate::logic::connection_graph::{ConnectionEdge, ConnectionGraph, OutputRole};
use crate::packet::{JsonLoader, PcapLoader};

use crate::core::building::Building;
use crate::core::buildings::conveyor::{Conveyor, EntrySide};
use crate::core::buildings::datacenter::Datacenter;
use crate::core::buildings::internet::Internet;
use crate::core::buildings::recycle_bin::RecycleBin;
use crate::core::dto::{BuildingId, WorldEvent};
use crate::logic::building_storage::BuildingStorage;
use crate::logic::packet_completion;

use crate::core::buildings::filters::content_filter::{ContentFilter, ContentFilterConfig};
use crate::core::buildings::filters::ip_filter::{IpFilter, IpFilterConfig, IpFilterDirection};
use crate::core::buildings::filters::length_filter::{
    LengthFilter, LengthFilterConfig, LengthFilterDirection,
};
use crate::core::buildings::filters::port_filter::{
    PortFilter, PortFilterConfig, PortFilterDirection,
};
use crate::core::buildings::filters::protocol_filter::{ProtocolFilter, ProtocolFilterConfig};
use crate::core::packet::Protocol;
fn variant_to_i32(value: &Variant) -> Option<i32> {
    if let Ok(i) = value.try_to::<i32>() {
        return Some(i);
    }

    if let Ok(f) = value.try_to::<f64>()
        && f.is_finite()
    {
        return Some(f.round() as i32);
    }

    if let Ok(gstr) = value.try_to::<GString>() {
        let text = gstr.to_string();
        if let Ok(i) = text.parse::<i32>() {
            return Some(i);
        }
        if let Ok(f) = text.parse::<f64>()
            && f.is_finite()
        {
            return Some(f.round() as i32);
        }
    }

    None
}

mod packet_export;

pub struct World {
    pub storage: BuildingStorage,
    map: BuildingMap,
    graph: ConnectionGraph,
    next_id: u64,
    events: Vec<WorldEvent>,
    route_counters: HashMap<(BuildingId, OutputRole), usize>,
    pub score: u32,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            storage: BuildingStorage::new(),
            map: BuildingMap::new(),
            graph: ConnectionGraph::new(),
            next_id: 0,
            events: Vec::new(),
            route_counters: HashMap::new(),
            score: 0,
        }
    }

    pub fn place_building(&mut self, pos: CoreVec2i, building_type: BuildingType, rotation: i32) {
        let id = self.next_id;

        let building: Box<dyn Building> = match building_type {
            BuildingType::Conveyor => Box::new(Conveyor::new(id, pos, rotation)),
            BuildingType::Internet => Box::new(Internet::new(id, pos, rotation)),
            BuildingType::Datacenter => Box::new(Datacenter::new(id, pos, rotation)),
            BuildingType::RecycleBin => Box::new(RecycleBin::new(id, pos, rotation)),
            BuildingType::IpFilter => Box::new(IpFilter::new(id, pos, rotation)),
            BuildingType::PortFilter => Box::new(PortFilter::new(id, pos, rotation)),
            BuildingType::LengthFilter => Box::new(LengthFilter::new(id, pos, rotation)),
            BuildingType::ProtocolFilter => Box::new(ProtocolFilter::new(id, pos, rotation)),
            BuildingType::ContentFilter => Box::new(ContentFilter::new(id, pos, rotation)),
            _ => return, // or handle error
        };
        let size = building.get_size();

        // 衝突検知
        for y in 0..size.y {
            for x in 0..size.x {
                let check_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                if self.map.get(&check_pos).is_some() {
                    return; // 既に何かある
                }
            }
        }

        self.next_id += 1;

        // ストレージに追加
        self.storage.add_building(building);

        // マップに登録
        for y in 0..size.y {
            for x in 0..size.x {
                let tile_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                self.map.insert(tile_pos, id);
            }
        }

        self.rebuild_connections();

        self.events.push(WorldEvent::BuildingPlaced {
            id,
            pos,
            building_type,
            rotation,
        });
    }

    pub fn place_protocol_filter_with_config(
        &mut self,
        pos: CoreVec2i,
        rotation: i32,
        config: ProtocolFilterConfig,
    ) {
        let id = self.next_id;

        let building: Box<dyn Building> =
            Box::new(ProtocolFilter::new_with_config(id, pos, rotation, config));
        let size = building.get_size();

        // 衝突検知
        for y in 0..size.y {
            for x in 0..size.x {
                let check_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                if self.map.get(&check_pos).is_some() {
                    return; // 既に何かある
                }
            }
        }

        self.next_id += 1;

        // ストレージに追加
        self.storage.add_building(building);

        // マップに登録
        for y in 0..size.y {
            for x in 0..size.x {
                let tile_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                self.map.insert(tile_pos, id);
            }
        }

        // 接続を再構築
        self.rebuild_connections();

        self.events.push(WorldEvent::BuildingPlaced {
            id,
            pos,
            building_type: BuildingType::ProtocolFilter,
            rotation,
        });
    }

    pub fn place_ip_filter_with_config(
        &mut self,
        pos: CoreVec2i,
        rotation: i32,
        config: IpFilterConfig,
    ) {
        let id = self.next_id;

        let building: Box<dyn Building> =
            Box::new(IpFilter::new_with_config(id, pos, rotation, config));
        let size = building.get_size();

        // 衝突検知
        for y in 0..size.y {
            for x in 0..size.x {
                let check_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                if self.map.get(&check_pos).is_some() {
                    return; // 既に何かある
                }
            }
        }

        self.next_id += 1;
        self.storage.add_building(building);
        // 設置
        for y in 0..size.y {
            for x in 0..size.x {
                let tile_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                self.map.insert(tile_pos, id);
            }
        }

        self.rebuild_connections();

        self.events.push(WorldEvent::BuildingPlaced {
            id,
            pos,
            building_type: BuildingType::IpFilter,
            rotation,
        });
    }

    pub fn place_length_filter_with_config(
        &mut self,
        pos: CoreVec2i,
        rotation: i32,
        config: LengthFilterConfig,
    ) {
        let id = self.next_id;

        let building: Box<dyn Building> =
            Box::new(LengthFilter::new_with_config(id, pos, rotation, config));
        let size = building.get_size();

        // 衝突検知
        for y in 0..size.y {
            for x in 0..size.x {
                let check_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                if self.map.get(&check_pos).is_some() {
                    return; // 既に何かある
                }
            }
        }

        self.next_id += 1;
        self.storage.add_building(building);
        // 設置
        for y in 0..size.y {
            for x in 0..size.x {
                let tile_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                self.map.insert(tile_pos, id);
            }
        }

        self.rebuild_connections();

        self.events.push(WorldEvent::BuildingPlaced {
            id,
            pos,
            building_type: BuildingType::LengthFilter,
            rotation,
        });
    }

    pub fn place_port_filter_with_config(
        &mut self,
        pos: CoreVec2i,
        rotation: i32,
        config: PortFilterConfig,
    ) {
        let id = self.next_id;

        let building: Box<dyn Building> =
            Box::new(PortFilter::new_with_config(id, pos, rotation, config));
        let size = building.get_size();

        // 衝突検知
        for y in 0..size.y {
            for x in 0..size.x {
                let check_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                if self.map.get(&check_pos).is_some() {
                    return; // 既に何かある
                }
            }
        }

        self.next_id += 1;

        // ストレージに追加
        self.storage.add_building(building);

        // マップに登録
        for y in 0..size.y {
            for x in 0..size.x {
                let tile_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                self.map.insert(tile_pos, id);
            }
        }

        self.rebuild_connections();

        self.events.push(WorldEvent::BuildingPlaced {
            id,
            pos,
            building_type: BuildingType::PortFilter,
            rotation,
        });
    }

    pub fn place_content_filter_with_config(
        &mut self,
        pos: CoreVec2i,
        rotation: i32,
        config: ContentFilterConfig,
    ) {
        let id = self.next_id;

        let building: Box<dyn Building> =
            Box::new(ContentFilter::new_with_config(id, pos, rotation, config));
        let size = building.get_size();

        // 衝突検知
        for y in 0..size.y {
            for x in 0..size.x {
                let check_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                if self.map.get(&check_pos).is_some() {
                    return; // 既に何かある
                }
            }
        }

        self.next_id += 1;

        // ストレージに追加
        self.storage.add_building(building);

        // マップに登録
        for y in 0..size.y {
            for x in 0..size.x {
                let tile_pos = CoreVec2i {
                    x: pos.x + x,
                    y: pos.y + y,
                };
                self.map.insert(tile_pos, id);
            }
        }

        self.rebuild_connections();

        self.events.push(WorldEvent::BuildingPlaced {
            id,
            pos,
            building_type: BuildingType::ContentFilter,
            rotation,
        });
    }

    pub fn remove_building(&mut self, pos: &CoreVec2i) {
        if let Some(id) = self.map.get(pos) {
            if let Some(building) = self.storage.get(id) {
                let size = building.get_size();
                let base_pos = building.position();

                for y in 0..size.y {
                    for x in 0..size.x {
                        let tile_pos = CoreVec2i {
                            x: base_pos.x + x,
                            y: base_pos.y + y,
                        };
                        self.map.remove(&tile_pos);
                    }
                }
            }

            self.storage.remove(id);
            self.graph.remove_node(id);
            self.route_counters
                .retain(|(tracked_id, _), _| *tracked_id != id);

            self.events
                .push(WorldEvent::BuildingRemoved { id, pos: *pos });
        }
    }

    pub fn update(&mut self, delta: f32) {
        // 1. 内部状態更新フェーズ
        for building in self.storage.iter_mut() {
            let old_progress = building.get_progress();
            building.update(delta);
            let new_progress = building.get_progress();
            if (new_progress - old_progress).abs() > f32::EPSILON {
                self.events.push(WorldEvent::BuildingProgressUpdated {
                    id: building.id(),
                    progress: new_progress,
                });
            }
        }

        // 2. 転送決定フェーズ (不変)
        let mut decisions: Vec<(BuildingId, ConnectionEdge)> = Vec::new();
        let mut packets_to_drop = Vec::new();

        let potential_sources: Vec<BuildingId> = self.storage.iter().map(|b| b.id()).collect();

        for from_id in potential_sources {
            let Some((building_type, source_pos, filter_result, packet_to_offload)) =
                self.prepare_packet_info(from_id)
            else {
                continue;
            };

            let Some(edges) = self.graph.get_outputs(from_id) else {
                packets_to_drop.push(from_id);
                continue;
            };
            let mut default_edges = Vec::new();
            let mut match_edges = Vec::new();
            let mut mismatch_edges = Vec::new();
            let mut has_any_targets = false;
            let mut has_any_accepting_targets = false;

            for edge in edges.iter() {
                if let Some(to_building) = self.storage.get(edge.to_id) {
                    has_any_targets = true;
                    if !to_building.can_accept(&packet_to_offload, source_pos) {
                        continue;
                    }
                    has_any_accepting_targets = true;
                } else {
                    continue;
                }

                match edge.role {
                    OutputRole::Default => default_edges.push(*edge),
                    OutputRole::FilterMatch => match_edges.push(*edge),
                    OutputRole::FilterMismatch => mismatch_edges.push(*edge),
                }
            }

            let selected_edge = select_edge_for_building(
                from_id,
                building_type,
                filter_result,
                &mut self.route_counters,
                &default_edges,
                &match_edges,
                &mismatch_edges,
            );

            if let Some(edge) = selected_edge {
                decisions.push((from_id, edge));
            } else if !has_any_targets {
                packets_to_drop.push(from_id);
            } else if !has_any_accepting_targets {
                // All targets are temporarily full; keep packet queued for a later tick.
                continue;
            } else {
                // Reaching here means there were accepting targets but routing failed; drop as a safeguard.
                packets_to_drop.push(from_id);
            }
        }

        // 3. 転送実行フェーズ (可変)
        for (from_id, edge) in decisions {
            if let Some((from_building, to_building)) =
                self.storage.get_two_mut(from_id, edge.to_id)
            {
                let packet = from_building.offload();
                let progress_start = packet.progress;
                let source_pos = from_building.position();
                let event_packet = packet.clone();
                to_building.accept(packet, source_pos);

                self.events.push(WorldEvent::PacketMoved {
                    packet: event_packet,
                    from_id,
                    to_id: edge.to_id,
                    progress_start,
                });
            }
        }

        // 4. パケット破棄フェーズ
        for id in packets_to_drop {
            if let Some(building) = self.storage.get_mut(id)
                && building.building_type() != BuildingType::RecycleBin
                && building.can_offload()
            {
                building.offload(); // パケットを破棄
                // ここでイベントを発行することも可能
            }
        }
    }

    pub fn drain_events(&mut self) -> Vec<WorldEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn get_building(&self, id: BuildingId) -> Option<&dyn Building> {
        self.storage.get(id)
    }

    pub fn rebuild_connections(&mut self) {
        self.graph.rebuild(&self.map, &self.storage);
        self.route_counters
            .retain(|(id, _), _| self.graph.get_outputs(*id).is_some());
    }

    #[cfg(test)]
    pub fn get_output_connections(&self, from_id: BuildingId) -> Option<&Vec<ConnectionEdge>> {
        self.graph.get_outputs(from_id)
    }

    #[cfg(test)]
    pub fn get_building_id_at(&self, pos: &CoreVec2i) -> Option<BuildingId> {
        self.map.get(pos)
    }

    fn prepare_packet_info(
        &self,
        from_id: BuildingId,
    ) -> Option<(
        BuildingType,
        CoreVec2i,
        Option<bool>,
        crate::core::packet::Packet,
    )> {
        let building = self.storage.get(from_id)?;
        if !building.can_offload() {
            return None;
        }

        let packets = building.get_packets();
        let packet = packets.into_iter().next()?;
        let building_type = building.building_type();
        let source_pos = building.position();
        let filter_result = filter_packet(building, &packet);

        Some((building_type, source_pos, filter_result, packet))
    }
}

fn select_edge_for_building(
    from_id: BuildingId,
    building_type: BuildingType,
    filter_result: Option<bool>,
    counters: &mut HashMap<(BuildingId, OutputRole), usize>,
    default_edges: &[ConnectionEdge],
    match_edges: &[ConnectionEdge],
    mismatch_edges: &[ConnectionEdge],
) -> Option<ConnectionEdge> {
    let primary_choice = match (building_type, filter_result) {
        (
            BuildingType::IpFilter
            | BuildingType::PortFilter
            | BuildingType::LengthFilter
            | BuildingType::ProtocolFilter
            | BuildingType::ContentFilter,
            Some(true),
        ) => select_edge_round_robin(from_id, OutputRole::FilterMatch, counters, match_edges),
        (
            BuildingType::IpFilter
            | BuildingType::PortFilter
            | BuildingType::LengthFilter
            | BuildingType::ProtocolFilter
            | BuildingType::ContentFilter,
            Some(false),
        ) => select_edge_round_robin(
            from_id,
            OutputRole::FilterMismatch,
            counters,
            mismatch_edges,
        ),
        _ => select_edge_round_robin(from_id, OutputRole::Default, counters, default_edges),
    };

    primary_choice
        .or_else(|| select_edge_round_robin(from_id, OutputRole::Default, counters, default_edges))
        .or_else(|| {
            select_edge_round_robin(from_id, OutputRole::FilterMatch, counters, match_edges)
        })
        .or_else(|| {
            select_edge_round_robin(
                from_id,
                OutputRole::FilterMismatch,
                counters,
                mismatch_edges,
            )
        })
}

fn select_edge_round_robin(
    from_id: BuildingId,
    role: OutputRole,
    counters: &mut HashMap<(BuildingId, OutputRole), usize>,
    edges: &[ConnectionEdge],
) -> Option<ConnectionEdge> {
    if edges.is_empty() {
        return None;
    }
    let key = (from_id, role);
    let cursor = counters.get(&key).copied().unwrap_or(0);
    let index = cursor % edges.len();
    let edge = edges[index];
    counters.insert(key, (cursor + 1) % edges.len());
    Some(edge)
}

fn filter_packet(building: &dyn Building, packet: &crate::core::packet::Packet) -> Option<bool> {
    match building.building_type() {
        BuildingType::IpFilter => building
            .as_any()
            .downcast_ref::<IpFilter>()
            .map(|f| f.filter(packet)),
        BuildingType::PortFilter => building
            .as_any()
            .downcast_ref::<PortFilter>()
            .map(|f| f.filter(packet)),
        BuildingType::LengthFilter => building
            .as_any()
            .downcast_ref::<LengthFilter>()
            .map(|f| f.filter(packet)),
        BuildingType::ProtocolFilter => building
            .as_any()
            .downcast_ref::<ProtocolFilter>()
            .map(|f| f.filter(packet)),
        BuildingType::ContentFilter => building
            .as_any()
            .downcast_ref::<ContentFilter>()
            .map(|f| f.filter(packet)),
        _ => None,
    }
}

pub(crate) fn conveyor_packet_position(
    tile_pos: CoreVec2i,
    rotation: i32,
    entry_side: EntrySide,
    progress: f32,
    tile_size: f32,
) -> Vector2 {
    let base_pos = Vector2::new(tile_pos.x as f32 * tile_size, tile_pos.y as f32 * tile_size);
    let center = base_pos + Vector2::new(tile_size / 2.0, tile_size / 2.0);

    let front_dir = match rotation.rem_euclid(4) {
        0 => Vector2::new(1.0, 0.0),
        1 => Vector2::new(0.0, 1.0),
        2 => Vector2::new(-1.0, 0.0),
        3 => Vector2::new(0.0, -1.0),
        _ => Vector2::new(1.0, 0.0),
    };

    let right_dir = Vector2::new(front_dir.y, -front_dir.x);
    let left_dir = -right_dir;
    let clamped = progress.clamp(0.0, 1.0);

    match entry_side {
        EntrySide::Back => {
            let start = center - front_dir * (tile_size / 2.0);
            start + front_dir * (clamped * tile_size)
        }
        EntrySide::Left | EntrySide::Right => {
            let entry_dir = if matches!(entry_side, EntrySide::Left) {
                left_dir
            } else {
                right_dir
            };

            let first_leg = tile_size / 2.0;
            let second_leg = tile_size / 2.0;
            let total = first_leg + second_leg;
            let distance = clamped * total;

            if distance <= first_leg {
                let entry_point = center + entry_dir * (tile_size / 2.0);
                entry_point - entry_dir * distance
            } else {
                let remaining = distance - first_leg;
                center + front_dir * remaining
            }
        }
    }
}

fn building_type_from_id(building_type_id: i32) -> Option<BuildingType> {
    match building_type_id {
        building::INTERNET => Some(BuildingType::Internet),
        building::DATACENTER => Some(BuildingType::Datacenter),
        building::CONVEYOR => Some(BuildingType::Conveyor),
        building::IP_FILTER => Some(BuildingType::IpFilter),
        building::PORT_FILTER => Some(BuildingType::PortFilter),
        building::LENGTH_FILTER => Some(BuildingType::LengthFilter),
        building::PROTOCOL_FILTER => Some(BuildingType::ProtocolFilter),
        building::CONTENT_FILTER => Some(BuildingType::ContentFilter),
        building::JUNCTION => Some(BuildingType::Junction),
        building::RECYCLE_BIN => Some(BuildingType::RecycleBin),
        _ => std::option::Option::None,
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct MapController {
    pub world: RefCell<World>,
    simulation_started: bool,
    simulation_paused: bool,
    simulation_speed: f32,
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for MapController {
    fn init(base: Base<Node>) -> Self {
        Self {
            world: RefCell::new(World::new()),
            simulation_started: false,
            simulation_paused: false,
            simulation_speed: 1.0,
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        let _events = {
            let mut world = self.world.borrow_mut();
            if self.simulation_started && !self.simulation_paused {
                let scaled_delta = (delta as f32) * self.simulation_speed;
                world.update(scaled_delta);
            }
            world.drain_events()
        };
    }
}

impl MapController {
    #[cfg(test)]
    pub fn packet_to_export_row_for_test(
        packet: &crate::core::packet::Packet,
        building_id: crate::core::dto::BuildingId,
    ) -> packet_export::PacketView {
        packet_export::PacketView::from_packet(packet, building_id)
    }
}

#[godot_api]
impl MapController {
    #[signal]
    fn building_placed(info: Variant);
    #[signal]
    fn building_removed(id: i64, pos: Vector2i);
    #[signal]
    fn building_updated(updates: VariantArray);
    #[signal]
    fn packet_moved(info: Variant);

    #[func]
    pub fn load_map(&mut self, map_path: GString) -> Variant {
        self.reset_world();

        let mut loader = ResourceLoader::singleton();

        if !loader.exists(&map_path) {
            godot_error!("Map file not found at path: {}", map_path);
            return Variant::nil();
        }

        let Some(resource) = loader.load(&map_path) else {
            godot_error!("Failed to load resource from path: {}", map_path);
            return Variant::nil();
        };

        let Ok(json_resource) = resource.try_cast::<Json>() else {
            godot_error!("Loaded resource is not a valid JSON resource: {}", map_path);
            return Variant::nil();
        };

        let data = json_resource.get_data();
        let dict = match data.try_to::<Dictionary>() {
            Ok(d) => d,
            Err(_) => {
                godot_error!("Map data is not a dictionary");
                return Variant::nil();
            }
        };

        let buildings_variant = match dict.get("buildings") {
            Some(b) => b,
            None => {
                godot_error!("'buildings' key not found in map data");
                return Variant::nil();
            }
        };

        let buildings_array = match buildings_variant.try_to::<VariantArray>() {
            Ok(a) => a,
            Err(_) => {
                godot_error!("'buildings' is not an array");
                return Variant::nil();
            }
        };

        let mut world = self.world.borrow_mut();
        for building_variant in buildings_array.iter_shared() {
            let building_dict = match building_variant.try_to::<Dictionary>() {
                Ok(d) => d,
                Err(_) => {
                    godot_warn!("Building data is not a dictionary, skipping");
                    continue;
                }
            };

            let x = building_dict.get("x").and_then(|v| variant_to_i32(&v));
            let y = building_dict.get("y").and_then(|v| variant_to_i32(&v));
            let block_id = building_dict
                .get("blockId")
                .and_then(|v| variant_to_i32(&v));
            let rotation = building_dict
                .get("rotation")
                .and_then(|v| variant_to_i32(&v));

            if let (Some(x), Some(y), Some(block_id), Some(rotation)) = (x, y, block_id, rotation) {
                if let Some(building_type) = building_type_from_id(block_id) {
                    world.place_building(CoreVec2i { x, y }, building_type, rotation);
                } else {
                    godot_warn!("Invalid blockId: {}, skipping", block_id);
                }
            } else {
                godot_warn!(
                    "Missing or invalid building data, skipping: {:?}",
                    building_dict
                );
            }
        }

        world.rebuild_connections();

        let mut return_meta = Dictionary::new();
        if let Some(meta_var) = dict.get("meta")
            && let Ok(meta_dict) = meta_var.try_to::<Dictionary>()
        {
            return_meta = meta_dict.clone(); // Clone all meta fields to return

            let packets_type = meta_dict
                .get("packetsType")
                .and_then(|v| v.try_to::<GString>().ok());
            let packets_path = meta_dict
                .get("packetsPath")
                .and_then(|v| v.try_to::<GString>().ok());

            if let (Some(ptype), Some(ppath)) = (packets_type, packets_path) {
                let traffic = if ptype.to_string() == "json" {
                    let mut loader = JsonLoader::new_alloc();
                    loader.call("load_traffic", &[ppath.to_variant()])
                } else if ptype.to_string() == "pcap" {
                    let mut loader = PcapLoader::new_alloc();
                    loader.call("load_traffic", &[ppath.to_variant()])
                } else {
                    Variant::nil()
                };

                if let Ok(traffic) = traffic.try_to::<Gd<crate::packet::Traffic>>() {
                    for internet in world.storage.get_internet_buildings_mut() {
                        internet.set_traffic(traffic.clone());
                    }
                    packet_completion::register_planned_traffic(&traffic);

                    //for datacenter in world.storage.get_datacenter_buildings_mut() {
                    //    datacenter.set_traffic(traffic.clone());
                    //}  //どのタイミングでコンタミしたかは不明　仕様変更が怖いので一旦CO
                } else {
                    godot_warn!("Failed to load traffic data from path: {}", ppath);
                }
            }
        }

        return_meta.to_variant()
    }

    #[func]
    pub fn place_building(&mut self, pos: Vector2i, building_type_id: i32, rotation: i32) {
        let Some(building_type) = building_type_from_id(building_type_id) else {
            godot_warn!("Invalid building_type_id: {}", building_type_id);
            return;
        };
        self.world
            .borrow_mut()
            .place_building(pos.into(), building_type, rotation);
    }

    #[func]
    pub fn place_ip_filter(
        &mut self,
        pos: Vector2i,
        rotation: i32,
        target_ip: GString,
        direction: GString,
    ) {
        let direction_enum = match direction.to_string().as_str() {
            "source" => IpFilterDirection::Source,
            "destination" => IpFilterDirection::Destination,
            _ => {
                godot_warn!(
                    "Invalid direction: {}. Use 'source' or 'destination'",
                    direction
                );
                return;
            }
        };

        let config = IpFilterConfig {
            target_ip: target_ip.to_string(),
            direction: direction_enum,
        };

        self.world
            .borrow_mut()
            .place_ip_filter_with_config(pos.into(), rotation, config);
    }

    #[func]
    pub fn place_length_filter(
        &mut self,
        pos: Vector2i,
        rotation: i32,
        threshold: i32,
        direction: GString,
    ) {
        let direction_enum = match direction.to_string().as_str() {
            "exact" => LengthFilterDirection::Exact,
            "less_than" => LengthFilterDirection::LessThan,
            "greater_than" => LengthFilterDirection::GreaterThan,
            _ => {
                godot_warn!(
                    "Invalid direction: {}. Use 'exact', 'less_than', or 'greater_than'",
                    direction
                );
                return;
            }
        };

        let config = LengthFilterConfig {
            threshold: threshold as u32,
            direction: direction_enum,
        };

        self.world
            .borrow_mut()
            .place_length_filter_with_config(pos.into(), rotation, config);
    }

    #[func]
    pub fn place_protocol_filter(&mut self, pos: Vector2i, rotation: i32, protocol: GString) {
        let protocol_enum = match protocol.to_string().as_str() {
            "tcp" => Protocol::Tcp,
            "udp" => Protocol::Udp,
            "unknown" => Protocol::Unknown,
            _ => {
                godot_warn!("Invalid protocol: {}. Use 'tcp' or 'udp'", protocol);
                return;
            }
        };

        let config = ProtocolFilterConfig {
            protocol: protocol_enum,
        };

        self.world
            .borrow_mut()
            .place_protocol_filter_with_config(pos.into(), rotation, config);
    }

    #[func]
    pub fn place_port_filter(
        &mut self,
        pos: Vector2i,
        rotation: i32,
        target_port: i32,
        direction: GString,
    ) {
        let direction_enum = match direction.to_string().as_str() {
            "source" => PortFilterDirection::Source,
            "destination" => PortFilterDirection::Destination,
            _ => {
                godot_warn!(
                    "Invalid direction: {}. Use 'source' or 'destination'",
                    direction
                );
                return;
            }
        };

        let config = PortFilterConfig {
            target_port: target_port as u16,
            direction: direction_enum,
        };

        self.world
            .borrow_mut()
            .place_port_filter_with_config(pos.into(), rotation, config);
    }

    #[func]
    pub fn place_content_filter(&mut self, pos: Vector2i, rotation: i32, pattern: GString) {
        let config = ContentFilterConfig {
            pattern: pattern.to_string(),
        };

        self.world
            .borrow_mut()
            .place_content_filter_with_config(pos.into(), rotation, config);
    }

    #[func]
    pub fn remove_building(&mut self, pos: Vector2i) {
        let core_pos: CoreVec2i = pos.into();
        self.world.borrow_mut().remove_building(&core_pos);
    }

    #[func]
    pub fn reset_world(&mut self) {
        self.world.replace(World::new());
        self.simulation_started = false;
        self.simulation_paused = false;
        self.simulation_speed = 1.0;
        packet_completion::clear_planned_packets();
    }

    #[func]
    pub fn start_heart_beat(&mut self) {
        self.simulation_started = true;
        self.simulation_paused = false;
    }

    #[func]
    pub fn stop_the_world(&mut self) {
        if self.simulation_started {
            self.simulation_paused = true;
        }
    }

    #[func]
    pub fn resume(&mut self) {
        if self.simulation_started {
            self.simulation_paused = false;
        }
    }

    #[func]
    pub fn over_clock(&mut self, speed: f32) {
        if speed > 0.0 {
            self.simulation_speed = speed;
        } else {
            godot_warn!(
                "Invalid speed '{}'. Keeping previous value {}",
                speed,
                self.simulation_speed
            );
        }
    }

    #[func]
    pub fn get_all_buildings_by_typeid(&self, building_type_id: i32) -> VariantArray {
        let world = self.world.borrow();

        let Some(target_type) = building_type_from_id(building_type_id) else {
            godot_warn!(
                "Invalid building_type_id for get_all_buildings_by_typeid: {}",
                building_type_id
            );
            return VariantArray::new();
        };

        world
            .storage
            .iter()
            .filter(|building| building.building_type() == target_type)
            .map(|building| {
                let mut dict = Dictionary::new();
                let pos = building.position();
                dict.set("id", building.id().to_variant());
                dict.set("pos", Vector2i::from(pos).to_variant());
                dict.set("rotation", building.rotation().to_variant());
                dict.set("progress", building.get_progress().to_variant());
                dict.to_variant()
            })
            .collect()
    }

    #[func]
    pub fn get_all_buildings(&self) -> VariantArray {
        let world = self.world.borrow();
        world
            .storage
            .iter()
            .map(|building| {
                let mut dict = Dictionary::new();
                let pos = building.position();
                dict.set("id", building.id().to_variant());
                dict.set("pos", Vector2i::from(pos).to_variant());
                dict.set("type", (building.building_type() as i32).to_variant());
                dict.set("rotation", building.rotation().to_variant());
                dict.set("progress", building.get_progress().to_variant());
                dict.to_variant()
            })
            .collect()
    }

    #[func]
    pub fn get_all_packet_progress(&self) -> VariantArray {
        let world = self.world.borrow();
        world
            .storage
            .iter()
            .flat_map(|building| {
                let building_type = building.building_type();
                let tile_pos = building.position();
                let rotation = building.rotation();
                building.get_packets().into_iter().map(move |packet| {
                    let mut dict = Dictionary::new();
                    dict.set("building_type", (building_type as i32).to_variant());
                    dict.set("source_ip", packet.source_ip.to_variant());
                    dict.set("dest_ip", packet.dest_ip.to_variant());
                    dict.set("source_port", (packet.source_port as i32).to_variant());
                    dict.set("dest_port", (packet.dest_port as i32).to_variant());
                    dict.set("protocol", (packet.protocol as i32).to_variant());
                    dict.set("length", (packet.length as i32).to_variant());
                    dict.set("progress", packet.progress.to_variant());
                    dict.set("tile_pos", Vector2i::from(tile_pos).to_variant());
                    dict.set("rotation", rotation.to_variant());
                    dict.set("label", packet.label.to_raw().to_variant());
                    dict.to_variant()
                })
            })
            .collect()
    }

    #[func]
    pub fn get_all_packet_positions(&self) -> VariantArray {
        let world = self.world.borrow();
        let tile_size = 16.0;
        let mut result = VariantArray::new();

        for building in world.storage.iter() {
            if building.building_type() != BuildingType::Conveyor {
                continue;
            }

            let Some(conveyor) = building.as_any().downcast_ref::<Conveyor>() else {
                continue;
            };

            let Some((packet, entry_side)) = conveyor.buffer_state() else {
                continue;
            };

            let pos = conveyor_packet_position(
                conveyor.position(),
                conveyor.rotation(),
                entry_side,
                packet.progress,
                tile_size,
            );

            let mut dict = Dictionary::new();
            dict.set("source_ip", packet.source_ip.to_variant());
            dict.set("dest_ip", packet.dest_ip.to_variant());
            dict.set("source_port", (packet.source_port as i32).to_variant());
            dict.set("dest_port", (packet.dest_port as i32).to_variant());
            dict.set("protocol", (packet.protocol as i32).to_variant());
            dict.set("length", (packet.length as i32).to_variant());
            dict.set("position", pos.to_variant());
            dict.set("label", packet.label.to_raw().to_variant());
            let variant = dict.to_variant();
            result.push(&variant);
        }

        result
    }

    #[func]
    pub fn get_datacenter_packets(&self) -> VariantArray {
        let world = self.world.borrow();
        packet_export::datacenter_packets(&world)
    }

    #[func]
    pub fn get_recyclebin_packets(&self) -> VariantArray {
        let world = self.world.borrow();
        packet_export::recyclebin_packets(&world)
    }

    #[func]
    pub fn completed_packets(&self) -> Variant {
        let world = self.world.borrow();
        packet_completion::completed_packets_variant(&world)
    }

    #[func]
    pub fn get_score(&self) -> u32 {
        self.world.borrow().score
    }

    #[func]
    pub fn set_filter_rules(&mut self, building_id: i64, rule: Dictionary) {
        let mut world = self.world.borrow_mut();
        let Some(building) = world.storage.get_mut(building_id as u64) else {
            godot_warn!("Building with id {} not found", building_id);
            return;
        };

        let building_type = building.building_type();

        match building_type {
            BuildingType::IpFilter => {
                if let Some(filter) = building.as_any_mut().downcast_mut::<IpFilter>() {
                    let target_ip = rule
                        .get("target_ip")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|s| s.to_string());
                    let direction_str = rule
                        .get("direction")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|s| s.to_string());

                    if let (Some(target_ip), Some(direction_str)) = (target_ip, direction_str) {
                        let direction = match direction_str.as_str() {
                            "source" => IpFilterDirection::Source,
                            "destination" => IpFilterDirection::Destination,
                            _ => {
                                godot_warn!(
                                    "Invalid direction: {}. Use 'source' or 'destination'",
                                    direction_str
                                );
                                return;
                            }
                        };

                        let config = IpFilterConfig {
                            target_ip,
                            direction,
                        };
                        filter.set_config(config);
                    } else {
                        godot_warn!("Missing target_ip or direction in IP filter rule");
                    }
                }
            }
            BuildingType::PortFilter => {
                if let Some(filter) = building.as_any_mut().downcast_mut::<PortFilter>() {
                    let target_port = rule
                        .get("target_port")
                        .and_then(|v| v.try_to::<i32>().ok())
                        .map(|p| p as u16);
                    let direction_str = rule
                        .get("direction")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|s| s.to_string());

                    if let (Some(target_port), Some(direction_str)) = (target_port, direction_str) {
                        let direction = match direction_str.as_str() {
                            "source" => PortFilterDirection::Source,
                            "destination" => PortFilterDirection::Destination,
                            _ => {
                                godot_warn!(
                                    "Invalid direction: {}. Use 'source' or 'destination'",
                                    direction_str
                                );
                                return;
                            }
                        };

                        let config = PortFilterConfig {
                            target_port,
                            direction,
                        };
                        filter.set_config(config);
                    } else {
                        godot_warn!("Missing target_port or direction in Port filter rule");
                    }
                }
            }
            BuildingType::LengthFilter => {
                if let Some(filter) = building.as_any_mut().downcast_mut::<LengthFilter>() {
                    let threshold = rule
                        .get("threshold")
                        .and_then(|v| v.try_to::<i32>().ok())
                        .map(|t| t as u32);
                    let direction_str = rule
                        .get("direction")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|s| s.to_string());

                    if let (Some(threshold), Some(direction_str)) = (threshold, direction_str) {
                        let direction = match direction_str.as_str() {
                            "exact" => LengthFilterDirection::Exact,
                            "less_than" => LengthFilterDirection::LessThan,
                            "greater_than" => LengthFilterDirection::GreaterThan,
                            _ => {
                                godot_warn!(
                                    "Invalid direction: {}. Use 'exact', 'less_than', or 'greater_than'",
                                    direction_str
                                );
                                return;
                            }
                        };

                        let config = LengthFilterConfig {
                            threshold,
                            direction,
                        };
                        filter.set_config(config);
                    } else {
                        godot_warn!("Missing threshold or direction in Length filter rule");
                    }
                }
            }
            BuildingType::ProtocolFilter => {
                if let Some(filter) = building.as_any_mut().downcast_mut::<ProtocolFilter>() {
                    let protocol_str = rule
                        .get("protocol")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|s| s.to_string());

                    if let Some(protocol_str) = protocol_str {
                        let protocol = match protocol_str.as_str() {
                            "tcp" => Protocol::Tcp,
                            "udp" => Protocol::Udp,
                            "unknown" => Protocol::Unknown,
                            _ => {
                                godot_warn!(
                                    "Invalid protocol: {}. Use 'tcp', 'udp' or 'unknown'",
                                    protocol_str
                                );
                                return;
                            }
                        };

                        let config = ProtocolFilterConfig { protocol };
                        filter.set_config(config);
                    } else {
                        godot_warn!("Missing protocol in Protocol filter rule");
                    }
                }
            }
            BuildingType::ContentFilter => {
                if let Some(filter) = building.as_any_mut().downcast_mut::<ContentFilter>() {
                    let pattern = rule
                        .get("pattern")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|s| s.to_string());

                    if let Some(pattern) = pattern {
                        let config = ContentFilterConfig { pattern };
                        filter.set_config(config);
                    } else {
                        godot_warn!("Missing pattern in Content filter rule");
                    }
                }
            }
            _ => {
                godot_warn!(
                    "Building with id {} is not a filter or does not support rule setting",
                    building_id
                );
            }
        }
    }

    #[func]
    pub fn get_filter_rule(&self, building_id: i64) -> Dictionary {
        let world = self.world.borrow();
        let mut result = Dictionary::new();

        let Some(building) = world.storage.get(building_id as u64) else {
            godot_warn!("Building with id {} not found", building_id);
            return result;
        };

        let building_type = building.building_type();

        match building_type {
            BuildingType::IpFilter => {
                if let Some(filter) = building.as_any().downcast_ref::<IpFilter>()
                    && let Some(config) = &filter.config
                {
                    result.set("target_ip", config.target_ip.clone());
                    let direction_str = match config.direction {
                        IpFilterDirection::Source => "source",
                        IpFilterDirection::Destination => "destination",
                    };
                    result.set("direction", direction_str);
                }
            }
            BuildingType::PortFilter => {
                if let Some(filter) = building.as_any().downcast_ref::<PortFilter>()
                    && let Some(config) = &filter.config
                {
                    result.set("target_port", config.target_port as i32);
                    let direction_str = match config.direction {
                        PortFilterDirection::Source => "source",
                        PortFilterDirection::Destination => "destination",
                    };
                    result.set("direction", direction_str);
                }
            }
            BuildingType::LengthFilter => {
                if let Some(filter) = building.as_any().downcast_ref::<LengthFilter>()
                    && let Some(config) = &filter.config
                {
                    result.set("threshold", config.threshold as i32);
                    let direction_str = match config.direction {
                        LengthFilterDirection::Exact => "exact",
                        LengthFilterDirection::LessThan => "less_than",
                        LengthFilterDirection::GreaterThan => "greater_than",
                    };
                    result.set("direction", direction_str);
                }
            }
            BuildingType::ProtocolFilter => {
                if let Some(filter) = building.as_any().downcast_ref::<ProtocolFilter>()
                    && let Some(config) = &filter.config
                {
                    let protocol_str = match config.protocol {
                        Protocol::Tcp => "tcp",
                        Protocol::Udp => "udp",
                        Protocol::Unknown => "unknown",
                    };
                    result.set("protocol", protocol_str);
                }
            }
            BuildingType::ContentFilter => {
                if let Some(filter) = building.as_any().downcast_ref::<ContentFilter>()
                    && let Some(config) = &filter.config
                {
                    result.set("pattern", config.pattern.clone());
                }
            }
            _ => {
                godot_warn!(
                    "Building with id {} is not a filter or does not support rule getting",
                    building_id
                );
            }
        }

        result
    }
}
