use crate::world::World;
pub(crate) struct EventBus {
    pub(crate) events: Vec<Box<dyn Fn(&mut World)>>,
}

impl EventBus {
    pub(crate) const fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, event: Box<dyn Fn(&mut World)>) {
        self.events.push(event);
    }
}