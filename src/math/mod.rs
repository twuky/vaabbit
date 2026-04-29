
use glam::Vec2;

pub fn bresenham_line(start: Vec2, end: Vec2) -> Vec<Vec2> {
    let mut points = Vec::new();
    let mut current = start.as_ivec2();
    let end = end.as_ivec2();

    // delta (overall change) in x and y
    let dx = (end.x - current.x).abs();
    let dy = -(end.y - current.y).abs();

    // sign of the deltas, for handling negative values
    let sx = if current.x < end.x { 1 } else { -1 };
    let sy = if current.y < end.y { 1 } else { -1 };

    let mut err = dx + dy;

    loop {
        points.push(current.as_vec2());

        if current == end {
            break;
        }

        let e2 = 2 * err;

        if e2 >= dy {
            err += dy;
            current.x += sx;
        }

        if e2 <= dx {
            err += dx;
            current.y += sy;
        }
    }
    points 
}

/**
 * Returns a list of "pixel movements" from the start point to the end point
 * Can be used as "instructions", ie. 'move up, move up, move left, move up'
 */
pub fn bresenham_line_movement(start: Vec2, end: Vec2) -> Vec<Vec2> {
    let mut points = Vec::new();
    let mut current = start.as_ivec2();
    let end = end.as_ivec2();

    // delta (overall change) in x and y
    let dx = (end.x - current.x).abs();
    let dy = -(end.y - current.y).abs();

    // sign of the deltas, for handling negative values
    let sx = if current.x < end.x { 1 } else { -1 };
    let sy = if current.y < end.y { 1 } else { -1 };

    let mut err = dx + dy;

    loop {
        if current == end {
            break;
        }

        let e2 = 2 * err;

        if e2 >= dy {
            err += dy;
            points.push(Vec2::new(sx as f32, 0.0));
            current.x += sx;
        }

        if e2 <= dx {
            err += dx;
            points.push(Vec2::new(0.0, sy as f32));
            current.y += sy;
        }
    }
    points 
}