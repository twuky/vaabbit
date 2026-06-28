use rapidhash::RapidHashMap;

use anymap::AnyMap;
use glam::Vec2;
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use smallvec::SmallVec;
use crate::{ID, TypedID, physics::{dynamictree::DynamicTree}, shapes::{AABB}};
use crate::physics::physicsbody::PhysicsBody;

pub struct PhyysicsEntry<T> {
    // source of truth for each body's index in the physics_bodies slotmap
    pub body_indices: SecondaryMap<slotmap::DefaultKey, slotmap::DefaultKey>,
    // each body maintains a list of current overlaps
    pub overlap_list: SecondaryMap<slotmap::DefaultKey, Vec<TypedID>>,

    pub _type: std::marker::PhantomData<T>,
}

pub(crate) struct Physics {
    physics_bodies: SlotMap<slotmap::DefaultKey, PhysicsBody>,
    entities: AnyMap,

    //tree: QuadTree<slotmap::DefaultKey>,
    tree: DynamicTree<slotmap::DefaultKey>,
    to_delete: SecondaryMap<slotmap::DefaultKey, ()>,

    // late collision detection. consumed by an object when it updates for events created by other object movement
    pub late_collision_enter: RapidRapidHashMap<TypedID, SmallVec<[TypedID; 8]>>,
    // late collision detection. consumed by an object when it updates for events created by other object movement
    pub late_collision_exit: RapidRapidHashMap<TypedID, SmallVec<[TypedID; 8]>>,

    tree_bounds: AABB,
}

impl Physics {
    pub fn new(size: AABB) -> Self {
        Self {
            physics_bodies: SlotMap::new(),
            entities: AnyMap::new(),
            //tree: QuadTree::new(size.width(), size.height(), 12),
            tree: DynamicTree::new(),
            to_delete: SecondaryMap::new(),

            late_collision_enter: RapidHashMap::default(),
            late_collision_exit: RapidHashMap::default(),

            tree_bounds: size,
        }
    }

    pub fn get_overlap_list<T: 'static>(&self, id: &ID<T>) -> &Vec<TypedID> {
        let entry = self.entities.get::<PhyysicsEntry<T>>().unwrap();
        let overlap_list = entry.overlap_list.get(id.index).unwrap();
        overlap_list
    }

    pub fn update_overlap_list<T: 'static>(&mut self, id: &ID<T>, overlap_list: &[TypedID], exit_list: &[TypedID]) {
        let entry = self.entities.get_mut::<PhyysicsEntry<T>>().unwrap();
        // only add items that are not already in the list
        let list = entry.overlap_list.get_mut(id.index).unwrap();
        
        for item in overlap_list {
            if !list.contains(item) {
                list.push(*item);
            }
        }

        list.retain(|existing| !exit_list.contains(existing));
    }

    pub(crate) fn get_late_collision_enter<T: 'static>(&mut self, id: &ID<T>) -> Option<SmallVec<[TypedID; 8]>> {
        // consume the list to return it
        self.late_collision_enter.remove(&id.into_typed_id())
    }
    pub(crate) fn get_late_collision_exit<T: 'static>(&mut self, id: &ID<T>) -> Option<SmallVec<[TypedID; 8]>> {
        // consume the list to return it
        self.late_collision_exit.remove(&id.into_typed_id())
    }

    pub(crate) fn add_late_collision_enter(&mut self, id: TypedID, other: TypedID) {
        let list = self.late_collision_enter.entry(id).or_default();
        list.push(other);
    }
    pub(crate) fn add_late_collision_exit(&mut self, id: TypedID, other: TypedID) {
        let list = self.late_collision_exit.entry(id).or_default();
        list.push(other);
    }

    pub(crate) fn register_type<T: 'static>(&mut self) {
        self.entities.insert::<PhyysicsEntry<T>>( PhyysicsEntry { 
            body_indices: SecondaryMap::new(),
            overlap_list: SecondaryMap::new(),
            _type: std::marker::PhantomData 
        });
    }

    #[inline(always)]
    pub fn idx_of<T: 'static>(&self, id: &ID<T>) -> Option<DefaultKey> {
        let entry = self.entities.get::<PhyysicsEntry<T>>().unwrap();
        entry.body_indices.get(id.index).cloned()
    }

    pub fn add_body<T: 'static>(&mut self, id: &ID<T>, body: PhysicsBody) {
        let mut bounds = body.bounds();
        let idx = self.physics_bodies.insert(body);

        // expand the bounds a bit
        bounds.expand(crate::physics::TREE_BOUNDS_PADDING);
        
        let entry = self.entities.get_mut::<PhyysicsEntry<T>>().unwrap();
        let _ = entry.body_indices.insert(id.index, idx);
        let _ = entry.overlap_list.insert(id.index, Vec::new());

        self.tree.insert(idx, &bounds);
    }

    #[inline(always)]
    pub fn get_body<T: 'static>(&self, id: &ID<T>) -> Option<&PhysicsBody> {
        let entry = self.entities.get::<PhyysicsEntry<T>>()?;
        let idx = entry.body_indices.get(id.index)?;
        self.physics_bodies.get(*idx)
    }

    #[inline(always)]
    pub fn get_body_pos<T: 'static>(&self, id: &ID<T>) -> Option<Vec2> {
        let entry = self.entities.get::<PhyysicsEntry<T>>()?;
        let idx = entry.body_indices.get(id.index)?;
        Some(self.physics_bodies.get(*idx)?.pos())
    }

    #[inline(always)]
    pub fn get_body_mut<T: 'static>(&mut self, id: &ID<T>) -> Option<&mut PhysicsBody> {
        let entry = self.entities.get::<PhyysicsEntry<T>>()?;
        let idx = entry.body_indices.get(id.index)?;
        self.physics_bodies.get_mut(*idx)
    }

    #[inline(always)]
    pub fn update_body_in_place<T: 'static>(&mut self, id: &ID<T>, body: PhysicsBody) {
        let entry = self.entities.get_mut::<PhyysicsEntry<T>>().unwrap();
        let idx = entry.body_indices.get(id.index).unwrap();
        *self.physics_bodies.get_mut(*idx).unwrap() = body; 
    }
    
    #[inline(always)]
    pub fn update_body<T: 'static>(&mut self, id: &ID<T>, body: PhysicsBody) {
        let mut bounds = body.bounds();
        let entry = self.entities.get_mut::<PhyysicsEntry<T>>().unwrap();

        let new_idx = self.physics_bodies.insert(body);
        
        let old_entry = entry.body_indices.insert(id.index, new_idx);

        bounds.expand(crate::physics::TREE_BOUNDS_PADDING);
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

        //self.tree = QuadTree::new(self.tree_bounds.width(), self.tree_bounds.height(), 12); // quadtree
        self.tree.clear(); // dynamictree
        
        //we'll try to calculate the smallest bounds of all the bodies
        let mut min_bounds = AABB { min: Vec2::ZERO, max: Vec2::ZERO };

        for (id, _d) in self.to_delete.iter() {
            self.physics_bodies.remove(id);
        }
        for (id, body) in self.physics_bodies.iter() {
            let bounds = body.bounds();
            min_bounds.min = min_bounds.min.min(bounds.min);
            min_bounds.max = min_bounds.max.max(bounds.max);
            //self.tree.insert_with_rebalance(id, &bounds);
            self.tree.insert(id, &bounds);
        }
        self.to_delete.clear();
        min_bounds.min -= 32.0;
        min_bounds.max += 32.0;
        self.tree_bounds = min_bounds;
    }

    pub fn query<'a>(&'a self, bounds: &AABB, out: &mut SmallVec<[&'a PhysicsBody; 4]>) {
        let q = self.tree.query(bounds);

        for (idx, _aabb) in q {
            if self.to_delete.contains_key(*idx) {continue}
            
            if let Some(body) = self.physics_bodies.get(*idx) {
                out.push(body);
            }
        }
    }

    pub fn query_filtered<'a>(&'a self, bounds: &AABB, out: &mut SmallVec<[&'a PhysicsBody; 4]>, filter: impl Fn(&PhysicsBody) -> bool) {
        let q = self.tree.query(bounds);

        for (idx, _aabb) in q {
            if self.to_delete.contains_key(*idx) {continue}
            
            if let Some(body) = self.physics_bodies.get(*idx) {
               if filter(body) {
                   out.push(body);
               }
            }
        }
    }

    pub(crate) fn query_against_id<'a>(&'a self, bounds: &AABB, out: &mut SmallVec<[&'a PhysicsBody; 4]>, id: TypedID) {
        let q = self.tree.query(bounds);

        for (idx, _aabb) in q {
            if self.to_delete.contains_key(*idx) {continue}
            
            if let Some(body) = self.physics_bodies.get(*idx) {
               if body.id != id {
                   out.push(body);
               }
            }
        }
    }

    pub fn get_debug_info(&self) -> Vec<(usize, AABB)> {
        self.tree.get_debug_info()
    }
}