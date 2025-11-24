use std::collections::HashMap;
use crate::{World, ID};

pub type Signal<T> = T;

pub trait EventEmitter<T> {
    fn emit<I: 'static>(world: &mut crate::world::World, id: &ID<I>, event: T);
}

impl<T: 'static> EventEmitter<T> for Signal<T> {
    fn emit<I: 'static>(world: &mut crate::world::World, id: &ID<I>, event: T) {
        world.emit(id.clone(), event);
    }
}

pub struct EventQueue {
    pub(crate) events: anymap::AnyMap,
}
pub(crate) struct EventListeners<E> {
    pub(crate) listeners: Vec<Box<dyn Fn(&mut World,&E)>>,
}

impl<E> EventListeners<E> {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }
}

struct EventQueueEntry<T, E> {
    pub(crate) emitters: HashMap<ID<T>, EventListeners<E>>,
}

impl<T, E> EventQueueEntry<T, E> {
    pub fn new() -> Self {
        Self {
            emitters: HashMap::new(),
        }
    }
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            events: anymap::AnyMap::new(),
        }
    }

    fn _get_entry<T: 'static, E: 'static>(&self) -> Option<&EventQueueEntry<T, E>> {
        self.events.get::<EventQueueEntry<T, E>>()
    }

    fn get_entry_mut<T: 'static, E: 'static>(&mut self) -> &mut EventQueueEntry<T, E> {
        if self.events.contains::<EventQueueEntry<T, E>>() {
            self.events.get_mut::<EventQueueEntry<T, E>>().unwrap()
        } else {
            let e = EventQueueEntry::<T, E>::new();
            self.events.insert(e);
            self.events.get_mut::<EventQueueEntry<T, E>>().unwrap()
        }
    }

    pub fn subscribe<E: 'static, T: 'static, L: 'static>
    (&mut self, emitter: ID<T>, _listener: ID<L>, closure: impl Fn(&mut World, &E) + 'static) {
        let entry = self.get_entry_mut::<T, E>();

        if !entry.emitters.contains_key(&emitter) {
            entry.emitters.insert(emitter, EventListeners::<E>::new());
        }
        let listeners = entry.emitters.get_mut(&emitter).unwrap();

        listeners.listeners.push(Box::new(closure));
    }

    pub(crate) fn _emit<E: 'static, T: 'static>(&mut self, world: &mut World, emitter: ID<T>, event: E) {
        let entry = self.get_entry_mut::<T, E>();
        if let Some(listeners) = entry.emitters.get_mut(&emitter) {
            for listener in listeners.listeners.iter_mut() {
                listener(world, &event);
            }
        }
    }

    pub(crate) fn get_listeners<E: 'static, T: 'static>(&mut self, emitter: ID<T>) -> Option<&mut EventListeners<E>> {
        let entry = self.get_entry_mut::<T, E>();
        entry.emitters.get_mut(&emitter)
    }

}