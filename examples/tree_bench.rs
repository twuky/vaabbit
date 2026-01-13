use glam::Vec2;
use vaabbit::{physics::{dynamictree, quadtree}, shapes::AABB};



fn main() {
    macroquad::prelude::rand::srand(8694);

    let total = 1000;

    let mut items = Vec::new();
    for i in 0..total {
        let pos = Vec2::new(macroquad::prelude::rand::gen_range(0.0, 1000.0), macroquad::prelude::rand::gen_range(0.0, 1000.0));
        let size = Vec2::new(30.0, 30.0);

        let rect = AABB::from_pos_size(pos, size);
        items.push((i, rect));
    }

    // bench quadtree creation
    let t = std::time::Instant::now();
    let mut quadtree = quadtree::QuadTree::<u32>::new(1000.0, 1000.0, 8);
    
    for item in &items {
        let _ = quadtree.insert(item.0, &item.1);
    }
    quadtree::QuadTree::<u32>::new(1000.0, 1000.0, 8);
    println!("quadtree creation time: {}", t.elapsed().as_secs_f64());

    // bench dynamic tree creation
    let t = std::time::Instant::now();
    let mut dynamic_tree = dynamictree::DynamicTree::<u32>::new();
    
    for item in &items {
        let _ = dynamic_tree.insert(item.0, &item.1);
    }
    println!("dynamictree creation time: {}", t.elapsed().as_secs_f64());


    let mut queries = Vec::new();
    for _ in 0..total {
        let pos = Vec2::new(macroquad::prelude::rand::gen_range(0.0, 1000.0), macroquad::prelude::rand::gen_range(0.0, 1000.0));
        let size = Vec2::new(macroquad::prelude::rand::gen_range(5.0, 30.0), macroquad::prelude::rand::gen_range(5.0, 30.0));

        let rect = AABB::from_pos_size(pos, size);
        queries.push(rect);
    }


    // correctness tests
    let mut results_quadtree = Vec::new();
    let mut results_dynamic_tree = Vec::new();

    // bench quadtree query time
    let t = std::time::Instant::now();
    for rect in &queries {
        let res = quadtree.query(rect);
        results_quadtree.push(res);
    }
    println!("quadtree query time: {}", t.elapsed().as_secs_f64());

    // bench dynamic tree query time
    let t = std::time::Instant::now();
    for rect in &queries {
        let res = dynamic_tree.query(&rect);
        results_dynamic_tree.push(res);
    }
    println!("dynamic tree query time: {}", t.elapsed().as_secs_f64());

    for (q_res, d_res) in results_quadtree.iter().zip(results_dynamic_tree.iter()) {
        println!("length q: {}, d: {}", q_res.len(), d_res.len());
    }
    
}