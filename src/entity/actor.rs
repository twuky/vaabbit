use std::any::TypeId;
use glam::Vec2;

use crate::{entity::{ID, TypedID}, world::{self, Registry, World}};

pub trait Actor<P: 'static>: Send + Sync + 'static {
    fn update(&mut self, id: &ID<Self>, world: &mut World, ctx: &mut P) where Self: Sized;

    fn update_system(world: &mut World, ctx: &mut P) where Self: Sized {
        let entities = Registry::get_entry::<Self>().entities.clone();

        for id in entities {
            let entry = Registry::get_mut(&id);

            if let Some(actor) = entry {
                actor.1.update(&actor.0, world, ctx);
                world.flush_events();
            } else {
                println!("update(): actor not found: {:?}", id.clone());
                println!("perhaps already in use?");
            }
        }
    }
    
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn pos<'a>(&self, id: &ID<Self>, world: &'a World) -> &'a Vec2 where Self: Sized {
        world.get_pos(id)
    }
    fn set_pos(&mut self, pos: Vec2, id: &ID<Self>, world: &'static mut World) where Self:Sized {
        world.set_pos(*id, pos);
    }
    fn move_by<'a>(&self, vector: &Vec2, id: &ID<Self>, world: &'a mut World) -> Vec2 where Self: Sized {
        world.move_by(*id, vector)
    }

    fn on_collision<'a>(&mut self, id: &ID<Self>, other: TypedID, world: &'a mut World) {

    }

}