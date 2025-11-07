use core::time;
use std::{any::{type_name, TypeId}, collections::HashMap, hash::Hash, iter::Map, marker::PhantomData, ops::Deref, time::{Duration, Instant}};
use std::cell::RefCell;

use glam::{vec2, Vec2};
use pulz_arena::{Arena, Mirror};
use smallvec::SmallVec;
use crate::{events::event::EventQueue, physics::{HasBounds, physics::{Physics, PhysicsBody, PhysicsData}}, shapes::{AABB, AABBI32, CollisionShape}, world::{self, actor::Actor}};

pub mod actor;

#[derive(Debug)]
pub struct ID<T: ?Sized> {
    pub index: pulz_arena::Index,
    pub _type: std::marker::PhantomData<T>,
}

impl<T: 'static> ID<T> {
    pub fn new(index: pulz_arena::Index) -> Self {
        Self {
            index,
            _type: std::marker::PhantomData,
        }
    }

    pub fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
    pub fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}

impl<T> Clone for ID<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _type: std::marker::PhantomData,
        }
    }
    
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}
impl <T> Copy for ID<T> {}

impl <T> Eq for ID<T> {
    
}

impl <T> Hash for ID<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl <T> PartialEq for ID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index

    }
}

impl PartialEq for ID<dyn Actor> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

pub struct RegistryEntry<T> {
    pub arena: Arena<(ID<T>,T)>,
    pub entities: Vec<ID<T>>,
}

pub struct Registry {
    pub types: Vec<TypeId>,
    pub map: anymap::AnyMap,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            map: anymap::AnyMap::new(),
        }
    }

    pub fn get_entry<T: 'static>(&self) -> &RegistryEntry<T> {
        self.map.get::<RegistryEntry<T>>().unwrap()
    }

    pub fn get_entry_mut<T: 'static>(&mut self) -> &mut RegistryEntry<T> {
        self.map.get_mut::<RegistryEntry<T>>().unwrap()
    }

    pub fn create_entry<T: 'static>(&mut self) -> &mut RegistryEntry<T> {
        let &mut entry;

        if !self.map.contains::<RegistryEntry<T>>() {
            let arena = Arena::<(ID<T>,T)>::new();
            let entities = Vec::new();

             entry = RegistryEntry {
                arena,
                entities,
            };
            
            self.map.insert(entry);
        }

        self.map.get_mut::<RegistryEntry<T>>().unwrap()
    }

    pub fn insert<T: 'static>(&mut self, entity: T) -> ID<T> {
        let entry = self.create_entry();

        let idx = entry.arena.insert_with(|idx| {
            (ID::new(idx), entity)
        });
        
        let id = ID::new(idx);
        entry.entities.push(id.clone());

        id
    }

    pub fn get<T: 'static>(&self, id: &ID<T>) -> Option<&(ID<T>,T)> {
        let entry = self.map.get::<RegistryEntry<T>>().unwrap();
        entry.arena.get(id.index)
    }

    pub fn get_mut<T: 'static>(&mut self, id: &ID<T>) -> Option<&mut (ID<T>,T)> {
        let entry = self.map.get_mut::<RegistryEntry<T>>().unwrap();
        entry.arena.get_mut(id.index)
    }
}



struct EventBus {
    events: Vec<Box<dyn FnOnce(&mut World)>>,
}

impl EventBus {
    const fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
}

static mut EVENT_BUS: EventBus = EventBus::new();

pub struct World {
    pub registry: Registry,
    pub logic_update: Duration,
    update_methods: Vec<fn(&mut World)>,

    events: EventQueue,
    physics: Physics,
    
}


static mut query: Vec<&PhysicsBody> = Vec::new();

impl World {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            update_methods: Vec::new(),
            logic_update: Duration::from_millis(16),
            physics: Physics::new(AABB { min: vec2(-2048.0, -2048.0), max: vec2(2048.0, 2048.0) }),
            events: EventQueue::new(),
        }
    }


    pub fn subscribe<E: 'static, T: 'static, L: 'static>(&mut self, emitter: ID<T>, listener: ID<L>, closure: impl Fn(&E) + 'static) {
        self.events.subscribe(emitter, listener, closure);
    }

    pub fn emit<E: 'static, T: 'static>(&mut self, emitter: ID<T>, event: E) {
        let events = self.events.get_listeners::<E, T>(emitter);
        if let Some(events) = events {
            for f in events.listeners.iter_mut() {
                f(&event);
            }
        }
    }

    pub fn debug_get_tree(&self) -> Vec<(usize, AABBI32)> {
        self.physics.get_debug_info()
    }

    pub(crate) fn register_type<T: Actor + 'static>(&mut self) {
        self.update_methods.push(T::update_system);
        self.registry.create_entry::<T>();
        self.physics.register_type::<T>();
    }

    pub fn add_actor<T: Actor + 'static>(&mut self, actor: T) -> ID<T> {
        let typeid = TypeId::of::<T>();
        if !self.registry.types.contains(&typeid) {
            self.registry.types.push(typeid);
            self.register_type::<T>();
        }
        let id = self.registry.insert(actor);

        let body = PhysicsBody::Actor(PhysicsData {
            pos: Vec2::ZERO,
            body: Some(CollisionShape::AABB(AABB { min: Vec2::ZERO, max: vec2(32.0, 32.0)})),
        });
        self.physics.add_body(&id, body);

        id
    }

    pub fn update_systems(&mut self) {
        let time = Instant::now();
        let systems = self.update_methods.clone();
        for system in systems {
            system(self);
        }

        self.physics.cleanup();
        self.logic_update = time.elapsed();
    }

    pub fn get_pos<T: 'static>(&self, id: &ID<T>) -> &Vec2 {
        let body = self.physics.get_body(id).unwrap();
        match body {
            PhysicsBody::Actor(data) => &data.pos,
            PhysicsBody::Solid(data) => &data.pos,
            PhysicsBody::Zone(data) => &data.pos,
            PhysicsBody::Node => &Vec2::ZERO,
        }
    }

    pub fn set_pos<T: 'static + Actor>(&mut self, id: &ID<T>, pos: Vec2) {
        let body = self.physics.get_body(id).unwrap();
        let new_body = match body {
            PhysicsBody::Actor(data) => PhysicsBody::Actor(PhysicsData { pos, body: data.body }),
            PhysicsBody::Solid(data) => PhysicsBody::Solid(PhysicsData { pos, body: data.body }),
            PhysicsBody::Zone(data) => PhysicsBody::Zone(PhysicsData { pos, body: data.body }),
            PhysicsBody::Node => PhysicsBody::Node,
        };
        let bounds = new_body.bounds();
        self.physics.update_body(id, new_body);

        let mut q = SmallVec::new();
        self.physics.query(&bounds, &mut q);

        for body in q {
            if let PhysicsBody::Actor(data) = body {
                if data.body.is_some() && data.bounds().overlaps_aabb(&bounds) {
                    self.with(id, |ett| {
                        ett.on_collision();
                    })
                }
            }
        }
    }

    pub fn move_by<T: 'static + Actor>(&mut self, id: &ID<T>, delta: &Vec2) -> Vec2 {
        let body = self.physics.get_body(id).unwrap();
        let new_pos = match body {
            PhysicsBody::Actor(data) => data.pos + delta,
            PhysicsBody::Solid(data) => data.pos + delta,
            PhysicsBody::Zone(data) => data.pos + delta,
            PhysicsBody::Node => Vec2::ZERO,
        };
        let new_body = match body {
            PhysicsBody::Actor(data) => PhysicsBody::Actor(PhysicsData { pos: new_pos, body: data.body }),
            PhysicsBody::Solid(data) => PhysicsBody::Solid(PhysicsData { pos: new_pos, body: data.body }),
            PhysicsBody::Zone(data) => PhysicsBody::Zone(PhysicsData { pos: new_pos, body: data.body }),
            PhysicsBody::Node => PhysicsBody::Node,
        };
        let bounds = new_body.bounds();
            
        self.physics.update_body(id, new_body);
        
        let mut q = SmallVec::new();
        self.physics.query(&bounds, &mut q);

        // if theres more than one body in the query, theres a collision (probably lol)
        if q.len() > 1 {
            self.with(id, |ett| {
                ett.on_collision();
            });
        }
        new_pos
    }

    /**
    Gets an immutable reference to the entity with the given ID

    You should not modify other entities directly inside update methods.
    If you need to mutate properties on another entity, queue an action on it using the `world.with()` method.
    */
    pub fn get<T: 'static>(&self, id: &ID<T>) -> &T {
        &self.registry.get(id).unwrap().1
    }

    fn get_mut<T: 'static>(&mut self, id: &ID<T>) -> &mut T {
        &mut self.registry.get_mut(id).unwrap().1
    }

    pub fn with<T: 'static>(&self, id: &ID<T>, f: impl FnOnce(&mut T) + 'static) {
        let id = id.clone();
        let closure = move |world: &mut World| {
            let mut entity = world.get_mut(&id.clone());
            f(&mut entity);
        };

        unsafe { EVENT_BUS.events.push(Box::new(closure)) }
    }

    pub fn query<T: 'static>(&self) -> impl Iterator<Item = &T> + use<'_, T> {
        self.registry.get_entry::<T>().arena.iter().map(|(_index, item)| &item.1)
    }

    pub fn query_id<T: 'static>(&self) -> impl Iterator<Item = &(ID<T>,T)> + use<'_, T> {
        self.registry.get_entry::<T>().arena.iter().map(|(_index, item)| item)
    }

    pub fn query_mut<T: 'static>(&mut self) -> impl Iterator<Item = &mut T> + use<'_, T> {
        self.registry.get_entry_mut::<T>().arena.iter_mut().map(|(_index, item)| &mut item.1)
    }

    pub fn query_id_mut<T: 'static>(&mut self) -> impl Iterator<Item = &mut (ID<T>,T)> + use<'_, T> {
        self.registry.get_entry_mut::<T>().arena.iter_mut().map(|(_index, item)| item)
    }

    fn flush_events(&mut self) {
        if unsafe { EVENT_BUS.events.len() } == 0 {return}
        for event in unsafe { EVENT_BUS.events.drain(..) } {
            event(self);
        }
    }
   
}