use glam::Vec2;
use crate::shapes::{AABB, Circle, Edge, Shape};

#[derive(Debug, Clone, Copy)]
pub enum Collider {
    AABB(AABB),
    CIRCLE(Circle)
}

impl Collider {
    pub fn bounds(&self) -> AABB {
        match self {
            Collider::AABB(shape) => shape.bounds(),
            Collider::CIRCLE(shape) => shape.bounds(),
        }
    }

    pub fn overlaps(&self, other: &Collider) -> bool {
        match (self, other) {
            (Collider::AABB(a), Collider::AABB(b)) => a.overlaps_aabb(b),
            (Collider::CIRCLE(a), Collider::AABB(b)) => a.overlaps_aabb(b),
            (Collider::CIRCLE(a), Collider::CIRCLE(b)) => a.overlaps_circle(b),
            (Collider::AABB(a), Collider::CIRCLE(b)) => a.overlaps_circle(b),
            //_ => false
        }
    }

    pub fn aabb(pos: Vec2, size: Vec2) -> Option<Self> {
        Some(Collider::AABB(AABB::from_pos_size(pos, size)))
    }

    pub fn circle(pos: Vec2, radius: f32) -> Option<Self> {
        Some(Collider::CIRCLE(Circle::new(pos, radius)))
    }
}

impl Shape for Collider {
    fn centroid(&self) -> Vec2 {
        match self {
            Collider::AABB(a) => a.centroid(),
            Collider::CIRCLE(c) => c.centroid(),
        }
    }

    fn edges(&self) -> Option<Vec<Edge>> {
        match self {
            Collider::AABB(a) => a.edges(),
            Collider::CIRCLE(c) => c.edges(),
        }
    }

    fn vertices(&self) -> Option<Vec<Vec2>> {
        match self {
            Collider::AABB(a) => a.vertices(),
            Collider::CIRCLE(c) => c.vertices(),
        }
    }

    fn translate(&mut self, offset: Vec2) {
        match self {
            Collider::AABB(a) => a.translate(offset),
            Collider::CIRCLE(c) => c.translate(offset),
        }
    }

    fn set_pos(&mut self, pos: Vec2) {
        match self {
            Collider::AABB(a) => a.set_pos(pos),
            Collider::CIRCLE(c) => c.set_pos(pos),
        }
    }

    fn bounds(&self) -> AABB {
        match self {
            Collider::AABB(a) => a.bounds(),
            Collider::CIRCLE(c) => c.bounds(),
        }
    }

    fn as_collision_shape(&self) -> Collider {
        *self
    }

    fn overlaps_point(&self, point: Vec2) -> bool {
        match self {
            Collider::AABB(a) => a.overlaps_point(point),
            Collider::CIRCLE(c) => c.overlaps_point(point),
        }
    }

    fn overlaps_edge(&self, edge: Edge) -> bool {
        match self {
            Collider::AABB(a) => a.overlaps_edge(edge),
            Collider::CIRCLE(c) => c.overlaps_edge(edge),
        }
    }

    fn overlaps_polygon(&self, other: &impl Shape) -> bool {
        match self {
            Collider::AABB(a) => a.overlaps_polygon(other),
            Collider::CIRCLE(c) => c.overlaps_polygon(other),
        }
    }

    fn overlaps_circle(&self, other: &Circle) -> bool {
        match self {
            Collider::AABB(a) => a.overlaps_circle(other),
            Collider::CIRCLE(c) => c.overlaps_circle(other),
        }
    }
}