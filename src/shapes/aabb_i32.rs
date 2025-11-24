use glam::IVec2;
use glam::Vec2;
use glam::ivec2;

#[derive(Debug, Clone, Copy)]
pub struct AABBI32 {
    pub min: IVec2,
    pub max: IVec2
}

impl AABBI32 {

    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min: min.as_ivec2(), max: max.as_ivec2() }
    }

    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self { min: pos.as_ivec2(), max: (pos + size).as_ivec2() }
    }

    pub fn from_aabb(aabb: super::AABB) -> Self {
        Self { min: aabb.min.as_ivec2(), max: aabb.max.as_ivec2() }
    }

    pub fn as_aabb(&self) -> super::AABB {
        super::AABB { min: self.min.as_vec2(), max: self.max.as_vec2() }
    }

    #[inline(always)]
    pub fn overlaps_aabb(&self, other: &AABBI32) -> bool {
        self.max.x >= other.min.x &&
        self.min.x <= other.max.x &&
        self.min.y <= other.max.y &&
        self.max.y >= other.min.y
    }

    pub fn pos(&self) -> IVec2 {
        self.min
    }

    pub fn size(&self) -> IVec2 {
        self.max - self.min
    }

    pub fn area(&self) -> f32 {
        self.size().element_product() as f32
    }

    pub fn is_within_aabb(&self, other: &AABBI32) -> bool {
        other.overlaps_point(self.min) &&
        other.overlaps_point(self.max)
    }

    pub fn overlaps_point(&self, point: IVec2) -> bool {
        return point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y;
    }

    pub fn center(&self) -> IVec2 {
        (self.min + self.size()) / 2
    }

    pub fn bottom_left(&self) -> IVec2 {
        self.min
    }

    pub fn bottom_right(&self) -> IVec2 {
        ivec2(self.max.x, self.min.y)
    }

    pub fn top_left(&self) -> IVec2 {
        ivec2(self.min.x, self.max.y)
    }

    pub fn top_right(&self) -> IVec2 {
        ivec2(self.max.x, self.max.y)
    }
}