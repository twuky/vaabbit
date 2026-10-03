use std::any::TypeId;

use anymap::AnyMap;
use rustc_hash::FxHashMap;
use vibarena::ArenaMap;

use crate::{Actor, TypedID};

#[derive(Clone, Debug, Default)]
pub struct TypedCollection<T: Default> {
    map: FxHashMap<TypeId, vibarena::ArenaMap<T>>
}

impl<T: Default> TypedCollection<T> where T: Default {
    pub fn new() -> Self {
        Self {
            map: FxHashMap::default(),
        }
    }

    pub fn get(&self, id: TypedID) -> Option<&T> {
        self.map.get(&id.type_id).and_then(|m| m.get(&id.index))
    }

    pub fn get_mut(&mut self, id: TypedID) -> Option<&mut T> {
        self.map.get_mut(&id.type_id).and_then(|m| m.get_mut(&id.index))
    }

    pub fn get_all(&self, type_id: TypeId) -> Option<&vibarena::ArenaMap<T>> {
        self.map.get(&type_id)
    }

    pub fn get_all_mut(&mut self, type_id: TypeId) -> Option<&mut vibarena::ArenaMap<T>> {
        self.map.get_mut(&type_id)
    }

    pub fn insert(&mut self, id: TypedID, value: T) {
        let map = self.map.entry(id.type_id).or_default();
        map.insert(id.index, value);
    }

    pub fn remove(&mut self, id: TypedID) {
        let map = self.map.get_mut(&id.type_id).unwrap();
        map.remove(&id.index);
    }
}