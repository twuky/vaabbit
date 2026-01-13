pub mod physics;
pub mod quadtree;
pub mod dynamictree;

pub trait HasBounds {
    fn bounds(&self) -> crate::shapes::AABB;
}

