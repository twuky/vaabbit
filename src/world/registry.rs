use rustc_hash::{FxHashSet};
use vibarena::{Arena, KeySet};
use std::{any::TypeId, cell::OnceCell};
use crate::{TypedID, World,ID};
use crate::collections::TypedCollection;

pub(crate) struct RegistryEntry<T> {
    pub arena: Arena<(ID<T>,T)>,
    pub entities: KeySet,

    pub z_index: Arena<i32>,
    pub is_enabled: Arena<bool>,
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
pub(crate) struct Registry {
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

    pub fn register_type<T: 'static>() -> &'static mut RegistryEntry<T> {
        let &mut entry;
        let map = Self::get_map();

        if !map.contains::<RegistryEntry<T>>() {
            let arena = Arena::<(ID<T>,T)>::with_capacity(1024);
            let mut entities = KeySet::default();
            entities.reserve(1024);
            let z_index = Arena::default();
            let is_enabled = Arena::default();

            entry = RegistryEntry {
                arena,
                entities,
                z_index,
                is_enabled
            };
            
            map.insert(entry);
        }

        map.get_mut::<RegistryEntry<T>>().unwrap()
    }


    pub fn insert_actor<T: 'static>(entity: T) -> ID<T> {
        let entry = Self::register_type();

        let idx = entry.arena.insert_with_key(|idx| {
            (ID::new(idx), entity)
        });

        let id = ID::new(idx);
        entry.entities.insert(idx);
        
        let k = entry.z_index.insert(0);
        assert!(k == idx);
        let y = entry.is_enabled.insert(true);
        assert!(y == idx);

        id
    }

    pub fn remove_actor<T: 'static>(id: &ID<T>) -> Option<T> {
        let entry = Self::get_entry_mut::<T>();
        let entity = entry.arena.remove(id.index)?;
        entry.entities.remove(&id.index);
        entry.z_index.remove(id.index);
        entry.is_enabled.remove(id.index);
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