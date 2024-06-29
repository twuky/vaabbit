use ::glam::Vec2;
use miniquad::window::set_window_size;
use vaabbit::{shapes::*, world::quadtree::{Node, QuadTree}};
use macroquad::{color, prelude::*};

#[macroquad::main("BasicShapes")]
async fn main() {
    set_window_size(800, 800);
    rand::srand(4);
    let mut tree = QuadTree::<u8>::new(450.0, 450.0, 6);

    for i in 0..80 {
        let rect = AABB {
            pos: Vec2::new(200.0, 200.0) + Vec2::new(rand::gen_range(-200.0, 200.0), rand::gen_range(-200.0, 200.0)),
            size: Vec2::new(rand::gen_range(5.0, 30.0), rand::gen_range(5.0, 30.0))
        };
        
        tree.insert(i, rect);
    }

    loop {
        rand::srand(4);
        draw_node(&tree.root);
        let (mut x, mut y) = mouse_position();
        x -= 25.0;
        y -= 25.0;

        

        let overlaps = tree.root.query(&AABB {
            pos: Vec2::new(x, y) + Vec2::new(-200.0, -200.0),
            size: Vec2::new(50.0, 50.0)
        });

        for item in overlaps {
            draw_aabb(item, WHITE)
        }

        draw_rectangle_lines(x, y, 50.0, 50.0, 1.0, YELLOW);
        next_frame().await
    }
    
}

fn draw_node(node: &Node<u8>) {
    let c = Color::new(rand::gen_range(0.0, 1.0), rand::gen_range(0.0, 1.0), rand::gen_range(0.0, 1.0), 1.0);
    draw_aabb(node.bounds, c);
    draw_text(&node.elements.len().to_string(), 200. + node.bounds.center().x, 200. + node.bounds.center().y, 16.0, c);

    for e in &node.elements {
        draw_aabb(e.1, c)
    }

    if let Some(nw) = &node.nw {
        draw_node(&nw)
    }
    if let Some(ne) = &node.ne {
        draw_node(&ne)
    }
    if let Some(se) = &node.se {
        draw_node(&se)
    }
    if let Some(sw) = &node.sw {
        draw_node(&sw)
    }
}

fn draw_aabb(aabb: AABB, color: Color) {
    draw_rectangle_lines(200.0 + aabb.pos.x, 200.0 + aabb.pos.y, aabb.size.x, aabb.size.y, 1.0, color)
}