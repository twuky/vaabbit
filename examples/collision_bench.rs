use std::time::SystemTime;

use glam::Vec2;
use vaabbit::{Actor, ID, TypedID, World, physics::PhysicsClass, shapes::{AABB, Collider}, world};
use vibbit::{Color, Vibbit};
use macroquad::{color, prelude::rand};

static mut DT : f32 = 0.0;

struct Rect {
    vel: glam::Vec2,
}

impl Rect {
    fn new() -> Self {
        Self {
            vel: glam::Vec2::new(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)),
        }
    }
}

impl Actor<()> for Rect {

    fn init_physicsbody(id:TypedID) -> vaabbit::physics::PhysicsBody where Self: Sized {
        vaabbit::physics::PhysicsBody::new(Vec2::ZERO, Some(Collider::AABB(AABB { min: Vec2::ZERO, max: glam::Vec2::new(32.0, 32.0)})), id, PhysicsClass::Actor)
    }

    fn update(&mut self, _id: &ID<Self>, _world: &mut World, ctx: &mut ()) {
        let pos = self.move_by(&(self.vel * unsafe{DT} * 60.0), _world);
        
        if pos.x < 0.0 || pos.x > 1280.0 - 32.0 {
            self.vel.x *= -1.0;
        }
        if pos.y < 0.0 || pos.y > 720.0 - 32.0 {
            self.vel.y *= -1.0;
        }
    }

    fn on_collision(&mut self, _id: &ID<Self>, _other: TypedID, _world: &mut World) {
        self.vel *= -1.0;
    }
}

fn main() {
    let mut vib = Vibbit::new(1280, 720, "bunnymark");
    vib.set_target_fps(0.0);
    let mut world = vaabbit::world::World::new();

    rand::srand(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64);

    let offset = glam::Vec2::new(-640.0, -360.0);
    let offset = glam::Vec2::new(-640.0, -360.0);

    for _ in 0..1000 {
        let id = world.add_actor(Rect::new());
        world.set_pos(id, glam::Vec2::new(rand::gen_range(0.0, 1280.0 - 32.0), rand::gen_range(0.0, 720.0 - 32.0)));
    }
    let font = vib.default_font();

    while !vib.should_close() {
        world.update_systems(&mut ());

        unsafe {DT = vib.get_delta_time(); }

        vib.clear_screen(Color::new(64,64,64,255));

        for (id, rect) in world.query::<Rect>() {
            let pos = world.get_pos(id).clone();
            let mut color = Color::new(255,255,255,255);
            let collided = world.get_colliding_bodies(&id).len();

            if collided > 0 {
                color = Color::new(255,0,0,255);
            }
            vib.draw_rect(pos + offset, 32.0, 32.0, color);
            vib.draw_rect(pos + offset, 32.0, 32.0, color);
            //vib.draw_text(&font, pos.x + offset.x, pos.y + offset.y, Color::new(0,0,0,255), &format!("{}", rect.collided), 1.0);
        }

        vib.end_frame();
    }
}