use ::glam::Vec2;
use vaabbit::shapes::*;
use macroquad::{color, prelude::*};

#[macroquad::main("BasicShapes")]
async fn main() {

    let mut aabb_1 = AABB {
        pos: Vec2::new(-50.0, 0.0),
        size: Vec2::new(30.0, 30.0)
    };
    let mut aabb_2 = AABB {
        pos: Vec2::new(-25.0, 0.0),
        size: Vec2::new(30.0, 30.0)
    };

    let mut circle_1 = vaabbit::shapes::Circle {
        pos: Vec2::new(25.0, 0.0),
        radius: 15.0
    };

    let mut circle_2 = vaabbit::shapes::Circle {
        pos: Vec2::new(50.0, 0.0),
        radius: 15.0
    };

    let col_grn = color::Color::new(0.0, 1.0, 0.0, 0.25);
    let col_red = color::Color::new(1.0, 0.0, 0.0, 0.25);

    loop {
        let mut red;
        let mut col;

        if is_key_down(KeyCode::Key1) {
            aabb_1.pos = mouse_position().into();
        }
        if is_key_down(KeyCode::Key2) {
            aabb_2.pos = mouse_position().into();
        }
        if is_key_down(KeyCode::Key3) {
            circle_1.pos = mouse_position().into();
        }
        if is_key_down(KeyCode::Key4) {
            circle_2.pos = mouse_position().into();
        }

        red = aabb_1.overlaps(&aabb_2) ||
            aabb_1.overlaps(&circle_1) ||
            aabb_1.overlaps(&circle_2);

        col = if red {col_red} else {col_grn};
        draw_rectangle(aabb_1.pos.x, aabb_1.pos.y, aabb_1.size.x, aabb_1.size.y, col);

        red = aabb_2.overlaps(&aabb_1) ||
            aabb_2.overlaps(&circle_1) ||
            aabb_2.overlaps(&circle_2);

        col = if red {col_red} else {col_grn};
        draw_rectangle(aabb_2.pos.x, aabb_2.pos.y, aabb_2.size.x, aabb_2.size.y, col);

        red = circle_1.overlaps(&aabb_1) ||
            circle_1.overlaps(&aabb_2) ||
            circle_1.overlaps(&circle_2);

        col = if red {col_red} else {col_grn};
        draw_circle(circle_1.pos.x, circle_1.pos.y, circle_1.radius, col);

        red = circle_2.overlaps(&aabb_1) ||
            circle_2.overlaps(&aabb_2) ||
            circle_2.overlaps(&circle_1);
            
        col = if red {col_red} else {col_grn};
        draw_circle(circle_2.pos.x, circle_2.pos.y, circle_2.radius, col);

        next_frame().await
    }
}