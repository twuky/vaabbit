use glam::{vec2, Vec2};
use smallvec::{smallvec, SmallVec};
use crate::shapes::{self, AABB};

const MAX_ELEMENTS: usize = 16;

pub struct Node<T> {
    pub node_bounds: AABB,
    pub children: Option<Box<[Node<T>; 4]>>,
    pub elements: SmallVec<[(T, AABB); 16]>,
}

impl<T> Node<T> where T: Copy {
    pub fn new(bounds: AABB, _depth: u8) -> Self {
        Self {
            node_bounds: bounds,
            elements: SmallVec::new(),
            children: None,
        }
    }

    pub fn get_debug_info(&self, out: &mut Vec<(usize, AABB)>) {
        match &self.children {
            Some(children) => {
                for child in children.iter() {
                    child.get_debug_info(out);
                }
            },
            None => {}
        };
        
        out.push((self.elements.len(), self.node_bounds));
    }

    fn collect_all<'a>(&'a self, out: &mut SmallVec<[&'a (T, AABB); 32]>) {
        out.extend(&self.elements);
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.collect_all(out);
            }
        }
    }

    #[inline(always)]
    pub fn query<'a>(&'a self, bounds: &AABB, out: &mut SmallVec<[&'a (T, AABB); 16]>) {
        let mut stack = SmallVec::<[&Node<T>; 32]>::new();
        stack.push(self);

        while let Some(node) = stack.pop() {
            if let Some(children) = &node.children {
                for child in children.iter() {
                    if bounds.overlaps_aabb(&child.node_bounds) {
                        stack.push(child);
                    }
                }
            }

            for e in &node.elements {
                if bounds.overlaps_aabb(&e.1) {
                    out.push(e);
                }
            }
           // out.extend(&node.elements);
        }
    }

    pub fn insert(&mut self, data: &T, bounds: &AABB, (depth, max_depth): (u8, u8), should_rebalance: bool) {
        match &mut self.children {
            Some(children) => {
                for child in children.iter_mut() {
                    let mut expanded = child.node_bounds;
                    expanded.expand(32.0);
                    if bounds.is_within_aabb(&expanded) {
                        child.insert(data, bounds, (depth + 1, max_depth), should_rebalance);
                        return;
                    }
                };
            },
            None => {}
        };

        // as a last resort, it is outside the tree, so this should be the root
        self.elements.push((*data, *bounds));

        if should_rebalance && self.children.is_none() && self.elements.len() > MAX_ELEMENTS && depth < max_depth  {
            self.rebalance((depth, max_depth));
        }
    }

    pub fn rebalance(&mut self, (depth, max_depth): (u8, u8)) {
        let d = depth + 1;
        let size = self.node_bounds.size() / 2.0;

        let create_child = |pos| {
            Node::new(AABB { min: pos, max: pos + size }, d)
        };

        
        match self.children {
            Some(_) => {},
            None => {
                self.children = Some(Box::new([
                    create_child(vec2(self.node_bounds.pos().x, self.node_bounds.pos().y + size.y)),
                    create_child(self.node_bounds.center()),
                    create_child(self.node_bounds.bottom_left()),
                    create_child(vec2(self.node_bounds.pos().x + size.x, self.node_bounds.pos().y)),
                ]));
            },
        }

        let to_replace = std::mem::replace(&mut self.elements, smallvec![]);

        for el in to_replace {
            let mut inserted = false;

            match &mut self.children {
                Some(children) => {
                    for child in children.iter_mut() {
                        if el.1.is_within_aabb(&child.node_bounds) {
                            child.insert(&el.0, &el.1, (d, max_depth), true);
                            inserted = true;
                            break;
                        }
                    };
                },
                None => {}  
            }

            if !inserted {
                self.elements.push(el);
            }
        }
    }

    pub fn remove_all(&mut self, to_remove: &mut Vec<Option<T>>) where T: PartialEq {
        match &mut self.children {
            Some(children) => {
                for child in children.iter_mut() {
                    child.remove_all(to_remove);
                }
            },
            None => {}
        };
        
        self.elements.retain(|item| {
            for (i, r) in &mut to_remove.iter().enumerate() {
                if Some(item.0.clone()) == *r {
                    to_remove[i] = None;
                    return false
                }
            }
            true
        });
    }

    pub fn get_total(&self) -> usize {
        let mut total = 0;
        self.get_total_recursive(&mut total, 0);
        total
    }

    fn get_total_recursive(&self, total: &mut usize, depth: u8) {
        *total += self.elements.len();
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.get_total_recursive(total, depth + 1);
            }
        }
    }
}

pub struct QuadTree<T> {
    pub root: Node<T>,
    max_depth: u8,
}

impl<T: Clone> QuadTree<T> where T: Clone, T: Copy {
    pub fn new(width: f32, height: f32, max_depth: u8) -> Self {
        let bounds = AABB {
            min: Vec2::new(-width, -height) * 1.5,
            max: Vec2::new(width, height) * 1.5,
        };
        Self {
            root: Node::new(bounds, 0),
            max_depth,
        }
    }

    pub fn query(&self, bounds: &AABB) -> SmallVec<[&(T, AABB); 16]> {
        let mut out = smallvec![];
        self.root.query(bounds, &mut out);
        out
    }

    pub fn len(&self) -> usize {
        self.root.get_total()
    }

    pub fn insert(&mut self, data: T, shape: &impl shapes::Shape) {
        self.root.insert(&data, &shape.bounds(), (0, self.max_depth), false);
    }

    pub fn insert_with_rebalance(&mut self, data: T, shape: &impl shapes::Shape) {
        self.root.insert(&data, &shape.bounds(), (0, self.max_depth), true);
    }

    pub fn remove_all(&mut self, to_remove: &mut Vec<Option<T>>) where T: PartialEq {
        self.root.remove_all(to_remove);
        to_remove.clear();
    }

    pub fn get_debug_info(&self) -> Vec<(usize, AABB)> {
        let mut out = Vec::with_capacity(1024);
        self.root.get_debug_info(&mut out);
        out
    }
}