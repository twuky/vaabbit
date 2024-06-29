use glam::{vec2, Vec2};

use crate::shapes::{self, Shape, AABB};

pub struct Node<T> {
    pub bounds: AABB,
    pub elements: Vec<(T, AABB)>,
    depth: u8,

    pub nw: Option<Box<Node<T>>>,
    pub ne: Option<Box<Node<T>>>,
    pub sw: Option<Box<Node<T>>>,
    pub se: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(bounds: AABB, depth: u8) -> Self {
        Self {
            bounds,
            elements: vec![],
            depth,

            nw: None,
            ne: None,
            sw: None,
            se: None,
        }
    }

    pub fn query(&self, bounds: &AABB) -> Vec<AABB> {
        let mut out = vec![];

        if bounds.bounds_overlaps_bounds(self.bounds) {
            for el in &self.elements {
                if bounds.bounds_overlaps_bounds(el.1) {
                    out.push(el.1)
                }
            }
        }

        if let Some(nw) = &self.nw {
            out.extend(&nw.query(bounds))
        }

        if let Some(ne) = &self.ne {
            out.extend(&ne.query(bounds))
        }

        if let Some(sw) = &self.sw {
            out.extend(&sw.query(bounds))
        }

        if let Some(se) = &self.se {
            out.extend(&se.query(bounds))
        }

        out
    }

    pub fn insert(&mut self, data: T, bounds: &AABB, (depth, max_depth): (u8, u8)) {
        // if at max depth or tree empty, add to self
        if depth == max_depth
            || self.elements.len() <= max_depth as usize
        {
            self.elements.push((data, *bounds));

            if self.elements.len() > max_depth as usize {
                self.rebalance((depth, max_depth));
            }
            return;
        }

        // otherwise, add to children
        if let Some(nw) = &mut self.nw {
            if bounds.is_within_aabb(&nw.bounds) {
                nw.insert(data, bounds, (depth + 1, max_depth));
                return;
            }
        }

        if let Some(ne) = &mut self.ne {
            if bounds.is_within_aabb(&ne.bounds) {
                ne.insert(data, bounds, (depth + 1, max_depth));
                return;
            }
        }

        if let Some(sw) = &mut self.sw {
            if bounds.is_within_aabb(&sw.bounds) {
                sw.insert(data, bounds, (depth + 1, max_depth));
                return;
            }
        }

        if let Some(se) = &mut self.se {
            if bounds.is_within_aabb(&se.bounds) {
                se.insert(data, bounds, (depth + 1, max_depth));
                return;
            }
        }

        // all else
        if bounds.bounds_overlaps_bounds(self.bounds) && !bounds.is_within_aabb(&self.bounds) {
            self.elements.push((data, *bounds));
            if self.elements.len() > max_depth as usize {
                self.rebalance((depth, max_depth));
            }
        }
    }

    pub fn rebalance(&mut self, (depth, max_depth): (u8, u8)) {
        let d = depth + 1;
        let size = self.bounds.size / 2.0;

        let nw = self.nw.get_or_insert_with(|| {
            Box::new(Node::new(
                AABB {
                    pos: vec2(self.bounds.pos.x, self.bounds.pos.y + size.y),
                    size,
                },
                d,
            ))
        });
        let ne = self.ne.get_or_insert_with(|| {
            Box::new(Node::new(
                AABB {
                    pos: self.bounds.center(),
                    size,
                },
                d,
            ))
        });
        let sw = self.sw.get_or_insert_with(|| {
            Box::new(Node::new(
                AABB {
                    pos: self.bounds.bottom_left(),
                    size,
                },
                d,
            ))
        });
        let se = self.se.get_or_insert_with(|| {
            Box::new(Node::new(
                AABB {
                    pos: vec2(self.bounds.pos.x + size.x, self.bounds.pos.y),
                    size,
                },
                d,
            ))
        });

        let to_replace = std::mem::replace(&mut self.elements, vec![]);

        for el in to_replace {
            if el.1.is_within_aabb(&nw.bounds) {
                nw.insert(el.0, &el.1, (d, max_depth));
            } else if el.1.is_within_aabb(&ne.bounds) {
                ne.insert(el.0, &el.1, (d, max_depth));
            } else if el.1.is_within_aabb(&sw.bounds) {
                sw.insert(el.0, &el.1, (d, max_depth));
            } else if el.1.is_within_aabb(&se.bounds) {
                se.insert(el.0, &el.1, (d, max_depth));
            } else {
                self.elements.push(el)
            }
        }
    }
}

pub struct QuadTree<T> {
    pub root: Box<Node<T>>,
    max_depth: u8,
}

impl<T> QuadTree<T> {
    pub fn new(width: f32, height: f32, max_depth: u8) -> Self {
        let bounds = AABB {
            pos: Vec2::new(0.0, 0.0),
            size: Vec2::new(width, height),
        };
        Self {
            root: Box::new(Node::<T>::new(bounds, 0)),
            max_depth,
        }
    }

    pub fn insert(&mut self, data: T, shape: impl shapes::Shape) {
        self.root.insert(data, &shape.bounds(), (0, self.max_depth))
    }
}
