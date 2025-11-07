use glam::{Vec2, i16vec2, ivec2, vec2};
use smallvec::{smallvec, SmallVec};
use crate::shapes::{self, AABB, AABBI32};

pub struct Node<T> {
    pub bounds: AABBI32,                      // 16 bytes
    pub elements: SmallVec<[(T, AABBI32); 16]>,      // 24 bytes
    pub children: Option<[Box<Node<T>>; 4]>,  // 4 * 8 bytes = 32 bytes
}

impl<T> Node<T> where T: Clone {
    pub fn new(bounds: AABBI32, _depth: u8) -> Self {
        Self {
            bounds,
            elements: SmallVec::new(),
            children: None,
        }
    }

    pub fn get_debug_info(&self, out: &mut Vec<(usize, AABBI32)>) {
        match &self.children {
            Some(children) => {
                for child in children {
                    child.get_debug_info(out);
                }
            },
            None => {}
        };
        
        out.push((self.elements.len(), self.bounds));
    }


    pub fn query<'a>(&'a self, bounds: &AABBI32, out: &mut SmallVec<[&'a (T, AABBI32); 8]>) {
        self.query_recursive(bounds, out);
        // outmost layer may contain items that are outside of the bounds
        if !bounds.overlaps_aabb(&self.bounds) {

            self.elements.iter().for_each(|el| {
                if bounds.overlaps_aabb(&el.1) {
                    out.push(el);
                }   
            });
        }
    }
    
    fn query_recursive<'a>(&'a self, bounds: &AABBI32, out: &mut SmallVec<[&'a (T, AABBI32); 8]>) {
        match &self.children {
            Some(children) => {
                for child in children {
                    if bounds.overlaps_aabb(&child.bounds) {
                        child.query_recursive(bounds, out);
                    }
                };
            },
            None => {}
        }

        if bounds.overlaps_aabb(&self.bounds) {
            for el in &self.elements {
                if bounds.overlaps_aabb(&el.1) {
                    out.push(&el);
                }   
            }
        }
    }

    pub fn insert(&mut self, data: &T, bounds: &AABBI32, (depth, max_depth): (u8, u8), should_rebalance: bool) {
        match &mut self.children {
            Some(children) => {
                for child in children {
                    if bounds.is_within_aabb(&child.bounds) {
                        child.insert(data, bounds, (depth + 1, max_depth), should_rebalance);
                        return;
                    }
                };
            },
            None => {}
        };

        self.elements.push((data.clone(), *bounds));
        if should_rebalance && self.children.is_none() && self.elements.len() > 8 && depth < max_depth  {
            self.rebalance((depth, max_depth));
        }
    }

    pub fn rebalance(&mut self, (depth, max_depth): (u8, u8)) {
        let d = depth + 1;
        let size = self.bounds.size() / 2;

        let create_child = |pos| {
            Box::new(Node::new(AABBI32 { min: pos, max: pos + size }, d))
        };

        
        match self.children {
            Some(_) => {},
            None => {
                self.children = Some([
                    create_child(ivec2(self.bounds.pos().x, self.bounds.pos().y + size.y)),
                    create_child(self.bounds.center()),
                    create_child(self.bounds.bottom_left()),
                    create_child(ivec2(self.bounds.pos().x + size.x, self.bounds.pos().y)),
                ]);
            },
        }

        let to_replace = std::mem::replace(&mut self.elements, smallvec![]);

        for el in to_replace {
            let mut inserted = false;

            match &mut self.children {
                Some(children) => {
                    for child in children {
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
                for child in children {
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
}

pub struct QuadTree<T> {
    pub root: Node<T>,
    max_depth: u8,
}

impl<T: Clone> QuadTree<T> {
    pub fn new(width: f32, height: f32, max_depth: u8) -> Self {
        let bounds = AABBI32::new(Vec2::new(-width, -height), Vec2::new(width, height));
        Self {
            root: Node::new(bounds, 0),
            max_depth,
        }
    }

    pub fn insert(&mut self, data: T, shape: &impl shapes::Shape) {
        self.root.insert(&data, &shape.bounds().as_aabbi32(), (0, self.max_depth), false);
    }

    pub fn insert_with_rebalance(&mut self, data: T, shape: &impl shapes::Shape) {
        self.root.insert(&data, &shape.bounds().as_aabbi32(), (0, self.max_depth), true);
    }

    pub fn remove_all(&mut self, to_remove: &mut Vec<Option<T>>) where T: PartialEq {
        self.root.remove_all(to_remove);
        to_remove.clear();
    }

    pub fn get_debug_info(&self) -> Vec<(usize, AABBI32)> {
        let mut out = Vec::with_capacity(1024);
        self.root.get_debug_info(&mut out);
        out
    }
}