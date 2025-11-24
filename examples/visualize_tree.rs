use std::time::Instant;

use ::glam::Vec2;
use miniquad::window::set_window_size;
use smallvec::SmallVec;
use vaabbit::{shapes::*, physics::quadtree::{Node, QuadTree}};
use macroquad::{prelude::*};

#[macroquad::main("visualize_tree")]
async fn main() {
    set_window_size(800, 800);
    rand::srand(8694);
    let mut tree = QuadTree::<u32>::new(600.0, 600.0, 8);

    let t = Instant::now();
    for i in 0..300 {
        let pos = Vec2::new(300.0, 300.0) + Vec2::new(rand::gen_range(-280.0, 280.0), rand::gen_range(-280.0, 280.0));
        let size = Vec2::new(rand::gen_range(5.0, 30.0), rand::gen_range(5.0, 30.0));

        let rect = AABB::from_pos_size(pos, size);
        
        tree.insert_with_rebalance(i, &rect);
    }
    let dur = Instant::now() - t;
    println!("tree time: {}ms", dur.as_secs_f64() * 1000.0);

    loop {
        
        rand::srand(4);
        draw_node(&tree.root);
        let (mut x, mut y) = mouse_position();
        x -= 25.0;
        y -= 25.0;

        let test = AABB::from_pos_size(Vec2::new(x, y) + Vec2::new(-100.0, -100.0), Vec2::new(50.0, 50.0));
        
        let t = Instant::now();
        let mut overlaps = SmallVec::new();
        tree.root.query(&test, &mut overlaps);
        let dur = Instant::now() - t;
        draw_text(&format!("fps: {}, q time: {}", get_fps(), dur.as_secs_f64() * 1000.0), 4.0, 24.0, 24.0, WHITE);

        
        for item in overlaps {
            draw_aabb(item.1, WHITE);
            if item.1.overlaps_aabb(&test) {
                draw_aabb(item.1, RED)
            }
        }

        draw_rectangle_lines(x, y, 50.0, 50.0, 1.0, YELLOW);
        next_frame().await
    }
    
}

fn draw_node<T>(node: &Node<T>) {
    let c = Color::new(rand::gen_range(0.0, 1.0), rand::gen_range(0.0, 1.0), rand::gen_range(0.0, 1.0), 1.0);
    draw_aabb(node.bounds, c);
    draw_text(&node.elements.len().to_string(), 100. + node.bounds.center().x, 100. + node.bounds.center().y, 16.0, c);

    for e in &node.elements {
        draw_aabb(e.1, c)
    }

    if let Some(children) = &node.children {
        for i in children {
            draw_node(&i)
        }
    }
}

fn draw_aabb(aabb: AABB, color: Color) {
    draw_rectangle_lines(100.0 + aabb.min.x, 100.0 + aabb.min.y, aabb.size().x, aabb.size().y, 1.0, color)
}