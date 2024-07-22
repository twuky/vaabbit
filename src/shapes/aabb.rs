use glam::*;
use super::*;

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub pos: Vec2,
    pub size: Vec2
}

impl AABB {
    pub fn overlaps_aabb(&self, other: &AABB) -> bool {
     ! (self.pos.x > other.pos.x + other.size.x ||
        self.pos.x + self.size.x < other.pos.x ||
        self.pos.y > other.pos.y + other.size.y ||
        self.pos.y + self.size.y < other.pos.y)
    }

    pub fn area(&self) -> f32 {
        self.size.element_product()
    }

    pub fn is_within_aabb(&self, other: &AABB) -> bool {
        other.overlaps_point(self.pos) &&
        other.overlaps_point(self.pos + self.size)
    }

    pub fn center(&self) -> Vec2 {
        self.pos + self.size / 2.0
    }

    pub fn bottom_left(&self) -> Vec2 {
        self.pos
    }

    pub fn bottom_right(&self) -> Vec2 {
        self.pos + vec2(self.size.x, 0.0)
    }

    pub fn top_left(&self) -> Vec2 {
        self.pos + vec2(0.0, self.size.y)
    }

    pub fn top_right(&self) -> Vec2 {
        self.pos + self.size
    }
}

impl Shape for AABB {
    fn as_collision_shape(&self) -> CollisionShape {
        CollisionShape::AABB(*self)
    }

    fn centroid(&self) -> Vec2 {
        vec2(self.pos.x + self.size.x / 2.0, self.pos.y + self.size.y / 2.0)
    }

    fn bounds(&self) -> AABB {
        *self
    }

    fn overlaps_point(&self, point: Vec2) -> bool {
        self.point_within_bounds(point)
    }

    fn overlaps_edge(&self, edge: Edge) -> bool {
        super::solve::overlaps_poly_edge(self, &edge)
    }
    
    fn overlaps_polygon(&self, other: impl Shape) -> bool {
        super::solve::overlaps_poly_poly(self, &other)
    }
    
    fn overlaps_circle(&self, other: Circle) -> bool {
        super::solve::overlaps_poly_circle(self, &other)
    }

    fn edges(&self) -> Option<Vec<Edge>> {
        let (a, b, c, d) = (
            self.pos,
            self.pos + vec2(self.size.x, 0.0),
            self.pos + self.size,
            self.pos + vec2(0.0, self.size.y),
        );

        Some(vec![
            Edge {a: a, b: b},
            Edge {a: b, b: c},
            Edge {a: c, b: d},
            Edge {a: d, b: a},
        ])
    }

    fn vertices(&self) -> Option<Vec<Vec2>> {
        Some(vec![
            self.pos,
            self.pos + vec2(self.size.x, 0.0),
            self.pos + self.size,
            self.pos + vec2(0.0, self.size.y),
        ])
    }
}