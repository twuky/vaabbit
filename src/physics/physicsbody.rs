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

    pub id: TypedID,
    pub class: PhysicsClass,
    
}

impl PhysicsBody {
    pub fn new(pos: Vec2, body: Option<Collider>, id: TypedID, class: PhysicsClass) -> Self {
        Self { pos, body, id, class }
    }

    pub fn new_node(id: TypedID) -> Self {
        Self { pos: Vec2::ZERO, body: None, id, class: PhysicsClass::Node }
    }


    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn get_shape(&self) -> Option<&Collider>{
        self.body.as_ref()
    }    

    pub fn get_shape_mut(&mut self) -> Option<&mut Collider>{
        self.body.as_mut()
    }

    pub fn translate(&mut self, delta: &Vec2) {
        if let Some(shape) = self.body.as_mut() {
            shape.translate(*delta);
            self.pos += *delta;
        }
    }

    pub fn set_pos(&mut self, pos: &Vec2) {
        if let Some(shape) = self.body.as_mut() {
            shape.set_pos(*pos);
            self.pos = *pos;    
        }
    }

    fn bounds(&self) -> crate::shapes::AABB {
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
            PhysicsClass::Node => return false,
            _ => {
                if let Some(body) = self.body {
                    if let Some(other_body) = other.body {
                        return body.overlaps(&other_body);
                    }
                }

                return false;
            }
        };
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