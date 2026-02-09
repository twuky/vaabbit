use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use anymap::AnyMap;
use glam::{Vec2, vec2};
use smallvec::SmallVec;

use crate::events::{self, EventBus, EventQueue};
use crate::physics::{HasBounds, Physics, PhysicsBody, PhysicsClass};
use crate::shapes::{AABB, Collider};
use crate::entity::{Actor, ID};
use crate::world::registry::Registry;
pub struct World {
    pub(crate) registry: Registry,
    pub logic_update: Duration,
    update_methods_any: AnyMap,

    event_bus: RefCell<EventBus>,

    events: EventQueue,
    pub(crate)physics: Physics,
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

        let body = PhysicsBody::new(Vec2::ZERO, Some(Collider::AABB(AABB { min: Vec2::ZERO, max: vec2(32.0, 32.0)})), id.into(), PhysicsClass::Actor);
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

    pub fn get_pos<T: 'static>(&self, id: &ID<T>) -> Vec2 {
        if let Some(body) = self.physics.get_body(id) {
            return body.pos();
        }
        Vec2::ZERO
    }

    pub fn set_pos<T: 'static + Actor<P>, P: 'static>(&mut self, id: ID<T>, pos: Vec2) {
        let mut new_body = *self.physics.get_body(&id).unwrap();
        let old_pos = new_body.pos();
        new_body.set_pos(&pos);

        // our tree contains "fat" bounding boxes, so if movement is small,
        // we dont need to rebalance the tree
        if pos.distance(old_pos) < crate::physics::TREE_BOUNDS_PADDING {
            // fast path
            self.physics.update_body_in_place(&id, new_body);
        } else {
            self.physics.update_body(&id, new_body);
        }
        let bounds = new_body.bounds();
        
        // perform broad phase collision
        let mut query = SmallVec::new();
        self.physics.query_against_id(&bounds, &mut query, id.into_typed_id());
        
        for collided in query {
            // near phase collision
            if new_body.overlaps(&collided) {
                let other_id = collided.id;
                self.with_world(&id, move |ett, world| {
                    ett.on_collision(&id, other_id, world);
                });
            }
        }
    }

    pub fn move_by<T: 'static + Actor<P>, P: 'static>(&mut self, id: ID<T>, delta: &Vec2) -> Vec2 {
        let mut new_body = *self.physics.get_body(&id).unwrap();
        new_body.translate(delta);

        let new_pos = new_body.pos();
        let bounds = new_body.bounds();

        // our tree contains "fat" bounding boxes, so if movement is small,
        // we dont need to rebalance the tree
        if delta.length() < crate::physics::TREE_BOUNDS_PADDING {
            // fast path
            self.physics.update_body_in_place(&id, new_body);
        } else {
            self.physics.update_body(&id, new_body);
        }

        let mut query = SmallVec::new();
        self.physics.query_against_id(&bounds, &mut query, id.into_typed_id());

        let overlap_list = self.physics.get_overlap_list(&id);
        // new objects we are overlapping with after movement
        let mut new_overlaps = Vec::new();
        // objects we are no longer overlapping with after movement
        let mut overlap_exits = Vec::new();

        // any IDs that are in the overlap list but not in the query are no longer overlapping
        for ov_id in overlap_list {
            if !query.iter().any(|c| c.id == *ov_id) {
                overlap_exits.push(*ov_id);
            }
        }

        // lifecycle: collision start
        for collided in query {
            if overlap_list.contains(&collided.id) { continue; }
            // near phase collision
            if new_body.overlaps(&collided) {
                let other_id = collided.id;
                new_overlaps.push(other_id);
                self.with_world(&id, move |ett, world| {
                    ett.on_collision(&id, other_id, world);
                });
            }
        }

        for other_id in &new_overlaps {
            self.physics.add_late_collision_enter(*other_id, id.into_typed_id());
        }

        // lifecycle: collision end
        for other_id in &overlap_exits {
            let o_id = other_id.clone();
            self.with_world(&id, move |ett, world| {
                ett.on_collision_end(&id, o_id, world);
            });
            self.physics.add_late_collision_exit(*other_id, id.into_typed_id());
        }

        self.physics.update_overlap_list(&id, new_overlaps, overlap_exits);

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