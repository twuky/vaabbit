use std::any::TypeId;
use glam::Vec2;

use crate::{shapes::CollisionShape, world::{World, ID}};

pub trait Actor: Send + Sync + 'static {
    fn update(&mut self, id: &ID<Self>, world: &mut World) where Self: Sized;

    fn update_system(world: &mut World) where Self: Sized {
        let entities = world.registry.get_entry::<Self>().entities.clone();

        for id in entities {
            unsafe {
                let mut actor = std::ptr::read(world.get_mut(&id));
                actor.update(&id, world);
                std::ptr::write(world.get_mut(&id), actor);
            }
            world.flush_events();
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
        world.set_pos(id, pos);
    }
    fn move_by<'a>(&self, vector: &Vec2, id: &ID<Self>, world: &'a mut World) -> Vec2 where Self: Sized {
        world.move_by(id, vector)
    }

    fn on_collision(&mut self) {
        
    }

}