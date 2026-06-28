use glam::*;
use super::*;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2
}



impl AABB {
    pub const ZERO: AABB = AABB { min: Vec2::ZERO, max: Vec2::ZERO };

    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn from_size(size: Vec2) -> Self {
        Self { min: Vec2::ZERO, max: size }
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
        #[cfg(target_arch = "x86_64")] {
            use std::arch::x86_64::*;
            unsafe {
                let a = _mm_loadu_ps(self  as *const AABB as *const f32);
                let b = _mm_loadu_ps(other as *const AABB as *const f32);
                // lhs = [self.min.x, self.min.y, other.min.x, other.min.y]
                // rhs = [other.max.x, other.max.y, self.max.x, self.max.y]
                let lhs = _mm_movelh_ps(a, b);
                let rhs = _mm_movehl_ps(a, b);
                // branchless: self.min < other.max (lanes 0,1) AND other.min < self.max (lanes 2,3)
                _mm_movemask_ps(_mm_cmplt_ps(lhs, rhs)) == 0xF
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.min.x < other.max.x &&
            self.max.x > other.min.x &&
            self.min.y < other.max.y &&
            self.max.y > other.min.y
        }
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

    #[inline(always)]
    pub fn is_within_aabb(&self, other: &AABB) -> bool {
        #[cfg(target_arch = "x86_64")] {
            use std::arch::x86_64::*;
            unsafe {
                let a = _mm_loadu_ps(self  as *const AABB as *const f32);
                let b = _mm_loadu_ps(other as *const AABB as *const f32);
                // lhs = [other.min.x, other.min.y, self.max.x,  self.max.y]
                // rhs = [self.min.x,  self.min.y,  other.max.x, other.max.y]
                let lhs = _mm_shuffle_ps(b, a, 0b11100100);
                let rhs = _mm_shuffle_ps(a, b, 0b11100100);
                // branchless: other.min <= self.min AND self.max <= other.max
                _mm_movemask_ps(_mm_cmple_ps(lhs, rhs)) == 0xF
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.min.x >= other.min.x &&
            self.min.y >= other.min.y &&
            self.max.x <= other.max.x &&
            self.max.y <= other.max.y
        }
    }

    #[inline(always)]
    pub fn expand(&mut self, amount: f32) {
        self.min -= vec2(amount, amount);
        self.max += vec2(amount, amount);
    }

    #[inline(always)]
    pub fn union(self, other: AABB) -> Self {
        #[cfg(target_arch = "x86_64")] {
            use std::arch::x86_64::*;
            unsafe {
                // #[repr(C)] AABB is exactly [min.x, min.y, max.x, max.y], so load
                // each as one unaligned 128-bit vector (no scalar gather).
                let a = _mm_loadu_ps(&self  as *const AABB as *const f32);
                let b = _mm_loadu_ps(&other as *const AABB as *const f32);
                let lo = _mm_min_ps(a, b);  // correct for lanes 0,1 (min components)
                let hi = _mm_max_ps(a, b);  // correct for lanes 2,3 (max components)
                // want [lo[0], lo[1], hi[2], hi[3]]; the compiler folds this to a movsd merge
                let r = _mm_shuffle_ps(lo, hi, 0b11100100);
                let mut out = std::mem::MaybeUninit::<AABB>::uninit();
                _mm_storeu_ps(out.as_mut_ptr() as *mut f32, r);
                out.assume_init()
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    #[inline(always)]
    pub fn perimeter(&self) -> f32 {
        let w = self.width();
        let h = self.height();
        w + w + h + h
    }

    /// useful for greedily inserting AABBs into the dynamic tree
    #[inline(always)]
    pub fn perimeter_heightweighted(&self) -> f32 {
        let w = self.width();
        let h = self.height();
        w + h + h
    }

    #[inline(always)]
    pub fn inseam(&self) -> f32 {
        self.width() + self.height()
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

    #[inline(always)]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline(always)]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}

impl PartialEq for AABB {
    fn eq(&self, other: &Self) -> bool {
        self.min == other.min && self.max == other.max
    }
}

impl Shape for AABB {
    fn as_collision_shape(&self) -> Collider {
        Collider::AABB(*self)
    }

    fn centroid(&self) -> Vec2 {
        self.center()
    }

    fn bounds(&self) -> AABB {
        *self
    }

    fn translate(&mut self, offset: Vec2) {
        self.min += offset;
        self.max += offset;
    }

    fn set_pos(&mut self, pos: Vec2) {
        let w_h = vec2(self.width(), self.height());
        self.min = pos;
        self.max = pos + w_h;
    }

    #[inline(always)]
    fn overlaps_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x && 
        point.x <= self.max.x && 
        point.y >= self.min.y && 
        point.y <= self.max.y
    }

    fn overlaps_edge(&self, edge: Edge) -> bool {
        super::solve::overlaps_poly_edge(self, &edge)
    }
    
    fn overlaps_polygon(&self, other: &impl Shape) -> bool {
        super::solve::overlaps_poly_poly(self, other)
    }
    
    fn overlaps_circle(&self, other: &Circle) -> bool {
        let closest = other.pos.clamp(self.min, self.max);
        closest.distance_squared(other.pos) < other.radius * other.radius
    }

    fn edges(&self) -> Option<Vec<Edge>> {
        let (a, b, c, d) = (
            self.bottom_left(),
            self.bottom_right(),
            self.top_right(),
            self.top_left()
        );

        Some(vec![
            Edge {a, b},
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