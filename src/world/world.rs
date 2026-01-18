use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use anymap::AnyMap;
use glam::{Vec2, vec2};
use smallvec::SmallVec;

use crate::events::{self, EventBus, EventQueue};
use crate::physics::{HasBounds, Physics, PhysicsBody, PhysicsData};
use crate::shapes::{AABB, CollisionShape};
use crate::entity::{Actor, ID};
use crate::world::registry::Registry;
pub struct World {
    pub(crate) registry: Registry,
    pub logic_update: Duration,
    update_methods_any: AnyMap,

    event_bus: RefCell<EventBus>,

    events: EventQueue,
    physics: Physics,
}

impl World {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            update_methods_any: AnyMap::new(),
            logic_update: Duration::from_millis(16),
            physics: Physics::new(AABB { min: vec2(-2048.0, -2048.0), max: vec2(2048.0, 2048.0) }),
            event_bus: RefCell::new(EventBus::new()),
            events: EventQueue::new(),
        }
    }

    pub fn subscribe<E: 'static, T: 'static, L: 'static>(&mut self, emitter: ID<T>, listener: ID<L>, closure: impl Fn(&mut World, &E) + 'static) {
        self.events.subscribe(emitter, listener, closure);
    }

    pub fn emit<E: 'static, T: 'static>(&mut self, emitter: ID<T>, event: E) {
        let mut drained = if let Some(e) = self.events.get_listeners::<E, T>(emitter) {
            std::mem::take(&mut e.listeners)
        } else {
            Vec::new()
        };

        for f in &drained {
            f(self, &event);
        }

        if let Some(e) = self.events.get_listeners::<E, T>(emitter) {
            e.listeners.append(&mut drained);
        }
    }

    pub fn debug_get_tree(&self) -> Vec<(usize, AABB)> {
        self.physics.get_debug_info()
    }

    pub(crate) fn register_type<T: Actor<P> + 'static, P: 'static>(&mut self) {
        //self.update_methods.push(T::update_system);
        println!("INFO: registering update generic type {:?}", std::any::type_name::<P>());
        if !self.update_methods_any.contains::<Vec<fn(&mut World, &mut P)>>() {
            let update_methods: Vec<fn(&mut World, &mut P)> = Vec::new();
            self.update_methods_any.insert(update_methods);
        }

        self.update_methods_any.get_mut::<Vec<fn(&mut World, &mut P)>>().unwrap().push(T::update_system);

        Registry::create_entry::<T>();
        self.physics.register_type::<T>();
    }

    pub fn add_actor<T: Actor<P> + 'static, P: 'static>(&mut self, actor: T) -> ID<T> {
        let typeid = TypeId::of::<T>();
        if !self.registry.types.contains(&typeid) {
            self.registry.types.push(typeid);
            self.register_type::<T, P>();
        }
        let id = Registry::insert(actor);

        let body = PhysicsBody::Actor(PhysicsData {
            pos: Vec2::ZERO,
            body: Some(CollisionShape::AABB(AABB { min: Vec2::ZERO, max: vec2(32.0, 32.0)})),
            id: id.into(),
        });
        self.physics.add_body(&id, body);

        id
    }

    pub fn update_systems<P: 'static>(&mut self, ctx: &mut P) {
        let time = Instant::now();

        if let Some(systems) = self.update_methods_any.get::<Vec<fn(&mut World, &mut P)>>() {
            let runtime_systems = systems.clone();
            for system in runtime_systems {
                system(self, ctx);
            }
        } else {
            println!("WARNING: no update methods registered for the generic type {:?}", std::any::type_name::<P>());
            panic!("Please make sure the argument passed into update_systems(), \"{}\",is the same as the generic type of the actor structs", std::any::type_name::<P>());
        }

        self.physics.cleanup();
        self.logic_update = time.elapsed();
    }

    pub fn get_pos<T: 'static>(&self, id: &ID<T>) -> &Vec2 {
        let body = self.physics.get_body(id).unwrap_or(&PhysicsBody::Node{});

        match body {
            PhysicsBody::Actor(data) => &data.pos,
            PhysicsBody::Solid(data) => &data.pos,
            PhysicsBody::Zone(data) => &data.pos,
            PhysicsBody::Node => &Vec2::ZERO,
        }
    }

    pub fn set_pos<T: 'static + Actor<P>, P: 'static>(&mut self, id: ID<T>, pos: Vec2) {
        let body = self.physics.get_body(&id).unwrap();
        
        let new_body = match body {
            PhysicsBody::Actor(data) => PhysicsBody::Actor(PhysicsData { pos, body: data.body, id: id.into() }),
            PhysicsBody::Solid(data) => PhysicsBody::Solid(PhysicsData { pos, body: data.body, id: id.into() }),
            PhysicsBody::Zone(data) => PhysicsBody::Zone(PhysicsData { pos, body: data.body, id: id.into() }),
            PhysicsBody::Node => PhysicsBody::Node,
        };

        let bounds = new_body.bounds();
        self.physics.update_body(&id, new_body.clone());
        

        let mut query = SmallVec::new();
        self.physics.query(&bounds, &mut query);
        

        let this_type = id.type_id();
        for collided in query {
            let b = match collided {
                PhysicsBody::Actor(data) => data,
                PhysicsBody::Solid(data) => data,
                PhysicsBody::Zone(data) => data,
                PhysicsBody::Node => continue,
            };

            if b.id.type_id == this_type && b.id.index == id.index {
                // we don't collide with ourselves
                continue;
            }

            if new_body.overlaps(&collided) {
                let other_id = b.id.clone();
                self.with_world(&id, move |ett, world| {
                    ett.on_collision(&id, other_id, world);
                });
            }
        }
    }

    pub fn move_by<T: 'static + Actor<P>, P: 'static>(&mut self, id: ID<T>, delta: &Vec2) -> Vec2 {
        let body = self.physics.get_body(&id).unwrap();
        let new_pos = match body {
            PhysicsBody::Actor(data) => data.pos + *delta,
            PhysicsBody::Solid(data) => data.pos + *delta,
            PhysicsBody::Zone(data) => data.pos + *delta,
            PhysicsBody::Node => Vec2::ZERO,
        };
        let new_body = match body {
            PhysicsBody::Actor(data) => PhysicsBody::Actor(PhysicsData { pos: new_pos, body: data.body, id: id.into() }),
            PhysicsBody::Solid(data) => PhysicsBody::Solid(PhysicsData { pos: new_pos, body: data.body, id: id.into() }),
            PhysicsBody::Zone(data) => PhysicsBody::Zone(PhysicsData { pos: new_pos, body: data.body, id: id.into() }),
            PhysicsBody::Node => PhysicsBody::Node,
        };
        let bounds = new_body.bounds();

        self.physics.update_body(&id, new_body.clone());

        let mut query = SmallVec::new();
        self.physics.query(&bounds, &mut query);

        let this_type = id.type_id();
        for collided in query {
            let b = match collided {
                PhysicsBody::Actor(data) => data,
                PhysicsBody::Solid(data) => data,
                PhysicsBody::Zone(data) => data,
                PhysicsBody::Node => continue,
            };
            
            if b.id.type_id == this_type && b.id.index == id.index {
                continue;
            }

            if new_body.overlaps(&collided) {
                let other_id = b.id.clone();
                self.with_world(&id, move |ett, world| {
                    ett.on_collision(&id, other_id, world);
                });
            }
        }

        new_pos
    }

    /**
    Gets an immutable reference to the entity with the given ID

    You should not modify other entities directly inside update methods.
    If you need to mutate properties on another entity, queue an action on it using the `world.with()` method.
    */
    pub fn get<'a, T: 'static>(&self, id: &'a ID<T>) -> Option<&'a T> {
        Some(&Registry::get(id)?.1)
    }

    pub(crate) fn get_mut<'a, T: 'static>(&mut self, id: &'a ID<T>) -> Option<&'a mut T> {
        Some(&mut Registry::get_mut(id)?.1)
    }

    pub fn with<T: 'static>(&self, id: &ID<T>, f: impl Fn(&mut T) + 'static) {
        let id = id.clone();
        let closure = move |_world: &mut World| {
            let entry = Registry::get_mut(&id);
            if let Some(entity) = entry {
                f(&mut entity.1);
            } else {
                println!("with(entity) not found: {:?}", id.clone());
                println!("perhaps already in use?");
            }
        };

        self.event_bus.borrow_mut().push(Box::new(closure));
    }

    pub fn with_world<T: 'static>(&self, id: &ID<T>, f: impl Fn(&mut T, &mut World) + 'static) {
        let id = id.clone();
        let closure = move |world: &mut World| {
            let entry = Registry::get_mut(&id);

            if let Some(entity) = entry {
                f(&mut entity.1, world);
            } else {
                println!("with_world(entity) not found: {:?}", id.clone());
                println!("perhaps already in use?");
            }
        };

        self.event_bus.borrow_mut().push(Box::new(closure));
    }

    pub fn query<T: 'static>(&self) -> impl Iterator<Item = &T> + use<'_, T> {
       Registry::get_entry::<T>().arena.iter().map(|(_index, item)| &item.1)
    }

    pub fn query_id<T: 'static>(&self) -> impl Iterator<Item = &(ID<T>,T)> + use<'_, T> {
        Registry::get_entry::<T>().arena.iter().map(|(_index, item)| item)
    }

    pub fn query_mut<T: 'static>(&mut self) -> impl Iterator<Item = &mut T> + use<'_, T> {
        Registry::get_entry_mut::<T>().arena.iter_mut().map(|(_index, item)| &mut item.1)
    }

    pub fn query_id_mut<T: 'static>(&mut self) -> impl Iterator<Item = &mut (ID<T>,T)> + use<'_, T> {
        Registry::get_entry_mut::<T>().arena.iter_mut().map(|(_index, item)| item)
    }

    pub(crate) fn flush_events(&mut self) {
        // we hoist/drain the event bus, so that when executing these,
        // new events can still be added to the bus
        let events = std::mem::take(&mut self.event_bus.borrow_mut().events);
        if events.is_empty() {return}
        for event in events {
            event(self);
        }
    }

}