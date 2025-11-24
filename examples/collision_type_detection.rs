use std::time::SystemTime;

use macroquad::{color, miniquad::window::set_window_size, prelude::*};
use vaabbit::{Actor, ID, Signal, TypedID, World};


type _CollectEvent = Signal<()>;

struct Player {
    vel: vaabbit::Vec2,
}

struct Coin {
    eaten: bool,
}

struct Block {

}

impl Actor for Block {
    fn update(&mut self, _id: &ID<Self>, world: &mut World) {
        let pos = self.pos(_id, world);
        draw_rectangle(pos.x, pos.y, 32.0, 32.0, color::WHITE);
    }
}

impl Actor for Coin {
    fn update(&mut self, _id: &ID<Self>, world: &mut World) {
        let pos = self.pos(_id, world);

        if self.eaten {return}
        draw_rectangle(pos.x, pos.y, 32.0, 32.0, color::YELLOW);
    }
}

impl Actor for Player {

    fn update(&mut self, _id: &ID<Self>, _world: &mut World) {

        let mut vel = vaabbit::Vec2::new(0.0, 0.0);

        for key in get_keys_down() {
            match key {
                KeyCode::W => vel.y -= 1.0,
                KeyCode::S => vel.y += 1.0,
                KeyCode::A => vel.x -= 1.0,
                KeyCode::D => vel.x += 1.0,
                _ => {}
            }
        }
        self.vel = vel;

        let pos = self.move_by(&(vel * 2.0), _id, _world);

        //println!("{:?}", self.pos);
        draw_rectangle(pos.x, pos.y, 32.0, 32.0, color::RED);
    }

    fn on_collision(&mut self, id: &ID<Self>, other: TypedID, world: &mut World) {
        let t_coin = vaabbit::type_of::<Coin>();
        let t_block = vaabbit::type_of::<Block>();
        match other.type_id {
            idx if idx == t_coin => {

                let c_id: ID<Coin> = other.into();

                if let Some(coin) = world.get(&c_id) {
                    if coin.eaten {return}
                }
                
                world.with(&ID::<Coin>::from(other), |coin: &mut Coin| {
                    coin.eaten = true;
                    println!("im eaten!");
                });
                
                world.with_world(&other.into(), move |_coin: &mut Coin, world| {
                    let random_pos = vaabbit::Vec2::new(rand::gen_range(0.0, 640.0), rand::gen_range(0.0, 480.0));
                    let c_id = world.add_actor(Coin {eaten: false});
                    println!("updating new coin pos: {:?}", c_id);
                    

                    world.with_world(&c_id, move |_coin: &mut Coin, world| {
                        world.set_pos(c_id, random_pos);
                        println!("--x moving coin to {:?}", c_id);
                    });
                });
                
            }
            idx if idx == t_block => {
                println!("player collision with {:?}", other);

                let p_id = id.clone();
                world.with_world(id, move |p, world| {
                    world.move_by(p_id, &(p.vel * -2.0));
                });
            }
            _ => {}
        }
    }
} 

#[macroquad::main("coins")]
async fn main() {
    set_window_size(640, 480);
    rand::srand(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64);

    let mut world = vaabbit::world::World::new();
    let _p_id = world.add_actor(Player {vel: vaabbit::Vec2::ZERO});

    for _i in 0..10 {
        let random_pos = vaabbit::Vec2::new(rand::gen_range(0.0, 640.0), rand::gen_range(0.0, 480.0));
        let c_id = world.add_actor(Coin {eaten: false});
        world.set_pos(c_id, random_pos);
    }

    for _i in 0..10 {
        let random_pos = vaabbit::Vec2::new(rand::gen_range(0.0, 640.0), rand::gen_range(0.0, 480.0));
        let c_id = world.add_actor(Block {});
        world.set_pos(c_id, random_pos);
    }


    loop {
        world.update_systems();
        next_frame().await;
    }
}