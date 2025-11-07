use std::{collections::HashMap};

use anymap::AnyMap;
use glam::Vec2;
use pulz_arena::{Arena, Index, Mirror};
use smallvec::SmallVec;
use crate::{physics::{self, HasBounds, quadtree::QuadTree}, shapes::{AABB, AABBI32, CollisionShape}, world::ID};

pub(crate) struct PhysicsData {
    pub pos: Vec2,
    pub body: Option<CollisionShape>,
}

pub(crate) enum PhysicsBody {
    Actor(PhysicsData),
    Solid(PhysicsData),
    Zone(PhysicsData),
    Node,
}

pub struct PhyysicsEntry<T> {
    pub body_indices: pulz_arena::Mirror<Index>,
    pub _type: std::marker::PhantomData<T>,
}

impl HasBounds for PhysicsData {
    fn bounds(&self) -> crate::shapes::AABB {
        match &self.body {
            Some(shape) => {
                let mut aabb = shape.bounds();
                aabb.min += self.pos;
                aabb.max += self.pos;
                aabb
            },
            None => crate::shapes::AABB { min: self.pos, max: self.pos },
        }
    }
}

impl HasBounds for PhysicsBody {
    fn bounds(&self) -> crate::shapes::AABB {
        match self {
            PhysicsBody::Actor(data) => data.bounds(),
            PhysicsBody::Solid(data) => data.bounds(),
            PhysicsBody::Zone(data) => data.bounds(),
            PhysicsBody::Node => crate::shapes::AABB { min: Vec2::ZERO, max: Vec2::ZERO },
        }
    }
}

pub(crate) struct Physics {
    physics_bodies: Arena<PhysicsBody>,
    entities: AnyMap,

    tree: QuadTree<Index>,
    to_delete: Vec<Index>,

    query_bodiies: Vec<PhysicsBody>
}

impl Physics {
    pub fn new(size: AABB) -> Self {
        Self {
            physics_bodies: Arena::new(),
            entities: AnyMap::new(),
            tree: QuadTree::new(size.size().x, size.size().y, 8),
            to_delete: Vec::new(),
            query_bodiies: Vec::new(),
        }
    }

    pub(crate) fn register_type<T: 'static>(&mut self) {
        self.entities.insert::<PhyysicsEntry<T>>( PhyysicsEntry { body_indices: Mirror::new(), _type: std::marker::PhantomData });
    }

    fn idx_of<T: 'static>(&self, id: &ID<T>) -> Option<Index> {
        let entry = self.entities.get::<PhyysicsEntry<T>>().unwrap();
        entry.body_indices.get(id.index).cloned()
    }

    pub fn add_body<T: 'static>(&mut self, id: &ID<T>, body: PhysicsBody) {
        let bounds = body.bounds();
        let idx = self.physics_bodies.insert(body);

        self.tree.insert(idx, &bounds);

        let entry = self.entities.get_mut::<PhyysicsEntry<T>>().unwrap();
        entry.body_indices.insert(id.index, idx);
    }

    pub fn get_body<T: 'static>(&self, id: &ID<T>) -> Option<&PhysicsBody> {
        let idx = self.idx_of::<T>(id)?;
        self.physics_bodies.get(idx)
    }

    pub fn get_body_mut<T: 'static>(&mut self, id: &ID<T>) -> Option<&mut PhysicsBody> {
        let idx = self.idx_of::<T>(id)?;
        self.physics_bodies.get_mut(idx)
    }
    
    pub fn update_body<T: 'static>(&mut self, id: &ID<T>, body: PhysicsBody) {
        let bounds = body.bounds();
        let new_idx = self.physics_bodies.insert(body);

        let entry = self.entities.get_mut::<PhyysicsEntry<T>>().unwrap();
        let old_entry = entry.body_indices.insert(id.index, new_idx);

        self.tree.insert(new_idx, &bounds);

        if let Ok(Some(old_idx)) = old_entry {
            self.to_delete.push(old_idx);
            self.physics_bodies.remove(old_idx);
        }
    }

    pub fn delete_body<T: 'static>(&mut self, id: &ID<T>) {
        let entry = self.entities.get_mut::<PhyysicsEntry<T>>().unwrap();
        let idx = entry.body_indices.get(id.index).unwrap();

        self.physics_bodies.remove(*idx);
        self.to_delete.push(*idx);
    }

    pub fn cleanup(&mut self) {
        self.tree = QuadTree::new(2048.0, 2048.0, 12);

        for id in self.to_delete.iter() {
            self.physics_bodies.remove(*id);
        }
        for (id, body) in self.physics_bodies.iter() {

            let bounds = body.bounds();
            self.tree.insert_with_rebalance(id, &bounds);
        }
        
        self.to_delete.clear();
    }


    pub fn query<'a>(&'a self, bounds: &AABB, out: &mut SmallVec<[&'a PhysicsBody; 8]>) {
        let mut q = smallvec::SmallVec::new();
        self.tree.root.query(&bounds.as_aabbi32(), &mut q);

        for (idx, _aabb) in &q {
            // if self.to_delete.contains(idx) {
            //     continue;
            // }
            match self.physics_bodies.get(*idx) {
                Some(body) => {
                    out.push(body);
                },
                _ => {}
            }
        }
    }

    pub fn get_debug_info(&self) -> Vec<(usize, AABBI32)> {
        self.tree.get_debug_info()
    }
}