use rustc_hash::{FxHashSet, FxHashMap};
use vibarena::{Arena, ArenaMap, Key, KeySet};
use std::{any::TypeId, cell::OnceCell, default, num::NonZero};
use crate::{TypedID, World, entity::{ID, TypedCollection}};


pub(crate) struct RegistryEntry<T> {
    pub arena: Arena<(ID<T>,T)>,
    pub entities: KeySet,
}

impl<T: 'static> RegistryEntry<T> {
    pub fn iter_actors<P: 'static>(&mut self, world: &mut World, ctx: &mut P, closure: impl Fn(&mut World, &mut P, &mut ID<T>, &mut T) + 'static) {

        for id in &self.entities{
            if let Some((id, actor)) = self.arena.get_mut(*id) {
                closure(world, ctx, id, actor);
            }
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct EntityData {
    pub is_enabled: TypedCollection<bool>,
    pub z_index: TypedCollection<i32>,
}

#[derive(Debug, Default)]
pub(crate) struct Registry {
    pub data: EntityData,
    pub types: FxHashSet<TypeId>,
    pub recently_removed: FxHashSet<TypedID>,
}

static mut MAP: OnceCell<anymap::AnyMap> = OnceCell::new();


impl Registry {
    pub fn new() -> Self {
        unsafe {
            MAP.get_or_init(|| {anymap::AnyMap::new()});
        }
        Self {..Default::default()}
    }

    #[inline(always)]
    fn get_map() -> &'static mut anymap::AnyMap {
        unsafe {MAP.get_mut().unwrap_unchecked()}
    }

    #[inline(always)]
    pub fn get_entry<T: 'static>() -> &'static RegistryEntry<T> {
        unsafe {MAP.get_mut().unwrap_unchecked()}.get::<RegistryEntry<T>>().unwrap()
    }

    #[inline(always)]
    pub fn get_entry_mut<T: 'static>() -> &'static mut RegistryEntry<T> {
        unsafe {MAP.get_mut().unwrap_unchecked()}.get_mut::<RegistryEntry<T>>().unwrap()
    }

    pub fn create_entry<T: 'static>() -> &'static mut RegistryEntry<T> {
        let &mut entry;
        let map = Self::get_map();

        if !map.contains::<RegistryEntry<T>>() {
            let arena = Arena::<(ID<T>,T)>::with_capacity(1024);
            let mut entities = KeySet::default();
            entities.reserve(1024);

            entry = RegistryEntry {
                arena,
                entities,
            };
            
            map.insert(entry);
        }

        map.get_mut::<RegistryEntry<T>>().unwrap()
    }


    pub fn insert_actor<T: 'static>(entity: T) -> ID<T> {
        let entry = Self::create_entry();

        let idx = entry.arena.insert_with_key(|idx| {
            (ID::new(idx), entity)
        });

        let id = ID::new(idx);
        entry.entities.insert(idx);

        id
    }

    pub fn remove_actor<T: 'static>(id: &ID<T>) -> Option<T> {
        let entry = Self::get_entry_mut::<T>();
        let entity = entry.arena.remove(id.index)?;
        entry.entities.remove(&id.index);
        Some(entity.1)
    }

    pub fn get<T: 'static>(id: &ID<T>) -> Option<&(ID<T>,T)> {
        let entry = Self::get_map().get::<RegistryEntry<T>>().unwrap();
        entry.arena.get(id.index)
    }

    // gets first entity that matches type
    pub fn get_first<T: 'static>() -> Option<&'static (ID<T>, T)> {
        let entry = Self::get_map().get::<RegistryEntry<T>>()?;
        let key = entry.entities.iter().next()?;
        entry.arena.get(*key)
    }

    pub fn get_mut<T: 'static>(id: &ID<T>) -> Option<&mut (ID<T>,T)> {
        let entry = Self::get_map().get_mut::<RegistryEntry<T>>().unwrap();
        entry.arena.get_mut(id.index)
    }

    pub fn iter_actors<T: 'static, P: 'static>(world: &mut World, ctx: &mut P, closure: impl Fn(&mut World, &mut P, &mut ID<T>, &mut T) + 'static) {
        let entry = Self::get_map().get_mut::<RegistryEntry<T>>().unwrap();
        entry.iter_actors(world, ctx, closure);
    }
    
}