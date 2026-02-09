mod physics;
pub mod quadtree;
pub mod dynamictree;
mod physicsbody;

pub(crate) use physics::{Physics};
pub(crate) use quadtree::QuadTree;
pub use physicsbody::PhysicsBody;
pub use physicsbody::PhysicsClass;

pub(crate) static TREE_BOUNDS_PADDING: f32 = 4.0;

pub trait HasBounds {
    fn bounds(&self) -> crate::shapes::AABB;
}

