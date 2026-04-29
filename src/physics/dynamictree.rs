use std::cell::UnsafeCell;
use glam::Vec2;
use smallvec::{smallvec, SmallVec};
use crate::shapes::AABB;

pub struct Node<T> {
    pub bounds: AABB,
    pub parent: Option<usize>,

    pub child_1: Option<usize>,
    pub child_2: Option<usize>,

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

    pub fn new_leaf(bounds: AABB, parent: Option<usize>, data: T) -> Self {
        Self {
            bounds,
            parent,
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
    pub root: usize,
    pub nodes: Vec<Node<T>>,

    query_stack: UnsafeCell<Vec<usize>>,
}

impl<T: Clone + std::cmp::PartialEq> Default for DynamicTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + std::cmp::PartialEq> DynamicTree<T> {
    pub fn new() -> Self {
        let leaf = Node::<T>::new(AABB {min: Vec2::ZERO, max: Vec2::ZERO});
        let mut nodes: Vec<Node<T>> = Vec::with_capacity(2048);
        nodes.push(leaf);
        Self {
            root: 0,
            nodes,
            query_stack: UnsafeCell::new(Vec::with_capacity(512)),
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        let leaf = Node::<T>::new(AABB {min: Vec2::ZERO, max: Vec2::ZERO});
        self.nodes.push(leaf);
        self.root = 0;
    }

    pub fn query(&self, bounds: &AABB) -> SmallVec<[(&T, &AABB); 16]> {
        let mut out = smallvec![];

        if self.nodes.len() <= 1 {return out}

        // safety: query func is not recursive,
        // and is the only function that can modify the scratch stack
        let stack = unsafe { &mut *self.query_stack.get() };
        stack.clear();
        stack.push(self.root);

        let mut cursor = 0;

        while cursor < stack.len() {
            let index = stack[cursor];
            cursor += 1;

            let node = unsafe {
                self.nodes.get_unchecked(index)
            };

            if !bounds.overlaps_aabb(&node.bounds) {
                continue;
            }

            if let Some(data) = &node.data {
                out.push((data, &node.bounds));
                continue;
            }

            if let Some(c1) = node.child_1 {
                stack.push(c1);
            }
            if let Some(c2) = node.child_2 {
                stack.push(c2);
            }
        }

        out
    }

    // returns true if the update would not require a rebalance of the tree
    pub fn try_update_body(&mut self, bounds: AABB, data: T) -> bool {

        let mut stack = SmallVec::<[usize; 64]>::new();
        stack.push(self.root);

        while let Some(index) = stack.pop() {
            let node = &self.nodes.get(index).unwrap();

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
            if let Some(child) = node.child_1 {
                stack.push(child);
            }
            if let Some(child) = node.child_2 {
                stack.push(child);
            }
        }

        false
    }

    pub fn insert(&mut self,  data: T, bounds: &AABB,) {
        unsafe  {
            let leaf = Node::new_leaf(*bounds, None, data);
            self.nodes.push(leaf);
            let leaf_index = self.nodes.len() - 1;

            let best_sibling_index = self.find_best_sibling(bounds);
            let sibling = self.nodes.get_unchecked(best_sibling_index);
            let old_parent = sibling.parent;

            let new_parent_bounds = sibling.bounds.union(*bounds);

            let mut new_parent = Node::<T>::new(new_parent_bounds);
            new_parent.parent = old_parent;
            new_parent.child_1 = Some(best_sibling_index);
            new_parent.child_2 = Some(leaf_index);

            self.nodes.push(new_parent);
            let new_parent_index = self.nodes.len() - 1;

            self.nodes.get_unchecked_mut(best_sibling_index).parent = Some(new_parent_index);
            self.nodes.get_unchecked_mut(leaf_index).parent = Some(new_parent_index);

            match old_parent {
                Some(parent_index) => {
                    let parent = self.nodes.get_unchecked_mut(parent_index);
                    if parent.child_1 == Some(best_sibling_index) {
                        parent.child_1 = Some(new_parent_index);
                    } else {
                        parent.child_2 = Some(new_parent_index);
                    }
                }
                None => {
                    self.root = new_parent_index;
                }
            }

            self.fix_upwards(old_parent);
        }
    }

    #[inline(always)]
    fn find_best_sibling(&self, leaf_bounds: &AABB) -> usize {
        unsafe {
            let mut index = self.root;

            let mut search;
            let mut child_1; let mut child_2;
            let mut cost_1; let mut cost_2;
            loop {
                search = self.nodes.get_unchecked(index);
                if search.child_1.is_none() {
                    return index;
                }

                child_1 = self.nodes.get_unchecked(search.child_1.unwrap_unchecked());
                child_2 = self.nodes.get_unchecked(search.child_2.unwrap_unchecked());

                cost_1 = child_1.bounds.union(*leaf_bounds).inseam();
                cost_2 = child_2.bounds.union(*leaf_bounds).inseam();

                index = if cost_1 < cost_2 { 
                    search.child_1.unwrap_unchecked()
                } else { 
                    search.child_2.unwrap_unchecked() 
                };
            }
        }
    }

    #[inline(always)]
    fn fix_upwards(&mut self, mut index: Option<usize>) {
        let mut new_bounds: AABB;
        let mut c1; let mut c2; let mut updated;
        
        unsafe {
            while let Some(i) = index {
                new_bounds = {
                    let n = &self.nodes.get_unchecked(i);
                    c1 = self.nodes.get_unchecked(n.child_1.unwrap_unchecked());
                    c2 = self.nodes.get_unchecked(n.child_2.unwrap_unchecked());

                    c1.bounds.union(c2.bounds)
                };
                
                updated = self.nodes.get_unchecked_mut(i);

                updated.bounds = new_bounds;
                index = updated.parent;
            }
        }
    }

    pub fn get_debug_info(&self) -> Vec<(usize, AABB)> {
        let mut out = vec![];

        let mut stack = SmallVec::<[&usize; 128]>::new();
        stack.push(&self.root);

        while let Some(index) = stack.pop() {

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

        out
    }
}