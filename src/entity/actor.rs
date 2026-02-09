use std::any::TypeId;
use glam::Vec2;

use crate::{entity::{ID, TypedID}, world::{Registry, World}};

pub trait Actor<P: 'static> where Self: 'static, Self: Sized {
    fn update(&mut self, id: &ID<Self>, world: &mut World, ctx: &mut P) where Self: Sized;

    #[inline]
    fn update_system(world: &mut World, ctx: &mut P) where Self: Sized {
        let entities = Registry::get_entry::<Self>().entities.clone();

        for id in entities {
            let entry = Registry::get_mut(&id);

            if let Some(actor) = entry {
                // late collision lifecycle hook
                if let Some(collisions) = world.physics.get_late_collision_enter(&id) {
                    world.physics.update_overlap_list(&id, collisions.to_vec(), Vec::new());
                    for collided in collisions {
                        actor.1.on_collision(&id, collided, world);
                    }
                }
                if let Some(collisions) = world.physics.get_late_collision_exit(&id) {
                    world.physics.update_overlap_list(&id, Vec::new(), collisions.to_vec());
                    for collided in collisions {
                        actor.1.on_collision_end(&id, collided, world);
                    }
                }
                // regular update lifecycle hook
                actor.1.update(&actor.0, world, ctx);
                world.flush_events();
            } else {
                println!("update(): actor not found: {:?}", id.clone());
                println!("perhaps already in use?");
            }
        }
    }
    
    #[inline(always)]
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    #[inline(always)]
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    #[inline(always)]
    fn pos<'a>(&self, id: &ID<Self>, world: &'a World) -> Vec2 where Self: Sized {
        world.get_pos(id)
    }
    #[inline(always)]
    fn set_pos(&mut self, pos: Vec2, id: &ID<Self>, world: &'static mut World) where Self:Sized {
        world.set_pos(*id, pos);
    }
    #[inline(always)]
    fn move_by<'a>(&self, vector: &Vec2, id: &ID<Self>, world: &'a mut World) -> Vec2 where Self: Sized {
        world.move_by(*id, vector)
    }

    fn on_collision<'a>(&mut self, id: &ID<Self>, other: TypedID, world: &'a mut World) {

    }

    fn on_collision_end<'a>(&mut self, id: &ID<Self>, other: TypedID, world: &'a mut World) {

    }

    fn get_colliding_bodies<'a>(&self, id: &ID<Self>, world: &'a World) -> &'a Vec<TypedID> {
        world.physics.get_overlap_list(id)
    }

}