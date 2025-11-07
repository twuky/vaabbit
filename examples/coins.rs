use std::time::{Instant, SystemTime};

use macroquad::{color, miniquad::window::set_window_size, prelude::*};
use vaabbit::{events::Event, world::{self, ID, actor::Actor}};


type CollectEvent = Event<()>;

struct Player {
    pos: Vec2,
}

struct Coin {
    pos: Vec2,
    eaten: bool,
    // typed "ID" setup for easy relationships between actors
    player: Option<ID<Player>>
}

impl Actor for Coin {
    fn update(&mut self, _id: &ID<Self>, world: &mut world::World) {
        if self.eaten {return}

        // we have a typed 'reference' to the player we can access
        let plyr = world.get(&self.player.unwrap());

        if self.pos.distance(plyr.pos) < 10.0 {
            self.eaten = true;

            // we can mutate the player by qeueing an action with it
            // this will run as soon as this update finishes
            world.with(&self.player.unwrap(), |player| {
                player.pos = Vec2::new(rand::gen_range(0.0, 640.0), rand::gen_range(0.0, 480.0));
            });

            world.emit(self.player.unwrap(), () as CollectEvent);
        }

        draw_rectangle(self.pos.x, self.pos.y, 5.0, 5.0, color::YELLOW);
    }

}

impl Actor for Player {

    fn update(&mut self, _id: &ID<Self>, _world: &mut world::World) {

        let mut vel = Vec2::new(0.0, 0.0); 

        for key in get_keys_down() {
            match key {
                KeyCode::W => vel.y -= 1.0,
                KeyCode::S => vel.y += 1.0,
                KeyCode::A => vel.x -= 1.0,
                KeyCode::D => vel.x += 1.0,
                _ => {}
            }
        }

        self.pos += vel * 2.0;

        //println!("{:?}", self.pos);
        draw_rectangle(self.pos.x, self.pos.y, 10.0, 10.0, color::RED);
    }
}

#[macroquad::main("coins")]
async fn main() {
    set_window_size(640, 480);
    rand::srand(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64);

    let mut world = vaabbit::world::World::new();

    let p_id = world.add_actor(Player {pos: Vec2::new(20.0, 20.0)});

    world.subscribe(p_id, p_id, |event: &CollectEvent| {
        println!("player collected a coin!");
    });

    for _i in 0..10 {
        let random_pos = Vec2::new(rand::gen_range(0.0, 640.0), rand::gen_range(0.0, 480.0));
        world.add_actor(Coin {pos: random_pos, eaten: false, player: Some(p_id.clone())});
    }


    loop {
        world.update_systems();
        next_frame().await;
    }
}