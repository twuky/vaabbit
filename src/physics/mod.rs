mod physics;
pub mod quadtree;
pub mod dynamictree;

pub(crate) use physics::{Physics, PhysicsBody, PhysicsData};
pub(crate) use quadtree::QuadTree;

pub trait HasBounds {
    fn bounds(&self) -> crate::shapes::AABB;
}

