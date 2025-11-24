use std::time::SystemTime;

use vaabbit::{World, Actor, ID, TypedID};
use vibbit::{Color, Vibbit};
use macroquad::prelude::rand;

static mut DT : f32 = 0.0;

struct Rect {
    vel: glam::Vec2,
    color: Color,
}

impl Rect {
    fn new() -> Self {
        Self {
            vel: glam::Vec2::new(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)),
            color: Color::new(255,255,255,255),
        }
    }
}

impl Actor for Rect {
    fn update(&mut self, _id: &ID<Self>, _world: &mut World) {
        self.color = Color::new(255,255,255,255);
        let pos = self.move_by(&(&self.vel * unsafe{DT} * 60.0), _id, _world);
        
        if pos.x < 0.0 || pos.x > 1280.0 - 32.0 {
            self.vel.x *= -1.0;
        }
        if pos.y < 0.0 || pos.y > 720.0 - 32.0 {
            self.vel.y *= -1.0;
        }
    }

    fn on_collision(&mut self, _id: &ID<Self>, _other: TypedID, _world: &mut World) {
        // println!("collision");
        self.vel *= -1.0;
        self.color = Color::new(255,0,0,255);
    }
}
fn main() {
    let mut vib = Vibbit::new(1280, 720, "bunnymark");
    vib.set_target_fps(0.0);
    let mut world = vaabbit::world::World::new();

    rand::srand(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64);

    let offset = vibbit::Vec2::new(-640.0, -360.0);

    for _ in 0..100 {
        let id = world.add_actor(Rect::new());
        world.set_pos(id, glam::Vec2::new(rand::gen_range(0.0, 1280.0 - 32.0), rand::gen_range(0.0, 720.0 - 32.0)));
    }

    while !vib.should_close() {
        world.update_systems();

        unsafe {DT = vib.get_delta_time(); }

        vib.clear_screen(Color::new(0,0,0,255));

        for (_id, rect) in world.query_id::<Rect>() {
            let pos = world.get_pos(_id);
            vib.draw_rect(vibbit::Vec2::new(pos.x, pos.y) + offset, 32.0, 32.0, rect.color);
        }

        vib.end_frame();
    }
}