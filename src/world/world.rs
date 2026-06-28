use std::any::TypeId;
use std::cell::RefCell;
use std::time::{Duration, Instant};

use anymap::AnyMap;
use glam::{vec2};

use crate::TypedID;
use crate::events::{EventBus, EventQueue};
use crate::physics::{Physics};
use crate::shapes::AABB;
use crate::entity::{Actor, ID};
use crate::world::registry::Registry;
pub struct World {
    pub(crate) registry: Registry,
    pub logic_update: Duration,
    update_methods_any: AnyMap,

    event_bus: RefCell<EventBus>,

    events: EventQueue,
    pub(crate)physics: Physics,
    singletons: AnyMap,

    pub(crate) current_actor: Option<TypedID>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
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
            singletons: AnyMap::new(),

            current_actor: None,
        }
    }

    pub fn subscribe<E: 'static, T: 'static, L: 'static>(&mut self, emitter: ID<T>, listener: ID<L>, closure: impl Fn(&mut World, &E) + 'static) {
        self.events.subscribe(emitter, listener, closure);
    }

    pub(crate) fn emit<E: 'static, T: 'static>(&mut self, emitter: ID<T>, event: E) {
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
        println!("INFO: registering update<{:?}> for {:?}", std::any::type_name::<P>(), std::any::type_name::<T>());
        if !self.update_methods_any.contains::<Vec<fn(&mut World, &mut P)>>() {
            let update_methods: Vec<fn(&mut World, &mut P)> = Vec::with_capacity(32);
            self.update_methods_any.insert(update_methods);
        }

        self.update_methods_any.get_mut::<Vec<fn(&mut World, &mut P)>>().unwrap().push(T::update_system);

        Registry::create_entry::<T>();
        self.physics.register_type::<T>();
    }

    pub fn add_actor<T: Actor<P> + 'static, P: 'static>(&mut self, actor: T) -> ID<T> {
        let typeid = TypeId::of::<T>();
        if !self.registry.types.contains(&typeid) {
            self.registry.types.insert(typeid);
            self.register_type::<T, P>();
        }

        let id = Registry::insert_actor(actor);
        let typed_id = id.into_typed_id();

        // generate default physics body for type
        let mut body = T::init_physicsbody(typed_id);
        // updates internal posision of physics shape based on the actor's position
        body.set_pos(&body.pos());


        self.physics.add_body(&id, body);

        id
    }

    pub fn remove_actor<T: Actor<P> + 'static, P: 'static>(&mut self, id: &ID<T>) {
        let id = *id;
        self.with_world(&id, move |_ett, world| {
            world.current_actor = Some(TypedID::from_id(id));
            // lifecycle: removal
            // ett.on_remove(id, world);

            // remove from events system

            // remove from actor registry
            let actor = Registry::remove_actor(&id);

            // remove from physics
            world.physics.delete_body(&id);

            if actor.is_some() {
                world.registry.recently_removed.insert(id.into_typed_id());
            }
        });
    }

    pub fn update_systems<P: 'static>(&mut self, ctx: &mut P) {
        let time = Instant::now();

        if let Some(systems) = self.update_methods_any.get_mut::<Vec<fn(&mut World, &mut P)>>() {
            for system in systems.clone() {
                system(self, ctx);
            }
        } else {
            println!("WARNING: no update methods registered for the generic type {:?}", std::any::type_name::<P>());
            panic!("Please make sure the argument passed into update_systems(), \"{}\",is the same as the generic type of the actor structs", std::any::type_name::<P>());
        }

        self.physics.cleanup();
        self.registry.recently_removed.clear();
        self.logic_update = time.elapsed();
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

    /**
    Queues an action to be executed on the given entity, with mutable access to the entity.
    */
    pub fn with<T: 'static>(&self, id: &ID<T>, f: impl Fn(&mut T) + 'static) {
        let id = *id;
        let closure = move |world: &mut World| {
            let entry = Registry::get_mut(&id);
            if let Some(entity) = entry {
                world.current_actor = Some(TypedID::from_id(id));
                f(&mut entity.1);
            } else {
                println!("with(entity) not found: {:?}", id.clone());
                println!("perhaps already in use?");
            }
        };

        self.event_bus.borrow_mut().push(Box::new(closure));
    }

    /**
    Queues an action to be executed on the given entity, with mutable access to the entity and the world.
    */ 
    pub fn with_world<T: 'static>(&self, id: &ID<T>, f: impl Fn(&mut T, &mut World) + 'static) {
        let id = *id;
        let closure = move |world: &mut World| {
            let entry = Registry::get_mut(&id);

            if let Some(entity) = entry {
                world.current_actor = Some(TypedID::from_id(id));
                f(&mut entity.1, world);
            } else {
                if world.registry.recently_removed.contains(&id.into_typed_id()) {
                    return; // skip error message
                }
                println!("with_world(entity) not found: {:?}", id.clone());
                println!("perhaps already in use?");
            }
        };

        self.event_bus.borrow_mut().push(Box::new(closure));
    }

    pub fn query<T: 'static>(&self) -> impl Iterator<Item = &(ID<T>,T)> + use<'_, T> {
        Registry::get_entry::<T>().arena.iter().map(|(_index, item)| item)
    }

    pub fn query_mut<T: 'static>(&mut self) -> impl Iterator<Item = &mut (ID<T>,T)> + use<'_, T> {
        Registry::get_entry_mut::<T>().arena.iter_mut().map(|(_index, item)| item)
    }

    pub fn get_singleton<T: 'static>(&self) -> Option<&T> {
        self.singletons.get::<T>()
    }

    pub fn set_singleton<T: 'static>(&mut self, value: T) -> Option<T> {
        self.singletons.insert(value)
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