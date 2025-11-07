use std::{collections::HashMap, vec};
use crate::world::ID;



pub type Event<T> = T;

pub trait EventEmitter<T> {
    fn emit<I: 'static>(world: &mut crate::world::World, id: &ID<I>, event: T);
}

impl<T: 'static> EventEmitter<T> for Event<T> {
    fn emit<I: 'static>(world: &mut crate::world::World, id: &ID<I>, event: T) {
        world.emit(id.clone(), event);
    }
}

pub struct EventQueue {
    pub(crate) events: anymap::AnyMap,

}
pub(crate) struct EventListeners<E> {
    _phantom: std::marker::PhantomData<E>,

    pub(crate) listeners: Vec<Box<dyn Fn(&E)>>,
}

impl<E> EventListeners<E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
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

    pub(crate) fn get_entry_mut<T: 'static, E: 'static>(&mut self) -> &mut EventQueueEntry<T, E> {
        if self.events.contains::<EventQueueEntry<T, E>>() {
            self.events.get_mut::<EventQueueEntry<T, E>>().unwrap()
        } else {
            let e = EventQueueEntry::<T, E>::new();
            self.events.insert(e);
            self.events.get_mut::<EventQueueEntry<T, E>>().unwrap()
        }
    }

    pub fn subscribe<E: 'static, T: 'static, L: 'static>
    (&mut self, emitter: ID<T>, listener: ID<L>, closure: impl Fn(&E) + 'static) {
        let entry = self.get_entry_mut::<T, E>();

        if !entry.emitters.contains_key(&emitter) {
            entry.emitters.insert(emitter, EventListeners::<E>::new());
        }
        let listeners = entry.emitters.get_mut(&emitter).unwrap();

        listeners.listeners.push(Box::new(closure));
    }

    pub(crate) fn emit<E: 'static, T: 'static>(&mut self, emitter: ID<T>, event: E) {
        let entry = self.get_entry_mut::<T, E>();
        if let Some(listeners) = entry.emitters.get_mut(&emitter) {
            for listener in listeners.listeners.iter_mut() {
                listener(&event);
            }
        }
    }

    pub(crate) fn get_listeners<E: 'static, T: 'static>(&mut self, emitter: ID<T>) -> Option<&mut EventListeners<E>> {
        let entry = self.get_entry_mut::<T, E>();
        entry.emitters.get_mut(&emitter)
    }

}