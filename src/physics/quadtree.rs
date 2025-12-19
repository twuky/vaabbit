use glam::{vec2, Vec2};
use smallvec::{smallvec, SmallVec};
use crate::shapes::{self, AABB};

const MAX_ELEMENTS: usize = 16;

pub struct Node<T> {
    pub bounds: AABB,                      // 16 bytes
    pub children: Option<Box<[Node<T>; 4]>>,  // 4 * 8 bytes = 32 bytes
    pub elements: SmallVec<[(T, AABB); 32]>,      // 24 bytes
    
}

impl<T> Node<T> where T: Clone {
    pub fn new(bounds: AABB, _depth: u8) -> Self {
        Self {
            bounds,
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
        
        out.push((self.elements.len(), self.bounds));
    }

    fn collect_all<'a>(&'a self, out: &mut SmallVec<[&'a (T, AABB); 32]>) {
        out.extend(&self.elements);
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.collect_all(out);
            }
        }
    }

    pub fn query<'a>(&'a self, bounds: &AABB, out: &mut SmallVec<[&'a (T, AABB); 32]>) {
        let mut stack = SmallVec::<[&Node<T>; 64]>::new();
        stack.push(self);

        while let Some(node) = stack.pop() {
            if let Some(children) = &node.children {
                for child in children.iter() {
                    if bounds.overlaps_aabb(&child.bounds) {
                        stack.push(child);
                    }
                }
            }

            out.extend(&node.elements);
        }
    }

    fn query_recursive<'a>(&'a self, bounds: &AABB, out: &mut SmallVec<[&'a (T, AABB); 32]>) {
        match &self.children {
            Some(children) => {
                for child in children.iter() {
                    if bounds.overlaps_aabb(&child.bounds) {
                        child.query_recursive(bounds, out);
                    }
                };
            },
            None => {}
        }

        out.extend(&self.elements);
    }

    pub fn insert(&mut self, data: &T, bounds: &AABB, (depth, max_depth): (u8, u8), should_rebalance: bool) {
        match &mut self.children {
            Some(children) => {
                for child in children.iter_mut() {
                    if bounds.is_within_aabb(&child.bounds) {
                        child.insert(data, bounds, (depth + 1, max_depth), should_rebalance);
                        return;
                    }
                };
            },
            None => {}
        };

        // as a last resort, it is outside the tree, so this should be the root
        self.elements.push((data.clone(), *bounds));

        if should_rebalance && self.children.is_none() && self.elements.len() > MAX_ELEMENTS && depth < max_depth  {
            self.rebalance((depth, max_depth));
        }
    }

    pub fn rebalance(&mut self, (depth, max_depth): (u8, u8)) {
        let d = depth + 1;
        let size = self.bounds.size() / 2.0;

        let create_child = |pos| {
            // the bounds are slightly expanded to avoid placing too many objects on "edges" (ie parent node)
            Node::new(AABB { min: pos - vec2(8.0, 8.0), max: pos + size + vec2(8.0, 8.0) }, d)
        };

        
        match self.children {
            Some(_) => {},
            None => {
                self.children = Some(Box::new([
                    create_child(vec2(self.bounds.pos().x, self.bounds.pos().y + size.y)),
                    create_child(self.bounds.center()),
                    create_child(self.bounds.bottom_left()),
                    create_child(vec2(self.bounds.pos().x + size.x, self.bounds.pos().y)),
                ]));
            },
        }

        let to_replace = std::mem::replace(&mut self.elements, smallvec![]);

        for el in to_replace {
            let mut inserted = false;

            match &mut self.children {
                Some(children) => {
                    for child in children.iter_mut() {
                        if el.1.is_within_aabb(&child.bounds) {
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

impl<T: Clone> QuadTree<T> {
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