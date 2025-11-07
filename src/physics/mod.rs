pub mod physics;
pub mod quadtree;

pub trait HasBounds {
    fn bounds(&self) -> crate::shapes::AABB;
}

