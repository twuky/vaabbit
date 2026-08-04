use macroquad::prelude::rand;
use vaabbit::{World, Actor, ID};
use vibbit::{Vibbit, Color};

struct Bunny {
    vel: glam::Vec2,
    color: Color,
    pos: glam::Vec2,
}

impl Bunny {
    fn new() -> Self {
        Self {
            pos: glam::Vec2::ZERO,
            vel: glam::Vec2::new(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)),
            color: Color::from_normalized(rand::gen_range(0.0, 1.0), rand::gen_range(0.0, 1.0), rand::gen_range(0.0, 1.0), 1.0),
        }
    }
}

impl Actor<()> for Bunny {
    #[inline(always)]
    fn update(&mut self, _id: &ID<Self>, _world: &mut World, ctx: &mut ()) {
        let vel = self.vel;
        self.pos += vel;
        
        if self.pos.x < 0.0 {
            self.vel.x *= -1.0;
        } else if self.pos.x > 640.0 {
            self.vel.x *= -1.0;
        }

        if self.pos.y < 0.0 {
            self.vel.y *= -1.0;
        } else if self.pos.y > 480.0 {
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

        world.update_systems(&mut ());

        if bunnies % 1000 == 0 { println!("bunnies: {}, ft: {:?}", bunnies, world.logic_update); }

        vib.clear_screen(Color::new(0,0,0,255));

        for (_id, bunny) in world.query::<Bunny>() {
            let pos = bunny.pos;
            vib.draw_texture(tex, offset + pos, bunny.color);
        }

        vib.end_frame();
    }
}