use std::vec;
use glam::*;

mod aabb;
mod circle;
mod solve;

pub use aabb::AABB;
pub use circle::Circle;

pub enum CollisionShape {
    AABB(AABB),
    CIRCLE(Circle)
}

#[derive(Debug, Clone, Copy)]
pub struct Edge {
    a: Vec2,
    b: Vec2
}

impl Edge {
    pub fn perpendicular_dir(&self) -> Vec2 {
        Vec2::new(-(self.b.y - self.a.y), self.b.x - self.a.x).normalize()
    }

    pub fn overlaps_circle(&self, circle: &Circle) -> bool {
        solve::overlaps_edge_circle(self, circle)
    }

    pub fn overlaps_edge(&self, other: &Edge) -> bool {
        solve::overlaps_edge_edge(self, other)
    }
}

pub trait Shape {
    fn centroid(&self) -> Vec2;
    fn edges(&self) -> Option<Vec<Edge>>;
    fn vertices(&self) -> Option<Vec<Vec2>>;

    fn bounds(&self) -> AABB;
    
    fn as_collision_shape(&self) -> CollisionShape;

    fn overlaps_point(&self, point: Vec2) -> bool;
    fn overlaps_edge(&self, edge: Edge) -> bool;
    fn overlaps_polygon(&self, other: impl Shape) -> bool;
    fn overlaps_circle(&self, other: Circle) -> bool;

    fn point_within_bounds(&self, point: Vec2) -> bool {
        let bounds = self.bounds();

        (point.x > bounds.pos.x && point.x < bounds.pos.x + bounds.size.x) &&
        (point.y > bounds.pos.y && point.y < bounds.pos.y + bounds.size.y)
    }

    fn bounds_overlaps_bounds(&self, other: impl Shape) -> bool {
        self.bounds().overlaps_aabb(other.bounds())
    }

    fn overlaps<T: Shape>(&self, other: &T) -> bool {
        let other_shape = other.as_collision_shape();
        match other_shape {
            CollisionShape::AABB(o) => self.overlaps_polygon(o),
            CollisionShape::CIRCLE(o) => self.overlaps_circle(o),
        }
    }
}