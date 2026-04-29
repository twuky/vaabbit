pub mod shapes;
pub mod world;
pub mod physics;
pub mod events;
pub mod entity;
pub mod math;

pub use glam::*;
pub use entity::{ID, TypedID, Actor};
pub use world::World;
pub use events::Signal;

pub const fn type_of<P: 'static, T: Actor<P> + 'static>() -> std::any::TypeId {
    std::any::TypeId::of::<T>()
}

pub fn init() -> World {
    World::new()
}