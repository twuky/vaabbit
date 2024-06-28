use super::{Circle, Edge, Shape};


pub fn overlaps_edge_edge(a: &Edge, b: &Edge) -> bool {
    let (a0, b0) = (a.a, a.b);
    let (c0, d0) = (b.a, b.b);

    let ab = b0 - a0;
    let cd = d0 - c0;
    let ca = a0 - c0;

    let denominator = ab.perp_dot(cd);
    if denominator == 0.0 {
        return false; // parallel lines
    }

    let t = ca.perp_dot(cd) / denominator;
    let u = ca.perp_dot(ab) / denominator;

    t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0
}

pub fn overlaps_edge_circle(a: &Edge, b: &Circle) -> bool {
    let ab = a.b - a.a;
    let ac = b.pos - a.a;

    let e = ac.dot(ab) / ab.length_squared();
    let closest_point = if e < 0.0 {
        a.a
    } else if e > 1.0 {
        a.b
    } else {
        a.a + e * ab
    };

    closest_point.distance(b.pos) < b.radius
}

pub fn overlaps_poly_edge(a: &impl Shape, b: &Edge) -> bool {
    if let Some(aabb_edges) = a.edges() {
        for aabb_edge in aabb_edges {
            if b.overlaps_edge(&aabb_edge) {
                return true;
            }
        }
    };
    false
}

pub fn overlaps_poly_poly(a: &impl Shape, b: &impl Shape) -> bool {
    let self_edges = if let Some(edges) = a.edges() {
        edges
    } else { return false; };

    let self_verts = if let Some(verts) = a.vertices() {
        verts
    } else { return false; };

    let other_verts = if let Some(verts) = b.vertices() {
        verts
    } else { return false; };

    for edge in &self_edges {
        let axis = edge.perpendicular_dir();

        let (mut min_self, mut max_self) = (f32::INFINITY, f32::NEG_INFINITY);
        for &vert in &self_verts {
            let dot = axis.dot(vert);
            if dot < min_self {
                min_self = dot;
            }
            if dot > max_self {
                max_self = dot;
            }
        }
        let (mut min_other, mut max_other) = (f32::INFINITY, f32::NEG_INFINITY);
        for &vert in &other_verts {
            let dot = axis.dot(vert);
            if dot < min_other {
                min_other = dot;
            }
            if dot > max_other {
                max_other = dot;
            }
        }
        if min_self > max_other || min_other > max_self {
            return false;
        }
    }

    true
}

pub fn overlaps_poly_circle(a: &impl Shape, b: &Circle) -> bool {
    if let Some(edges) = a.edges() {
        for edge in edges {
            if edge.overlaps_circle(&b) { return true }
        }
    }
    
    false
}