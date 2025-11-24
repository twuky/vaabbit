pub mod shapes;
pub mod world;
pub mod physics;
pub mod events;
pub mod entity;

pub use glam::*;
pub use entity::{ID, TypedID, Actor};
pub use world::World;
pub use events::Signal;

pub fn type_of<T: 'static>() -> std::any::TypeId {
    std::any::TypeId::of::<T>()
}