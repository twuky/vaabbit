use slotmap::{SlotMap};
use std::{any::TypeId, cell::OnceCell};
use crate::entity::ID;


pub(crate) struct RegistryEntry<T> {
    pub arena: SlotMap<slotmap::DefaultKey,(ID<T>,T)>,
    pub entities: Vec<ID<T>>,
}

pub(crate) struct Registry {
    pub types: Vec<TypeId>,
    //pub map: anymap::AnyMap,
}

static mut MAP: OnceCell<anymap::AnyMap> = OnceCell::new();

impl Registry {
    pub fn new() -> Self {
        unsafe {
            MAP.get_or_init(|| {anymap::AnyMap::new()});
        }
        Self {
            types: Vec::new(),
            //map: anymap::AnyMap::new(),
        }
    }

    fn get_map() -> &'static mut anymap::AnyMap {
        unsafe {MAP.get_mut().unwrap()}
    }

    pub fn get_entry<T: 'static>() -> &'static RegistryEntry<T> {
        Self::get_map().get::<RegistryEntry<T>>().unwrap()
    }

    pub fn get_entry_mut<T: 'static>() -> &'static mut RegistryEntry<T> {
        Self::get_map().get_mut::<RegistryEntry<T>>().unwrap()
    }

    pub fn create_entry<T: 'static>() -> &'static mut RegistryEntry<T> {
        let &mut entry;

        if !Self::get_map().contains::<RegistryEntry<T>>() {
            let arena = SlotMap::<slotmap::DefaultKey,(ID<T>,T)>::with_capacity(1024);
            let entities = Vec::new();

             entry = RegistryEntry {
                arena,
                entities,
            };

            Self::get_map().insert(entry);
        }

        Self::get_map().get_mut::<RegistryEntry<T>>().unwrap()
    }

    pub fn insert<T: 'static>(entity: T) -> ID<T> {
        let entry = Self::create_entry();

        let idx = entry.arena.insert_with_key(|idx| {
            (ID::new(idx), entity)
        });

        let id = ID::new(idx);
        entry.entities.push(id.clone());

        id
    }

    pub fn get<T: 'static>(id: &ID<T>) -> Option<&(ID<T>,T)> {
        let entry = Self::get_map().get::<RegistryEntry<T>>().unwrap();
        entry.arena.get(id.index)
    }

    pub fn get_mut<T: 'static>(id: &ID<T>) -> Option<&mut (ID<T>,T)> {
        let entry = Self::get_map().get_mut::<RegistryEntry<T>>().unwrap();
        entry.arena.get_mut(id.index)
    }
}