use crate::core::dto::{BuildingId, Vec2i};
use std::collections::HashMap;

pub struct BuildingMap {
    map: HashMap<Vec2i, BuildingId>,
}

impl Default for BuildingMap {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildingMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, pos: &Vec2i) -> Option<BuildingId> {
        self.map.get(pos).cloned()
    }

    pub fn insert(&mut self, pos: Vec2i, id: BuildingId) {
        self.map.insert(pos, id);
    }

    pub fn remove(&mut self, pos: &Vec2i) -> Option<BuildingId> {
        self.map.remove(pos)
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}
