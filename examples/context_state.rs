use vaabbit::{Actor, World, ID, Vec2};
use vibbit::Vibbit;

struct Player {

}

// The generic argument to Actor is the shared state objects receive.
// Typically this would be some object for rendering/input, but you could
// create a broader custom context object for "global" variables as well.
impl Actor<Vibbit> for Player {
    fn update(&mut self, id: &ID<Self>, world: &mut World, vib: &mut Vibbit) {
        // since the vibbit context is passed in, all our game objects can
        // get user input and draw to the screen
        let dir = vib.get_dir_wasd();

        let pos = self.move_by(&Vec2::new(dir.x, dir.y), &id, world);
        vib.draw_rect(vibbit::Vec2::new(pos.x, pos.y), 10.0, 10.0, vibbit::Color::new(255,255,255,255));
    }
}

pub fn main() {
    let mut world = vaabbit::world::World::new();
    let mut vib = Vibbit::new(640, 480, "context_state");

    let p_id = world.add_actor(Player {});

    loop {
        vib.clear_screen(vibbit::Color::new(0,0,0,255));
        // Since our actor logic relies on the vibbit context, we need to pass it to the update method.
        world.update_systems(&mut vib);
        vib.end_frame();
    }
}