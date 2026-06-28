use std::any::TypeId;
use glam::Vec2;
use smallvec::SmallVec;

use crate::{entity::{ID, TypedID}, physics::{PhysicsBody, PhysicsClass}, world::{Registry, World}};

pub trait Actor<P: 'static> where Self: 'static, Self: Sized {
    fn update(&mut self, id: &ID<Self>, world: &mut World, ctx: &mut P) where Self: Sized;

    #[inline]
    // Allows you to specify a default physics body for your actor when it is initialized
    fn init_physicsbody(id:TypedID) -> PhysicsBody where Self: Sized {
        PhysicsBody::new(Vec2::ZERO, None, id, crate::physics::PhysicsClass::Node)
    }

    #[inline]
    // System that updates the actor's state each frame, applying lifecycle hooks
    fn update_system(world: &mut World, ctx: &mut P) where Self: Sized {
        let registry_entry = &mut Registry::get_entry_mut::<Self>();
        // clone prevents flicker, ie objects spawning in the same frame
        let entities = registry_entry.entities.to_vec();

        for id in &entities {
            let entry = registry_entry.arena.get_mut(id.index);

            if let Some(actor) = entry {
                world.current_actor = Some(TypedID::from_id(actor.0));
                // late collision lifecycle hook         
                if let Some(collisions) = world.physics.get_late_collision_enter(id) {
                    let overlap_list = world.physics.get_overlap_list(id).clone();
                    world.physics.update_overlap_list(id, &collisions, &[]);
                    for collided in collisions {
                        // prevent double collision events if object already collided last frame
                        if overlap_list.contains(&collided) { continue; }
                        actor.1.on_collision(id, collided, world);
                    }
                }
                // late collision end lifecycle hook
                if let Some(collisions) = world.physics.get_late_collision_exit(id) {
                    world.physics.update_overlap_list(id, &[], &collisions);
                    for collided in collisions {
                        actor.1.on_collision_end(id, collided, world);
                    }
                }
                // regular update lifecycle hook
                actor.1.update(id, world, ctx);
                world.flush_events();
            } else {
                println!("update<{:?}>: actor not found: {:?}", id.type_name(), id.index);
                println!("perhaps already in use?");
            }
        }
    }
    
    #[inline(always)]
    // Returns the type id of the actor
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    #[inline(always)]
    // Returns the name of the actor
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    #[inline(always)]
    // Returns the position of the actor in the game world
    fn pos(&self, world: &World) -> Vec2 where Self: Sized {
        let id = &ID::<Self>::from_typed_id(world.current_actor.unwrap());
        world.get_pos(id)
    }
    #[inline(always)]
    // Sets the position of the actor in the game world
    fn set_pos(&mut self, pos: Vec2, world: &'static mut World) where Self:Sized {
        let id = &ID::<Self>::from_typed_id(world.current_actor.unwrap());
        world.set_pos(*id, pos);
    }
    #[inline(always)]
    // Moves the actor by the given vector, returning the new position
    fn move_by(&self, vector: &Vec2, world: &mut World) -> Vec2 where Self: Sized {
        let id = &ID::<Self>::from_typed_id(world.current_actor.unwrap());
        world.move_by(*id, vector)
    }

    #[inline(always)]
    fn move_and_slide(&self, vector: &Vec2, world: &mut World) -> MovementResults where Self: Sized {
        let id = &ID::<Self>::from_typed_id(world.current_actor.unwrap());
        world.move_and_slide(*id, vector)
    }

    // Lifecycle hook: called when the actor enters a collision with another actor
    fn on_collision(&mut self, _id: &ID<Self>, _other: TypedID, _world: &mut World) {
        // user override
    }

    // Lifecycle hook: called when the actor leaves a collision with another actor
    fn on_collision_end(&mut self, _id: &ID<Self>, _other: TypedID, _world: &mut World) {
        // user override
    }

    // Returns a list of all actors that are currently colliding with this actor
    fn get_colliding_bodies<'a>(&self, world: &'a World) -> &'a Vec<TypedID> {
        let id = &ID::<Self>::from_typed_id(world.current_actor.unwrap());
        world.physics.get_overlap_list(id)
    }

    #[inline(always)]
    // Moves the actor by the given vector, returning the new position
    fn emit<E: 'static>(&self, world: &mut World, event: E) {
        let id = &ID::<Self>::from_typed_id(world.current_actor.unwrap());
        world.emit(*id, event);
    }

    #[inline(always)]
    // Moves the actor by the given vector, returning the new position
    fn listen<E: 'static, O: 'static>(&self, world: &mut World, other: ID<O>, closure: impl Fn(&mut World, &E) + 'static) {
        let id = &ID::<Self>::from_typed_id(world.current_actor.unwrap());
        world.subscribe(other, *id, closure);
    }

}

pub struct MovementResults {
    pub final_pos: Vec2,

    pub touching_below: bool,
    pub touching_above: bool,
    pub touching_left: bool,
    pub touching_right: bool,
}

impl World {

    pub fn move_and_slide<T: 'static + Actor<P>, P: 'static>(&mut self, id: ID<T>, delta: &Vec2) -> MovementResults {
        let actor_body = self.physics.get_body(&id).unwrap();

        let start_point = actor_body.pos();
        let end_point = actor_body.pos() + actor_body.pos_remainder + *delta;
        let _remainder = end_point - end_point.trunc();
        // list of pixel positions to check collisions against
        let movement_steps = crate::math::bresenham_line_movement(start_point, start_point + *delta);

        // query area around the actor
        let mut query_bounds = actor_body.bounds();
        query_bounds.expand(delta.abs().max_element() + 2.0);

        let mut query_results = SmallVec::new();
        self.physics.query_against_id(&query_bounds, &mut query_results, id.into_typed_id());

        let currently_overlapping = self.physics.get_overlap_list(&id);
        // new objects we are overlapping with after movement
        let mut new_overlaps: Vec<TypedID> = Vec::with_capacity(2);
        // objects we are no longer overlapping with after movement
        let mut overlap_exits: Vec<TypedID> = Vec::with_capacity(2);

        let mut final_body = *actor_body;
        let mut test_body = *actor_body;
        let mut stopped: bool;
        // move actor along the line
        for movement in &movement_steps {
            // test new location
            let test_point = final_body.pos() + movement;
            test_body.set_pos(&test_point);

            // check for collisions
            stopped = false;    
            for other_body in &query_results {
                new_overlaps.push(other_body.id);
                if other_body.class == PhysicsClass::Solid && test_body.overlaps(other_body) {
                    stopped = true;
                }
            }

            // if not stopped by any collisions, update the final body
            if !stopped {
                final_body.set_pos(&test_point);
            }
        }

        // update overlap list
        for other_body in &query_results {
            if final_body.overlaps(other_body) {
                new_overlaps.push(other_body.id);
            } else {
                if new_overlaps.contains(&other_body.id) {
                    overlap_exits.push(other_body.id);
                    // remove from new overlaps
                    new_overlaps.retain(|i| *i != other_body.id);
                    continue;
                }
                if currently_overlapping.contains(&other_body.id) {
                    overlap_exits.push(other_body.id);
                }
            }
        }

        let mut result = MovementResults {
            final_pos: final_body.pos(),
            touching_below: false,
            touching_above: false,
            touching_left: false,
            touching_right: false,
        };

        let actor_bounds = final_body.bounds();
        test_body = final_body;
        for other_body in &query_results {
            if final_body.is_actor() && !other_body.is_solid() { continue; }

            let other_bounds = other_body.bounds();

            if actor_bounds.min.y >= other_bounds.max.y && (actor_bounds.min.y - 1.0) <= other_bounds.max.y {
                test_body.set_pos(&(result.final_pos + Vec2::new(0.0, -1.0)));
                if test_body.overlaps(other_body) {
                    result.touching_below = true;
                }
            }
            if actor_bounds.max.y < other_bounds.min.y && (actor_bounds.max.y + 1.0) > other_bounds.min.y {
                test_body.set_pos(&(result.final_pos + Vec2::new(0.0, 1.0)));
                if test_body.overlaps(other_body) {
                    result.touching_above = true;
                }
            }
            if actor_bounds.min.x > other_bounds.max.x && (actor_bounds.min.x - 1.0) < other_bounds.max.x {
                test_body.set_pos(&(result.final_pos + Vec2::new(-1.0, 0.0)));
                if test_body.overlaps(other_body) {
                    result.touching_left = true;
                }
            }
            if actor_bounds.max.x < other_bounds.min.x && (actor_bounds.max.x + 1.0) > other_bounds.min.x {
                test_body.set_pos(&(result.final_pos + Vec2::new(1.0, 0.0)));
                if test_body.overlaps(other_body) {
                    result.touching_right = true;
                }
            }
        }

        drop(query_results); // ends borrow of self

        // may not be necessary
        new_overlaps.dedup();
        overlap_exits.dedup();

        // lifecycle: collision end
        for other_id in &new_overlaps {
            let other_id = *other_id;
            self.with_world(&id, move |ett, world| {
                ett.on_collision(&id, other_id, world);
            });
            // defer collision lifecycle hook on other bodies
            self.physics.add_late_collision_enter(other_id, id.into_typed_id());
        }

        // lifecycle: collision end
        for other_id in &overlap_exits {
            let o_id = *other_id;
            self.with_world(&id, move |ett, world| {
                ett.on_collision_end(&id, o_id, world);
            });
            // defer collision lifecycle hook on other bodies
            self.physics.add_late_collision_exit(*other_id, id.into_typed_id());
        }

        // commit changes
        self.physics.update_overlap_list(&id, &new_overlaps, &overlap_exits);
        self.physics.update_body(&id, final_body);

        result
    }
}