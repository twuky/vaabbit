use glam::*;
use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub pos: Vec2,
    pub radius: f32
}

impl Circle {
    pub fn area(&self) -> f32 {
        std::f32::consts::PI * self.radius * self.radius
    }

    pub fn diameter(&self) -> f32 { self.radius * 2.0}
}

impl Shape for Circle {
    fn as_collision_shape(&self) -> CollisionShape {
        CollisionShape::CIRCLE(*self)
    }

    fn centroid(&self) -> Vec2 {
        self.pos
    }

    fn bounds(&self) -> AABB {
        AABB {
            pos: self.pos + vec2(-self.radius, -self.radius),
            size: vec2(self.radius * 2.0, self.radius * 2.0)
        }
    }

    fn overlaps_point(&self, point: Vec2) -> bool {
        self.pos.distance(point) < self.radius
    }

    fn overlaps_edge(&self, edge: Edge) -> bool {
        edge.overlaps_circle(self)
    }
    
    fn overlaps_polygon(&self, other: impl Shape) -> bool {
        super::solve::overlaps_poly_circle(&other, self)
    }
    
    fn overlaps_circle(&self, other: Circle) -> bool {
        self.pos.distance(other.pos) < self.radius + other.radius
    }

    fn edges(&self) -> Option<Vec<Edge>> {
        None
    }

    fn vertices(&self) -> Option<Vec<Vec2>> {
        None
    }
}
