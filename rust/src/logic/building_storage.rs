use crate::core::building::Building;
use crate::core::buildings::datacenter::Datacenter;
use crate::core::buildings::internet::Internet;
use crate::core::dto::BuildingId;
use std::collections::HashMap;

pub struct BuildingStorage {
    buildings: HashMap<BuildingId, Box<dyn Building>>,
}

impl Default for BuildingStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildingStorage {
    pub fn new() -> Self {
        Self {
            buildings: HashMap::new(),
        }
    }

    pub fn add_building(&mut self, building: Box<dyn Building>) {
        self.buildings.insert(building.id(), building);
    }

    pub fn get(&self, id: BuildingId) -> Option<&dyn Building> {
        self.buildings.get(&id).map(|b| &**b)
    }

    pub fn get_mut(&mut self, id: BuildingId) -> Option<&mut dyn Building> {
        // ポインタを経由することで、ライフタイムの問題を回避する
        self.buildings.get_mut(&id).map(|b| {
            let ptr = b.as_mut() as *mut dyn Building;
            unsafe { &mut *ptr }
        })
    }

    pub fn get_two_mut(
        &mut self,
        id1: BuildingId,
        id2: BuildingId,
    ) -> Option<(&mut dyn Building, &mut dyn Building)> {
        if id1 == id2 {
            return None;
        }

        let ptr1 = self.buildings.get_mut(&id1)? as *mut Box<dyn Building>;
        let ptr2 = self.buildings.get_mut(&id2)? as *mut Box<dyn Building>;

        // 2つのポインタが異なる場所を指していることは確認済みなので安全
        unsafe { Some((&mut **ptr1, &mut **ptr2)) }
    }

    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Building> + 'a> {
        Box::new(self.buildings.values().map(|b| &**b))
    }

    pub fn iter_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn Building> + 'a> {
        // ポインタを経由することで、ライフタイムの問題を回避する
        let ptrs: Vec<*mut dyn Building> = self
            .buildings
            .values_mut()
            .map(|b| b.as_mut() as *mut dyn Building)
            .collect();
        Box::new(ptrs.into_iter().map(|ptr| unsafe { &mut *ptr }))
    }

    pub fn get_internet_buildings_mut(&mut self) -> Vec<&mut Internet> {
        self.buildings
            .values_mut()
            .filter_map(|b| b.as_any_mut().downcast_mut::<Internet>())
            .collect()
    }

    pub fn get_datacenter_buildings_mut(&mut self) -> Vec<&mut Datacenter> {
        self.buildings
            .values_mut()
            .filter_map(|b| b.as_any_mut().downcast_mut::<Datacenter>())
            .collect()
    }

    pub fn remove(&mut self, id: BuildingId) -> Option<Box<dyn Building>> {
        self.buildings.remove(&id)
    }
}
