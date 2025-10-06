use crate::core::building::{Building, BuildingType};
use crate::core::dto::{BuildingId, Vec2i};
use crate::logic::building_map::BuildingMap;
use crate::logic::building_storage::BuildingStorage;
use std::collections::HashMap;

pub struct ConnectionGraph {
    outputs: HashMap<BuildingId, Vec<ConnectionEdge>>, // from_id -> to_id with role
}

impl Default for ConnectionGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionGraph {
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }

    pub fn update_connections_at(
        &mut self,
        _pos: Vec2i,
        _map: &BuildingMap,
        _storage: &BuildingStorage,
    ) {
        // Placeholder for incremental update
    }

    pub fn remove_node(&mut self, id: BuildingId) {
        self.outputs.remove(&id);

        self.outputs.retain(|_, edges| {
            edges.retain(|edge| edge.to_id != id);
            !edges.is_empty()
        });
    }

    pub fn get_outputs(&self, from_id: BuildingId) -> Option<&Vec<ConnectionEdge>> {
        self.outputs.get(&from_id)
    }

    /// 再構築時点の建物配置から接続グラフを構築する。
    ///
    /// ソース建物が出力として提示するタイルに別建物が存在し、かつターゲット建物が
    /// 入力として許可するタイルにソース建物が存在する場合にのみ、有向エッジを追加する。
    /// これにより、建物ごとの出力・入力ポート設定を尊重した接続関係を得られる。
    pub fn rebuild(&mut self, map: &BuildingMap, storage: &BuildingStorage) {
        self.outputs.clear();
        for from_building in storage.iter() {
            let from_id = from_building.id();

            let mut connections = Vec::new();

            for out_pos in from_building.get_output_poses() {
                if let Some(to_id) = map.get(&out_pos) {
                    if to_id == from_id {
                        continue;
                    }

                    if let Some(target_building) = storage.get(to_id) {
                        let accepts_from_source = target_building
                            .get_input_poses()
                            .into_iter()
                            .any(|input_pos| map.get(&input_pos) == Some(from_id));

                        if accepts_from_source {
                            let role = classify_output_role(from_building, out_pos);
                            connections.push(ConnectionEdge { to_id, role });
                        }
                    }
                }
            }

            // 重複を排除し, ソートしてから追加
            if !connections.is_empty() {
                connections.sort_unstable_by_key(|a| edge_sort_key(a));
                connections.dedup_by(|a, b| edge_sort_key(a) == edge_sort_key(b));
                self.outputs.insert(from_id, connections);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutputRole {
    Default,
    FilterMatch,
    FilterMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConnectionEdge {
    pub to_id: BuildingId,
    pub role: OutputRole,
}

fn edge_sort_key(edge: &ConnectionEdge) -> (BuildingId, i32) {
    (edge.to_id, role_index(edge.role))
}

fn role_index(role: OutputRole) -> i32 {
    match role {
        OutputRole::Default => 0,
        OutputRole::FilterMatch => 1,
        OutputRole::FilterMismatch => 2,
    }
}

fn classify_output_role(building: &dyn Building, out_pos: Vec2i) -> OutputRole {
    match building.building_type() {
        BuildingType::IpFilter
        | BuildingType::PortFilter
        | BuildingType::LengthFilter
        | BuildingType::ProtocolFilter
        | BuildingType::ContentFilter => classify_filter_output(building, out_pos),
        _ => OutputRole::Default,
    }
}

fn classify_filter_output(building: &dyn Building, out_pos: Vec2i) -> OutputRole {
    let pos = building.position();
    let rot = building.rotation().rem_euclid(4);

    let front = match rot {
        0 => Vec2i {
            x: pos.x,
            y: pos.y - 1,
        },
        1 => Vec2i {
            x: pos.x + 1,
            y: pos.y,
        },
        2 => Vec2i {
            x: pos.x,
            y: pos.y + 1,
        },
        3 => Vec2i {
            x: pos.x - 1,
            y: pos.y,
        },
        _ => Vec2i {
            x: pos.x,
            y: pos.y - 1,
        },
    };

    if out_pos == front {
        OutputRole::FilterMismatch
    } else {
        OutputRole::FilterMatch
    }
}
