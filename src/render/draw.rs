use crate::{Actor, ID, TypedID, World, physics::PhysicsBody, world::Registry};



pub trait Draw<P: 'static> where Self: 'static, Self: Sized, Self: Actor<P> {
    fn draw_system(world: &mut World, ctx: &mut P, layer: i32) where Self: Sized {
        if Self::CFG_DOES_NOT_DRAW { return; }
        let entry = Registry::get_entry_mut::<Self>();

        let layer = world.renderer.get_layer::<Self, P>(layer);
        
        if let Some(actors) = layer {
            let actors: Vec<ID<Self>> = actors.iter().cloned().collect();

            for id in actors {
                let disabled = entry.is_enabled.get(id.index);
                if disabled.is_none() {
                    continue;
                }
                if !*&disabled.unwrap() {
                    continue;
                }
                let body = unsafe {std::ptr::read(world.physics.get_body(&id).unwrap_unchecked())};
                world.current_actor = Some(TypedID::from_id(id));
                let actor = entry.arena.get_mut(id.index).unwrap();
                actor.1.draw(&id, &body, world, ctx);
            }
        }
    }
}
impl<P: 'static, T: Actor<P>> Draw<P> for T {}