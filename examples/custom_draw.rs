use glam::Vec2;
use macroquad::input::KeyCode::S;
use vaabbit::{Actor, Draw, ID, TypedID, World, physics::{PhysicsBody, PhysicsClass}, shapes::Collider};
use vibbit::{Color, Vibbit};


struct Rect {
  pos: glam::Vec2,
  color: vibbit::Color,
  vel: glam::Vec2,
}

impl Actor<Vibbit> for Rect {
    fn init_physicsbody(id:TypedID) -> PhysicsBody where Self: Sized {
        PhysicsBody::new(Vec2::ZERO, Collider::aabb(Vec2::ZERO, Vec2 { x: 32., y: 32. }), id, PhysicsClass::Solid)
    }

    fn update(&mut self, _id: &ID<Self>, world: &mut World, _ctx: &mut Vibbit) where Self: Sized {
        let pos = self.move_by(&{self.vel}, world);
        
        if pos.x < -1280.0 - 32.0 || pos.x > 1280.0 - 32.0 {
            self.vel.x *= -1.0;
        }
        if pos.y < -720.0 - 32.0 || pos.y > 720.0 - 32.0 {
            self.vel.y *= -1.0;
        }

        self.set_z(-pos.y as i32, world);
    }

    fn draw(&mut self, _id: &ID<Self>, body: &PhysicsBody, world: &mut World, vib: &mut Vibbit) where Self: Sized {
        let c = self.color.rgba_f32();
        let lighten = Color::from_normalized(c.0 * 0.5, c.1 * 0.5, c.2 * 0.5, c.3);

        
        let pos = body.pos();
        vib.draw_rect(pos, 32.0, 24.0, lighten);
        vib.draw_rect(pos + Vec2::new(0.0, 24.0), 32.0, 32.0, self.color);
    }
}

struct Player {
}

impl Actor<Vibbit> for Player {
    fn init_physicsbody(id:TypedID) -> PhysicsBody where Self: Sized {
        PhysicsBody::new(Vec2::ZERO, Collider::aabb(Vec2::ZERO, Vec2 { x: 16., y: 6. }), id, PhysicsClass::Actor)
    }

    fn update(&mut self, _id: &ID<Self>, world: &mut World, vib: &mut Vibbit) where Self: Sized {
        let movement = vib.get_dir_arrows() * 2.0;
        self.move_and_slide(&movement, world);

        let pos = self.pos(world);
        self.set_z(-pos.y as i32, world);
    }

    fn draw(&mut self, _id: &ID<Self>, body: &PhysicsBody, world: &mut World, vib: &mut Vibbit) where Self: Sized {
        vib.draw_rect(body.pos(), 18.0, 18.0, vibbit::RED);
    }
}

fn main() {
    let mut vib = Vibbit::new(1280, 720, "custom z order");
    let mut world = vaabbit::world::World::new();
    vib.cfg.window_show_fps(true);

    let _player = world.add_actor(Player{});

    for _ in 0..100 {
      let pos = vib.rand_vec2(-1280.0..1280.0, -720.0..720.0);
      let color = Color::from_normalized(vib.rand_f32(), vib.rand_f32(), vib.rand_f32(), 1.0);
      let vel = vib.rand_dir();
      let rect = world.add_actor(Rect { pos, color, vel });
      world.set_pos(rect, pos);
    }

    for _ in 0..100 {
      let pos = vib.rand_vec2(-1280.0..1280.0, -720.0..720.0);
      let color = Color::from_normalized(vib.rand_f32(), vib.rand_f32(), vib.rand_f32(), 1.0);
      let vel = Vec2 { x: 0., y: 0. };
      let rect = world.add_actor(Rect { pos, color, vel });
      world.set_pos(rect, pos);
    }

    while !vib.should_close() {
      vib.clear_screen(vibbit::DARKGREY);
      world.update_systems(&mut vib);
      world.draw_systems(&mut vib);
      vib.end_frame();
    }
}