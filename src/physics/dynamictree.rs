use std::num::NonZeroUsize;
use glam::{vec2, Vec2};
use smallvec::{smallvec, SmallVec};
use crate::shapes::{self, AABB};

type Index = slotmap::DefaultKey;

// size = 48 (0x30), align = 0x4, no Drop
type SlotmapNode = Node<Index>;

pub struct Node<T> {
    pub bounds: AABB,
    pub parent: Option<Index>,

    pub child_1: Option<Index>,
    pub child_2: Option<Index>,

    pub data: Option<T>,
}

impl<T> Node<T> {
    pub fn new(bounds: AABB) -> Self {
        Self {
            bounds,
            parent: None,
            child_1: None,
            child_2: None,
            data: None,
        }
    }

    pub fn new_leaf(bounds: AABB, parent: Option<Index>, data: T) -> Self {
        Self {
            bounds,
            parent: parent,
            child_1: None,
            child_2: None,
            data: Some(data),
        }
    }

    fn is_leaf(&self) -> bool {
        self.child_1.is_none()
    }
}

pub struct DynamicTree<T> {
    pub root: Option<Index>,
    pub nodes: slotmap::SlotMap<slotmap::DefaultKey, Node<T>>,
}

impl<T: Clone + std::cmp::PartialEq> DynamicTree<T> {
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: slotmap::SlotMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.nodes.clear();
    }

    pub fn query(&self, bounds: &AABB) -> SmallVec<[(&T, &AABB); 32]> {
        let mut out = smallvec![];

        // if empty tree
        let Some(root) = self.root else {
            return out;
        };

        let mut stack = SmallVec::<[&Index; 64]>::new();
        stack.push(&root);

        while let Some(index) = stack.pop() {
            let node = &self.nodes.get(*index).unwrap();

            // skip if not overlapping
            if !bounds.overlaps_aabb(&node.bounds) {
                continue;
            }

            // if leaf
            if let Some(data) = &node.data {
                out.push((data, &node.bounds));
                continue;
            }

            // if internal
            if let Some(child) = &node.child_1 {
                stack.push(child);
            }
            if let Some(child) = &node.child_2 {
                stack.push(child);
            }
        }

        out
    }

    // returns true if the update would not require a rebalance of the tree
    pub fn try_update_body(&mut self, bounds: AABB, data: T) -> bool {
        // if empty tree
        let Some(root) = self.root else {
            return false;
        };

        let mut stack = SmallVec::<[&Index; 64]>::new();
        stack.push(&root);

        while let Some(index) = stack.pop() {
            let node = &self.nodes.get(*index).unwrap();

            if let Some(node_data) = &node.data {
                if data == *node_data {
                    // we store leaves as larger than the object, so updates
                    // that are smaller dont need to rebalance the tree

                    // exits early
                    return bounds.is_within_aabb(&node.bounds);
                } else { continue; }
            }

            // skip if not overlapping
            if !bounds.overlaps_aabb(&node.bounds) {
                continue;
            }

            // if internal
            if let Some(child) = &node.child_1 {
                stack.push(child);
            }
            if let Some(child) = &node.child_2 {
                stack.push(child);
            }
        }

        return false;
    }

    pub fn insert(&mut self,  data: T, bounds: &AABB,) {
        // we make the bounds slightly larger to allow
        // updating positions of objects without needing to
        // rebalance the tree
        let mut expanded_bounds = bounds.clone();
        expanded_bounds.expand(0.0);

        let leaf = Node::new_leaf(expanded_bounds, None, data);
        let leaf_index = self.nodes.insert(leaf);

        if self.root.is_none() {
            self.root = Some(leaf_index);
            return;
        }

        let best_sibling_index = self.find_best_sibling(&expanded_bounds);
        let sibling = self.nodes.get(best_sibling_index).unwrap();
        let old_parent = sibling.parent;

        let new_parent_bounds = sibling.bounds.union(&expanded_bounds);

        let mut new_parent = Node::<T>::new(new_parent_bounds);
        new_parent.parent = old_parent;
        new_parent.child_1 = Some(best_sibling_index);
        new_parent.child_2 = Some(leaf_index);

        let new_parent_index = self.nodes.insert(new_parent);

        self.nodes.get_mut(best_sibling_index).unwrap().parent = Some(new_parent_index);
        self.nodes.get_mut(leaf_index).unwrap().parent = Some(new_parent_index);

        match old_parent {
            Some(parent_index) => {
                let parent = self.nodes.get_mut(parent_index).unwrap();
                if parent.child_1 == Some(best_sibling_index) {
                    parent.child_1 = Some(new_parent_index);
                } else {
                    parent.child_2 = Some(new_parent_index);
                }
            }
            None => {
                self.root = Some(new_parent_index);
            }
        }

        self.fix_upwards(old_parent);
    }

    fn find_best_sibling(&self, leaf_bounds: &AABB) -> Index {
        let mut index = self.root.unwrap();

        loop {
            let search = self.nodes.get(index).unwrap();
            if search.is_leaf() {
                return index;
            }

            let child_1 = self.nodes.get(search.child_1.unwrap()).unwrap();
            let child_2 = self.nodes.get(search.child_2.unwrap()).unwrap();

            let cost_1 = child_1.bounds.union(leaf_bounds).perimeter();
            let cost_2 = child_2.bounds.union(leaf_bounds).perimeter();

            index = if cost_1 < cost_2 { 
                search.child_1.unwrap() 
            } else { 
                search.child_2.unwrap() 
            };
        }
    }

    fn fix_upwards(&mut self, mut index: Option<Index>) {
        while let Some(i) = index {
            let (c1, c2) = {
                let n = &self.nodes.get(i).unwrap();
                let c1 = self.nodes.get(n.child_1.unwrap()).unwrap();
                let c2 = self.nodes.get(n.child_2.unwrap()).unwrap();

                (c1, c2)
            };

            let new_bounds = c1.bounds.union(&c2.bounds);
            let n = self.nodes.get_mut(i).unwrap();

            n.bounds = new_bounds;
            index = n.parent;
        }
    }

    pub fn get_debug_info(&self) -> Vec<(usize, AABB)> {
        let mut out = vec![];

        // if empty tree
        let Some(root) = self.root else {
            return out;
        };

        let mut stack = SmallVec::<[&Index; 64]>::new();
        stack.push(&root);

        let mut depth = 0;
        while let Some(index) = stack.pop() {
            depth += 1;

            let node = &self.nodes.get(*index).unwrap();

            let mut is_leaf = 0;
            if node.is_leaf() {
                is_leaf = 1;
            }

            out.push((is_leaf, node.bounds));

            if let Some(c1) = &node.child_1 {
                stack.push(c1);
            }
            if let Some(c2) = &node.child_2 {
                stack.push(c2);
            }
        }

        return out;
    }
}