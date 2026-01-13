
use anymap::AnyMap;
use glam::Vec2;
use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};
use smallvec::SmallVec;
use crate::{ID, TypedID, physics::{HasBounds, dynamictree::DynamicTree, quadtree::QuadTree}, shapes::{AABB, CollisionShape}};

pub(crate) struct PhysicsData {
    pub pos: Vec2,
    pub body: Option<CollisionShape>,

    pub id: TypedID,
}

pub(crate) enum PhysicsBody {
    Actor(PhysicsData),
    Solid(PhysicsData),
    Zone(PhysicsData),
    Node,
}

impl PhysicsBody {
    pub fn get_shape(&self) -> Option<&CollisionShape>{
        match self {
            PhysicsBody::Actor(data) => data.body.as_ref(),
            PhysicsBody::Solid(data) => data.body.as_ref(),
            PhysicsBody::Zone(data) => data.body.as_ref(),
            PhysicsBody::Node => None
        }
    }

    pub fn overlaps(&self, other: &PhysicsBody) -> bool {
        let s = self.get_shape();
        let o = other.get_shape();

        if s.is_none() || o.is_none() {
            return false;
        }
        s.unwrap().overlaps(o.unwrap())
    }
}
pub struct PhyysicsEntry<T> {
    pub body_indices: SparseSecondaryMap<slotmap::DefaultKey, slotmap::DefaultKey>,
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
    physics_bodies: SlotMap<slotmap::DefaultKey, PhysicsBody>,
    entities: AnyMap,

    tree: DynamicTree<slotmap::DefaultKey>,
    to_delete: SparseSecondaryMap<slotmap::DefaultKey, ()>,

    tree_bounds: AABB,
}

impl Physics {
    pub fn new(size: AABB) -> Self {
        Self {
            physics_bodies: SlotMap::new(),
            entities: AnyMap::new(),
            tree: DynamicTree::new(),
            to_delete: SparseSecondaryMap::new(),
            tree_bounds: size,
        }
    }

    pub(crate) fn register_type<T: 'static>(&mut self) {
        self.entities.insert::<PhyysicsEntry<T>>( PhyysicsEntry { body_indices: SparseSecondaryMap::new(), _type: std::marker::PhantomData });
    }

    fn idx_of<T: 'static>(&self, id: &ID<T>) -> Option<DefaultKey> {
        let entry = self.entities.get::<PhyysicsEntry<T>>().unwrap();
        entry.body_indices.get(id.index).cloned()
    }

    pub fn add_body<T: 'static>(&mut self, id: &ID<T>, body: PhysicsBody) {
        let bounds = body.bounds();
        let idx = self.physics_bodies.insert(body);

        self.tree.insert(idx, &bounds);

        let entry = self.entities.get_mut::<PhyysicsEntry<T>>().unwrap();
        let _ = entry.body_indices.insert(id.index, idx);
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

        if let Some(old_idx) = old_entry {
            self.to_delete.insert(old_idx, ());
            self.physics_bodies.remove(old_idx);
        }
    }

    pub fn delete_body<T: 'static>(&mut self, id: &ID<T>) {
        let entry = self.entities.get_mut::<PhyysicsEntry<T>>().unwrap();
        let idx = entry.body_indices.get(id.index).unwrap();

        self.physics_bodies.remove(*idx);
        self.to_delete.insert(*idx, ());
    }

    pub fn cleanup(&mut self) {
        //self.tree = QuadTree::new(self.tree_bounds.width(), self.tree_bounds.height(), 16);

        self.tree.clear();
        //we'll try to calculate the smallest bounds of all the bodies
        let mut min_bounds = AABB { min: Vec2::ZERO, max: Vec2::ZERO };

        for (id, _d) in self.to_delete.iter() {
            self.physics_bodies.remove(id);
        }
        for (id, body) in self.physics_bodies.iter() {
            let bounds = body.bounds();

            if bounds.min.x < min_bounds.min.x {
                min_bounds.min.x = bounds.min.x;
            }
            if bounds.min.y < min_bounds.min.y {
                min_bounds.min.y = bounds.min.y;
            }
            if bounds.max.x > min_bounds.max.x {
                min_bounds.max.x = bounds.max.x;
            }
            if bounds.max.y > min_bounds.max.y {
                min_bounds.max.y = bounds.max.y;
            }

            //self.tree.insert_with_rebalance(id, &bounds);
            self.tree.insert(id, &bounds);
        }
        
        self.to_delete.clear();
        min_bounds.min.x -= 32.0;
        min_bounds.min.y -= 32.0;
        min_bounds.max.x += 32.0;
        min_bounds.max.y += 32.0;
        self.tree_bounds = min_bounds;
    }


    pub fn query<'a>(&'a self, bounds: &AABB, out: &mut SmallVec<[&'a PhysicsBody; 32]>) {
        let q = self.tree.query(bounds);

        let mut passed = 0;
        for (idx, aabb) in q {
            if self.to_delete.contains_key(*idx) {continue}
            passed += 1;
            if !bounds.overlaps_aabb(aabb) {continue}
            

            match self.physics_bodies.get(*idx) {
                Some(body) => {
                    out.push(body);
                },
                _ => {}
            }
        }
        // debug
        // println!("passed: {}, total: {}", passed, q.len());
    }

    pub fn get_debug_info(&self) -> Vec<(usize, AABB)> {
        self.tree.get_debug_info()
    }
}