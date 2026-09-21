use glam::Vec2;
use vaabbit::{Actor, ID, TypedID, World, physics::PhysicsClass, shapes::{AABB, Collider}};
use vibbit::Vibbit;
use macroquad::{prelude::rand};

static mut DT : f32 = 0.0;
static BLOCK_SIZE : f32 = 32.0;

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
        vaabbit::physics::PhysicsBody::new(Vec2::ZERO, Some(Collider::AABB(AABB { min: Vec2::ZERO, max: glam::Vec2::new(BLOCK_SIZE, BLOCK_SIZE)})), id, PhysicsClass::Actor)
    }

    fn update(&mut self, _id: &ID<Self>, _world: &mut World, _ctx: &mut ()) {
        let pos = self.move_by(&(self.vel * unsafe{DT} * 60.0), _world);
        
        if pos.x < 0.0 || pos.x > 1280.0 - BLOCK_SIZE {
            self.vel.x *= -1.0;
        }
        if pos.y < 0.0 || pos.y > 720.0 - BLOCK_SIZE {
            self.vel.y *= -1.0;
        }
    }

    fn on_collision(&mut self, _id: &ID<Self>, _other: TypedID, _world: &mut World) {
        self.vel *= -1.0;
    }
}

fn main() {
    let mut vib = Vibbit::new(1280, 720, "bunnymark");
    vib.cfg.set_target_fps(0.0);
    vib.cfg.window_show_fps(true);
    let mut world = vaabbit::world::World::new();

    let offset = glam::Vec2::new(-640.0, -360.0);

    for _ in 0..1000 {
        let id = world.add_actor(Rect::new());
        world.set_pos(id, vib.rand_vec2(0.0..1280.0 - BLOCK_SIZE, 0.0..720.0 - BLOCK_SIZE));
    }

    while !vib.should_close() {
        world.update_systems(&mut ());
        
        unsafe {DT = vib.get_delta_time(); }

        vib.clear_screen(vibbit::DARKGREY);

        for (id, _rect) in world.query::<Rect>() {
            let pos = world.get_pos(id).clone();

            let color = if world.get_colliding_bodies(&id).len() > 0 {   
                vibbit::RED
            } else {
                vibbit::WHITE
            };

            vib.draw_rect(pos + offset, BLOCK_SIZE, BLOCK_SIZE, color);
        }
        vib.end_frame();
    }
}