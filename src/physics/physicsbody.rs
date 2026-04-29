use glam::Vec2;

use crate::{TypedID, physics::HasBounds, shapes::{Collider, Shape}};

#[derive(Clone, Copy, PartialEq)]
pub enum PhysicsClass {
    Actor,
    Solid,
    Zone,
    Node,
}

#[derive(Clone, Copy)]
pub struct PhysicsBody {
    pos: Vec2,
    body: Option<Collider>,

    origin: Vec2,

    /** remaining position data after translation */
    pub(crate) pos_remainder: Vec2,

    pub id: TypedID,
    pub class: PhysicsClass,
    
}

impl PhysicsBody {
    pub fn new(pos: Vec2, body: Option<Collider>, id: TypedID, class: PhysicsClass) -> Self {
        Self { pos, origin: Vec2::ZERO, pos_remainder: Vec2::ZERO, body, id, class }
    }

    pub fn new_node(id: TypedID) -> Self {
        Self { pos: Vec2::ZERO, origin: Vec2::ZERO, pos_remainder: Vec2::ZERO, body: None, id, class: PhysicsClass::Node }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn origin(&self) -> Vec2 {
        self.origin
    }

    pub fn get_shape(&self) -> Option<&Collider>{
        self.body.as_ref()
    }    

    pub fn get_shape_mut(&mut self) -> Option<&mut Collider>{
        self.body.as_mut()
    }

    pub fn translate(&mut self, delta: &Vec2) {
        self.set_pos(&{self.pos + *delta});
    }

    pub fn set_origin(&mut self, origin: Vec2) {
        self.origin = origin;
        self.set_pos(&{self.pos});
    }

    /** 
    Sets the origin of the physics body to the center of its bounding box
    */
    pub fn set_origin_as_center(&mut self, center_x: bool, center_y: bool) -> Self {
        let bounds = self.bounds();
        let mut origin = bounds.center();
        if !center_x {
            origin.x = 0.0;
        }
        if !center_y {
            origin.y = 0.0;
        }
        self.set_origin(origin);
        *self
    }

    /**
    Sets the origin of the physics body to the given percent of objects bounding box

    For example, if the bounds are 100x100, and the origin is set to (0.5, 0.5),
    the body will be positioned at the center of the bounds (50, 50)
    */
    pub fn set_origin_as_percent(&mut self, percent: Vec2) {
        let bounds = self.bounds();
        let origin = Vec2::new(
            bounds.width() * percent.x,
            bounds.height() * percent.y
        );
        self.set_origin(origin);
    }

    pub fn set_pos(&mut self, pos: &Vec2) {
        if let Some(shape) = self.body.as_mut() {
            shape.set_pos(*pos - self.origin);
        }
        self.pos = *pos;    
    }

    pub fn bounds(&self) -> crate::shapes::AABB {
        if self.class == PhysicsClass::Node {
            return crate::shapes::AABB::ZERO;
        }
        match self.body {
            Some(shape) => {
                shape.bounds()
            },
            None => crate::shapes::AABB::ZERO,
        }
    }

    pub fn overlaps(&self, other: &PhysicsBody) -> bool {
        if other.class == PhysicsClass::Node {
            return false;
        }

        match self.class {
            PhysicsClass::Node => false,
            _ => {
                if let Some(body) = self.body {
                    if let Some(other_body) = other.body {
                        return body.overlaps(&other_body);
                    }
                }
                false
            }
        }
    }

    pub fn is_solid(&self) -> bool {
        self.class == PhysicsClass::Solid
    }
    pub fn is_actor(&self) -> bool {
        self.class == PhysicsClass::Actor
    }
    pub fn is_zone(&self) -> bool {
        self.class == PhysicsClass::Zone
    }
    pub fn is_node(&self) -> bool {
        self.class == PhysicsClass::Node
    }

}

impl HasBounds for PhysicsBody {
    fn bounds(&self) -> crate::shapes::AABB {
        self.bounds()
    }
}

impl PartialEq for PhysicsBody {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}