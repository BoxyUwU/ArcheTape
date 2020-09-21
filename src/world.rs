use super::anymap::{AnyMap, AnyMapBorrow, AnyMapBorrowMut};
use super::archetype_iter::{Query, QueryInfos};
use super::bundle::Bundle;
use super::entities::{Entities, Entity};
use super::lifetime_anymap::{LifetimeAnyMap, LifetimeAnyMapBorrow, LifetimeAnyMapBorrowMut};
use super::untyped_vec::UntypedVec;
use std::any::TypeId;
use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;

pub struct Archetype {
    pub lookup: HashMap<TypeId, usize, crate::anymap::TypeIdHasherBuilder>,
    pub type_ids: Vec<TypeId>,
    pub entities: Vec<Entity>,
    pub component_storages: Vec<RwLock<UntypedVec>>,
}

impl Archetype {
    pub fn new<T: Bundle>() -> Archetype {
        T::new_archetype()
    }

    pub fn add<T: Bundle>(&mut self, components: T, entity: Entity) {
        components.add_to_archetype(self, entity);
    }
}

pub struct World {
    pub archetypes: Vec<Archetype>,
    owned_resources: AnyMap,
    entities: Entities,
    cache: Vec<(Vec<TypeId>, usize)>,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            owned_resources: AnyMap::new(),
            entities: Entities::new(),
            cache: Vec::with_capacity(8),
        }
    }

    pub fn query<T: QueryInfos>(&self) -> Query<T> {
        Query::<T>::new(self)
    }

    pub fn find_archetype<T: Bundle>(&mut self, type_ids: &[TypeId]) -> Option<usize> {
        debug_assert!(T::type_ids() == type_ids);

        for (cached_type_id, archetype) in self.cache.iter() {
            if *cached_type_id == type_ids {
                return Some(*archetype);
            }
        }

        let position = self
            .archetypes
            .iter()
            .position(|archetype| archetype.type_ids == type_ids);

        if let Some(position) = position {
            if self.cache.len() > 8 {
                self.cache.pop();
            }
            self.cache.insert(0, (Vec::from(type_ids), position));
        }

        position
    }

    pub fn find_archetype_or_insert<T: Bundle>(&mut self, type_ids: &[TypeId]) -> usize {
        debug_assert!(T::type_ids() == type_ids);

        self.find_archetype::<T>(type_ids).unwrap_or_else(|| {
            self.cache.clear();
            self.archetypes.push(T::new_archetype());
            self.archetypes.len() - 1
        })
    }

    pub fn query_archetypes<T: QueryInfos>(&self) -> impl Iterator<Item = usize> + '_ {
        self.archetypes
            .iter()
            .enumerate()
            .filter(|(_, archetype)| {
                T::type_ids().iter().all(|id| {
                    if let Some(id) = id {
                        archetype.type_ids.contains(id)
                    } else {
                        // If id is none then the id should be skipped
                        true
                    }
                })
            })
            .map(|(n, _)| n)
    }

    pub fn spawn<T: Bundle>(&mut self, bundle: T) {
        let entity = self.entities.spawn();

        let type_ids = T::type_ids();
        let archetype_idx = self.find_archetype_or_insert::<T>(&type_ids);
        self.archetypes
            .get_mut(archetype_idx)
            .unwrap()
            .add(bundle, entity);
    }

    pub fn run(&mut self) -> RunWorldContext {
        RunWorldContext {
            world: self,
            temp_resources: LifetimeAnyMap::new(),
        }
    }
}

pub struct RunWorldContext<'run> {
    world: &'run mut World,
    temp_resources: LifetimeAnyMap<'run>,
}

impl<'run> RunWorldContext<'run> {
    pub fn insert_owned_resource<T: 'static>(&mut self, data: T) {
        self.world.owned_resources.insert(data);
    }

    pub fn get_owned_resource<'a, T: 'static>(
        &'a self,
    ) -> Result<AnyMapBorrow<'a, T>, Box<dyn Error + 'a>> {
        self.world.owned_resources.get()
    }

    pub fn get_owned_resource_mut<'a, T: 'static>(
        &'a self,
    ) -> Result<AnyMapBorrowMut<'a, T>, Box<dyn Error + 'a>> {
        self.world.owned_resources.get_mut()
    }

    pub fn insert_temp_resource<'a, T: 'static>(&'a mut self, resource: &'run mut T) {
        self.temp_resources.insert(resource);
    }

    pub fn get_temp_resource<'a, T: 'static>(
        &'a self,
    ) -> Result<LifetimeAnyMapBorrow<'a, T>, Box<dyn Error + 'a>> {
        self.temp_resources.get()
    }

    pub fn get_temp_resource_mut<'a, T: 'static>(
        &'a self,
    ) -> Result<LifetimeAnyMapBorrowMut<'a, T>, Box<dyn Error + 'a>> {
        self.temp_resources.get_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn borrow_temp_resource_mut() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;
        run_ctx.insert_temp_resource(&mut foo);

        let mut foo_borrow = run_ctx.get_temp_resource_mut::<u32>().unwrap();
        *foo_borrow += 1;
    }

    #[test]
    pub fn borrow_temp_resource() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;
        run_ctx.insert_temp_resource(&mut foo);

        run_ctx.get_temp_resource::<u32>().unwrap();
    }

    #[test]
    pub fn borrow_temp_resource_mut_twice() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;

        run_ctx.insert_temp_resource(&mut foo);

        let _borrow_1 = run_ctx.get_temp_resource_mut::<u32>().unwrap();
        assert!(run_ctx.get_temp_resource_mut::<u32>().is_err());
    }

    #[test]
    pub fn borrow_temp_resource_twice() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;

        run_ctx.insert_temp_resource(&mut foo);

        let _1 = run_ctx.get_temp_resource::<u32>().unwrap();
        let _2 = run_ctx.get_temp_resource::<u32>().unwrap();
    }

    #[test]
    pub fn borrow_temp_resource_shared_and_mut() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;

        run_ctx.insert_temp_resource(&mut foo);

        let _borrow_1 = run_ctx.get_temp_resource::<u32>().unwrap();
        assert!(run_ctx.get_temp_resource_mut::<u32>().is_err());
    }

    #[test]
    pub fn multi_borrow() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;

        run_ctx.insert_temp_resource(&mut foo);

        run_ctx.get_temp_resource_mut::<u32>().unwrap();
        run_ctx.get_temp_resource_mut::<u32>().unwrap();
        run_ctx.get_temp_resource_mut::<u32>().unwrap();
        run_ctx.get_temp_resource_mut::<u32>().unwrap();
    }
}
