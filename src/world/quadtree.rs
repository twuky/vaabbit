use glam::{vec2, Vec2};
use pulz_arena::{Arena, Index};

use crate::shapes::{self, Shape, AABB};

pub struct Node {
    pub bounds: AABB,                      // 16 bytes
    pub elements: Vec<(Index, AABB)>,      // 24 bytes
    depth: u8,                             // 1 byte
    // 3 bytes padding to align the next field
    pub children: [Option<Box<Node>>; 4],  // 4 * 8 bytes = 32 bytes
}

impl Node {
    pub fn new(bounds: AABB, depth: u8) -> Self {
        Self {
            bounds,
            elements: Vec::new(),
            depth,
            children: [None, None, None, None],
        }
    }

    pub fn query(&self, bounds: &AABB) -> Vec<(Index, AABB)> {
        let mut out: Vec<(Index, AABB)> = vec![];

        if bounds.overlaps_aabb(&self.bounds) {
            out.extend(&self.elements);
            // for el in &self.elements {
            //     if bounds.overlaps_aabb(&el.1) {
            //         out.push(*el)
            //     }
            // }
        }

        for child in &self.children {
            if let Some(child_node) = child {
                out.extend(&child_node.query(bounds));
            }
        }

        out
    }

    pub fn insert(&mut self, data: Index, bounds: &AABB, (depth, max_depth): (u8, u8)) {
        if bounds.overlaps_aabb(&self.bounds) && !bounds.is_within_aabb(&self.bounds) {
            self.elements.push((data, *bounds));
            if self.elements.len() > max_depth as usize {
                self.rebalance((depth, max_depth));
            }
            return;
        }

        for child in &mut self.children {
            if let Some(child_node) = child {
                if bounds.is_within_aabb(&child_node.bounds) {
                    child_node.insert(data, bounds, (depth + 1, max_depth));
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

        match self.children[0] {
            Some(_) => {},
            None => {
                self.children[0] = Some(create_child(vec2(self.bounds.pos.x, self.bounds.pos.y + size.y)));
                self.children[1] = Some(create_child(self.bounds.center()));
                self.children[2] = Some(create_child(self.bounds.bottom_left()));
                self.children[3] = Some(create_child(vec2(self.bounds.pos.x + size.x, self.bounds.pos.y)));
            },
        }

        let to_replace = std::mem::replace(&mut self.elements, vec![]);

        for el in to_replace {
            let mut inserted = false;
            for child in &mut self.children {
                if let Some(child_node) = child {
                    if el.1.is_within_aabb(&child_node.bounds) {
                        child_node.insert(el.0, &el.1, (d, max_depth));
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
