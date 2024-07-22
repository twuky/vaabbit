use glam::{vec2, Vec2};
use pulz_arena::{Arena, Index};

use crate::shapes::{self, Shape, AABB};

pub struct Node {
    pub bounds: AABB,                      // 16 bytes
    pub elements: Vec<(Index, AABB)>,      // 24 bytes
    pub children: Option<[Box<Node>; 4]>,  // 4 * 8 bytes = 32 bytes
}

impl Node {
    pub fn new(bounds: AABB, _depth: u8) -> Self {
        Self {
            bounds,
            elements: Vec::new(),
            children: None,
        }
    }

    pub fn query(&self, bounds: &AABB) -> Vec<(Index, AABB)> {
        let mut out: Vec<(Index, AABB)> = Vec::with_capacity(self.elements.len());
        self.query_recursive(bounds, &mut out);
        out
    }
    
    fn query_recursive(&self, bounds: &AABB, out: &mut Vec<(Index, AABB)>) {
        if bounds.overlaps_aabb(&self.bounds) {
            for el in &self.elements {
                if bounds.overlaps_aabb(&el.1) {
                    out.push(el.clone());
                }   
            }

            if let Some(children) = &self.children {
                for child in children {
                    child.query_recursive(bounds, out);
                }
            }
        }
    }

    pub fn insert(&mut self, data: Index, bounds: &AABB, (depth, max_depth): (u8, u8)) {
        if bounds.overlaps_aabb(&self.bounds) && !bounds.is_within_aabb(&self.bounds) {
            self.elements.push((data, *bounds));
            if self.elements.len() > max_depth as usize {
                self.rebalance((depth, max_depth));
            }
            return;
        }

        if let Some(children) = &mut self.children {
            for child in children {
                if bounds.is_within_aabb(&child.bounds) {
                    child.insert(data, bounds, (depth + 1, max_depth));
                    return;
                }
            }
        }

        self.elements.push((data, *bounds));
        if self.elements.len() > max_depth as usize {
            self.rebalance((depth, max_depth));
        }
    }

    pub fn rebalance(&mut self, (depth, max_depth): (u8, u8)) {
        let d = depth + 1;
        let size = self.bounds.size / 2.0;

        let create_child = |pos| {
            Box::new(Node::new(AABB { pos, size }, d))
        };

        
        match self.children {
            Some(_) => {},
            None => {
                self.children = Some([
                    create_child(vec2(self.bounds.pos.x, self.bounds.pos.y + size.y)),
                    create_child(self.bounds.center()),
                    create_child(self.bounds.bottom_left()),
                    create_child(vec2(self.bounds.pos.x + size.x, self.bounds.pos.y)),
                ]);
            },
        }

        let to_replace = std::mem::replace(&mut self.elements, vec![]);

        for el in to_replace {
            let mut inserted = false;
            if let Some(children) = &mut self.children {
                for child in children {
                    if el.1.is_within_aabb(&child.bounds) {
                        child.insert(el.0, &el.1, (d, max_depth));
                        inserted = true;
                        break;
                    }
                }
            }
            if !inserted {
                self.elements.push(el);
            }
        }
    }
}

pub struct QuadTree<T> {
    pub root: Box<Node>,
    max_depth: u8,
    items: Arena<T>
}

impl<T> QuadTree<T> {
    pub fn new(width: f32, height: f32, max_depth: u8) -> Self {
        let bounds = AABB {
            pos: Vec2::new(0.0, 0.0),
            size: Vec2::new(width, height),
        };
        Self {
            root: Box::new(Node::new(bounds, 0)),
            items: Arena::new(),
            max_depth,
        }
    }

    pub fn insert(&mut self, data: T, shape: impl shapes::Shape) {
        let idx = self.items.insert(data);
        self.root.insert(idx, &shape.bounds(), (0, self.max_depth))
    }
}
