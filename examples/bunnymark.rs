use macroquad::prelude::rand;
use vaabbit::{World, Actor, ID};
use vibbit::{Vibbit, Color};

struct Bunny {
    vel: glam::Vec2,
    color: Color,
}

impl Bunny {
    fn new() -> Self {
        Self {
            vel: glam::Vec2::new(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)),
            color: Color::from_normalized(rand::gen_range(0.0, 1.0), rand::gen_range(0.0, 1.0), rand::gen_range(0.0, 1.0), 1.0),
        }
    }
}

impl Actor for Bunny {
    fn update(&mut self, _id: &ID<Self>, _world: &mut World) {
        let pos = self.move_by(&self.vel, _id, _world);
        
        if pos.x < 0.0 {
            self.vel.x *= -1.0;
        } else if pos.x > 640.0 {
            self.vel.x *= -1.0;
        }

        if pos.y < 0.0 {
            self.vel.y *= -1.0;
        } else if pos.y > 480.0 {
            self.vel.y *= -1.0;
        }
    }
}


fn main() {
    let mut vib = Vibbit::new(640, 480, "bunnymark");
    let mut world = vaabbit::world::World::new();

    let tex = vib.load_texture("examples/assets/wabbit_alpha.png");

    let mut bunnies = 0;

    let offset = vibbit::Vec2::new(-320.0, -240.0);

    while !vib.should_close() {
        if vib.get_fps() > 58 && bunnies < 500_000{
            for _ in 0..100 {
                world.add_actor(Bunny::new());
                bunnies += 1;
            }
        }

        world.update_systems();

        if bunnies % 1000 == 0 { println!("bunnies: {}, ft: {:?}", bunnies, world.logic_update); }

        vib.clear_screen(Color::new(0,0,0,255));

        for (_id, bunny) in world.query_id::<Bunny>() {
            let pos = world.get_pos(_id);
            vib.draw_texture(tex, offset + vibbit::Vec2::new(pos.x, pos.y), bunny.color);
        }

        vib.end_frame();
    }
}