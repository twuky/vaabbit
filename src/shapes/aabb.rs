use glam::*;
use super::*;

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2
}

impl AABB {

    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self { min: pos, max: pos + size }
    }

    pub fn from_aabbi32(aabb: super::AABBI32) -> Self {
        Self { min: aabb.min.as_vec2(), max: aabb.max.as_vec2() }
    }

    pub fn as_aabbi32(&self) -> super::AABBI32 {
        super::AABBI32::new(self.min, self.max)
    }

    #[inline(always)]
    pub fn overlaps_aabb(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x &&
        self.max.x >= other.min.x &&
        self.min.y <= other.max.y &&
        self.max.y >= other.min.y
    }

    pub fn pos(&self) -> Vec2 {
        self.min
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn area(&self) -> f32 {
        self.size().element_product()
    }

    pub fn is_within_aabb(&self, other: &AABB) -> bool {
        other.overlaps_point(self.min) &&
        other.overlaps_point(self.max)
    }

    pub fn center(&self) -> Vec2 {
        self.min + self.size() / 2.0
    }

    pub fn bottom_left(&self) -> Vec2 {
        self.min
    }

    pub fn bottom_right(&self) -> Vec2 {
        vec2(self.max.x, self.min.y)
    }

    pub fn top_left(&self) -> Vec2 {
        vec2(self.min.x, self.max.y)
    }

    pub fn top_right(&self) -> Vec2 {
        vec2(self.max.x, self.max.y)
    }
}

impl Shape for AABB {
    fn as_collision_shape(&self) -> CollisionShape {
        CollisionShape::AABB(*self)
    }

    fn centroid(&self) -> Vec2 {
        self.center()
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
            self.bottom_left(),
            self.bottom_right(),
            self.top_right(),
            self.top_left()
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
            self.bottom_left(),
            self.bottom_right(),
            self.top_right(),
            self.top_left()
        ])
    }
}