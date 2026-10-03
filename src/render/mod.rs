mod draw;

use anymap::AnyMap;
pub use draw::Draw;
use rustc_hash::FxHashSet;
use vibarena::KeySet;

use crate::{Actor, ID, World, collections::SortkeyVec};

pub(crate) struct Renderer {
    pub(crate) layers: SortkeyVec<i32>,
    pub(crate) type_layers: AnyMap,

    //draw_systems: Vec<fn(&mut World, &mut P, i32)>,
    pub(crate) draw_methods: AnyMap,
}

impl Default for Renderer {
    fn default() -> Self {
        let mut layers = SortkeyVec::new();
        layers.insert(0, 0);
        
        Self {
            layers,
            type_layers: AnyMap::new(),
            draw_methods: AnyMap::new(),
        }
    }
}

impl Renderer {

    pub(crate) fn register_type<T: Actor<P> + 'static, P: 'static>(&mut self) {
        if !self.type_layers.contains::<SortkeyVec<FxHashSet<ID<T>>>>() {
            let mut layer = SortkeyVec::<FxHashSet<ID<T>>>::new();
            let hash_set = FxHashSet::<ID<T>>::default();
            layer.insert(0, hash_set);

            self.type_layers.insert(layer);

            let draw_func = T::draw_system;
            
            if self.draw_methods.contains::<Vec<fn(&mut World, &mut P, i32)>>() {
                self.draw_methods.get_mut::<Vec<fn(&mut World, &mut P, i32)>>().unwrap().push(draw_func);
            } else {
                let mut draw_methods: Vec<fn(&mut World, &mut P, i32)> = Vec::with_capacity(32);
                draw_methods.push(draw_func);
                self.draw_methods.insert(draw_methods);
            }
        }
    }

    pub(crate) fn add_layer<T: Actor<P> + 'static, P: 'static>(&mut self, layer: i32) {
        self.layers.insert(layer, layer);
        self.type_layers.get_mut::<SortkeyVec<FxHashSet<ID<T>>>>().unwrap().insert(layer, FxHashSet::default());
    }

    pub(crate) fn get_draw_methods<P: 'static>(&mut self) -> Option<&Vec<fn(&mut World, &mut P, i32)>> {
        self.draw_methods.get::<Vec<fn(&mut World, &mut P, i32)>>()
    }

    #[inline(always)]
    pub(crate) fn get_layer<T: Actor<P> + 'static, P: 'static>(&mut self, layer: i32) -> Option<&mut FxHashSet<ID<T>>> {
        self.type_layers.get_mut::<SortkeyVec<FxHashSet<ID<T>>>>().unwrap().get_mut(layer)
    }

    pub(crate) fn add_actor<T: Actor<P> + 'static, P: 'static>(&mut self, id: ID<T>) {
        let set = self.get_layer::<T, P>(0).unwrap();
        set.insert(id);
    }

    pub(crate) fn update_z<T: Actor<P> + 'static, P: 'static>(&mut self, to_layer: i32, id: ID<T>, from_layer: i32) {
        self.layers.insert(to_layer, to_layer);
        self.get_layer::<T, P>(from_layer).unwrap().remove(&id);
        let layer = self.get_layer::<T, P>(to_layer);
        if let Some(layer) = layer {
            layer.insert(id);
        } else {
            self.add_layer::<T, P>(to_layer);
            self.get_layer::<T, P>(to_layer).unwrap().insert(id);
        }
    }

    pub(crate) fn remove_actor<T: Actor<P> + 'static, P: 'static>(&mut self, id: ID<T>, layer: i32) {
        self.get_layer::<T, P>(layer).unwrap().remove(&id);
    }

    pub(crate) fn remove_actor_unknown<T: Actor<P> + 'static, P: 'static>(&mut self, id: ID<T>) {
        let layers = self.type_layers.get_mut::<SortkeyVec<FxHashSet<ID<T>>>>().unwrap();
        for layer in layers.iter_mut_values() {
            layer.remove(&id);
        }
    }
    
}