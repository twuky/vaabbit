use vibbit::{Vibbit};
use glam::Vec2;
use vaabbit::{Actor, ID, World, shapes::AABB, shapes::Collider};
use macroquad::{prelude::rand};

struct Player {
    vel: f32,
}

struct Ground {

}

struct Block {

}

impl Actor<Vibbit> for Block {
    fn init_physicsbody(id:vaabbit::TypedID) -> vaabbit::physics::PhysicsBody where Self: Sized {
        vaabbit::physics::PhysicsBody::new(
            Vec2::new(rand::gen_range(-640.0, 640.0), rand::gen_range(-480.0, 480.0)).round(),
            Collider::aabb(Vec2::ZERO, Vec2::new(rand::gen_range(16.0, 64.0), rand::gen_range(16.0, 64.0)).round()),
            id, 
            vaabbit::physics::PhysicsClass::Solid
        )
    }

    fn update(&mut self, id: &ID<Self>, world: &mut World, vib: &mut Vibbit) where Self: Sized {
        let body = world.get_physics_body(&id).unwrap();
        let color = vibbit::Color::new(255,255,255,255);
        vib.draw_rect(body.pos(), body.bounds().width(), body.bounds().height(), color);

        if rand::gen_range(0.0, 1000.0) < 0.5 {
            world.remove_actor(id);
            world.add_actor(Block {});
        }
    }
}

struct Coin {
    eaten: bool,
}

impl Actor<Vibbit> for Coin {
    fn init_physicsbody(id:vaabbit::TypedID) -> vaabbit::physics::PhysicsBody where Self: Sized {
        vaabbit::physics::PhysicsBody::new(
            Vec2::new(rand::gen_range(-640.0, 640.0), rand::gen_range(-480.0, 480.0)),
            Collider::aabb(Vec2::ZERO, Vec2::new(8.0, 8.0)),
            id, 
            vaabbit::physics::PhysicsClass::Zone
        )
    }

    fn update(&mut self, id: &ID<Self>, world: &mut World, vib: &mut Vibbit) where Self: Sized {
        let color = vibbit::Color::new(255,255,128,255);
        vib.draw_rect(self.pos(world), 8.0, 8.0, color);
    }

    fn on_collision<'a>(&mut self, id: &ID<Self>, other: vaabbit::TypedID, world: &'a mut World) {
        // check type of collision
        if let Some(_player) = other.is::<Player>() {
            self.eaten = true;
            println!("removed");
            world.remove_actor(&id);
        }
    }
}

impl Actor<Vibbit> for Ground {
    fn init_physicsbody(id:vaabbit::TypedID) -> vaabbit::physics::PhysicsBody where Self: Sized {
        let mut body = vaabbit::physics::PhysicsBody::new(
            Vec2::new(0.0, -32.0),
            Collider::aabb(Vec2::ZERO, Vec2::new(640.0, 32.0)),
            id, 
            vaabbit::physics::PhysicsClass::Solid
        );
        body.set_origin_as_center(true, false);
        body
    }

    fn update(&mut self, id: &ID<Self>, world: &mut World, vib: &mut Vibbit) where Self: Sized {
        let body = world.get_physics_body(&id).unwrap();
        let pos = body.pos() + Vec2::new(-320.0, 16.0);
        vib.draw_rect(pos, 640., 16., vibbit::Color::new(200,255,200,255));
    }
}

// The generic argument to Actor is the shared state objects receive.
// Typically this would be some object for rendering/input, but you could
// create a broader custom context object for "global" variables as well.
impl Actor<Vibbit> for Player {
    fn init_physicsbody(id:vaabbit::TypedID) -> vaabbit::physics::PhysicsBody where Self: Sized {
        vaabbit::physics::PhysicsBody::new(
            Vec2::new(0.0, 20.0),
            Some(vaabbit::shapes::Collider::AABB(AABB {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(10.0, 10.0),
            })), 
            id, 
            vaabbit::physics::PhysicsClass::Actor
        )
    }

    fn update(&mut self, id: &ID<Self>, world: &mut World, vib: &mut Vibbit) {
        // since the vibbit context is passed in, all our game objects can
        // get user input and draw to the screen
        let dir = vib.get_dir_arrows();
        if vib.key_press(vibbit::input::Key::Z) {
            self.vel = 4.0;
        }

        let dir = Vec2::new(dir.x * 2.0, self.vel);
        // move based on input, then apply gravity.
        let results = self.move_and_slide(&dir, world);

        // are we grounded?
        if results.touching_below && self.vel <= 0.0 {
            self.vel = 0.0;
        } else {
            // apply gravity
            self.vel -= 0.1;
            self.vel = self.vel.clamp(-3.0, 3.0);
        }

        // draw
        let body = world.get_physics_body(&id).unwrap();
        let color = if results.touching_below { vibbit::Color::new(200,255,200,255) } else { vibbit::Color::new(200,200,255,255) };
        vib.draw_rect(body.pos(), 10.0, 10.0, color);
    }
}

pub fn main() {
    let mut world = vaabbit::world::World::new();
    let mut vib = Vibbit::new(1280, 720, "context_state");
    vib.set_target_fps(60.0);

    let p_id = world.add_actor(Player {vel: 0.0});
    let g_id = world.add_actor(Ground {});

    for _ in 0..100 {
        world.add_actor(Coin {eaten: false});
        world.add_actor(Block {});
    }

    let mut camera = vibbit::Camera2D::new(vibbit::Vec2 { x: 0.0, y: 0.0 });
    camera.zoom.x = 2.0;
    camera.zoom.y = 2.0;

    loop {
        vib.clear_screen(vibbit::Color::new(0,0,0,255));
        let pos = world.get_pos(&p_id);
        camera.pos.x = pos.x * 2.0;
        camera.pos.y = pos.y * 2.0;

        vib.gfx_set_camera(camera);
        // Since our actor logic relies on the vibbit context, we need to pass it to the update method.
        world.update_systems(&mut vib);
        vib.end_frame();

        if vib.should_close() ||  pos.y < -1000.0 { break; }
    }
}