use super::archetype_iter::{Query, QueryInfos};
use super::entities::{Entities, Entity};
use super::sparse_array::SparseArray;
use super::untyped_vec::UntypedVec;
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::RwLock;

const PAGE_SIZE: usize = 256;

pub struct Archetype {
    pub sparse: SparseArray<usize, PAGE_SIZE>,
    pub type_ids: Vec<TypeId>,
    pub lookup: HashMap<TypeId, usize, crate::TypeIdHasherBuilder>,

    pub entities: Vec<Entity>,
    pub component_storages: Vec<UnsafeCell<UntypedVec>>,
}

use crate::entity_builder::TupleEntry;

impl Archetype {
    pub fn new<T: TupleEntry>(entity: Entity, tuple: T) -> Self {
        fn new_recursive<T: TupleEntry>(
            entity: Entity,
            tuple: T,
            mut type_ids: Vec<TypeId>,
            mut untyped_vecs: Vec<UnsafeCell<UntypedVec>>,
            mut lookup: HashMap<TypeId, usize, crate::TypeIdHasherBuilder>,
        ) -> Archetype {
            if let Some((left, right)) = tuple.next() {
                type_ids.push(TypeId::of::<T::Left>());
                let mut untyped_vec = UntypedVec::new::<T::Left>();
                untyped_vec.push(left);
                untyped_vecs.push(UnsafeCell::new(untyped_vec));
                lookup.insert(TypeId::of::<T::Left>(), untyped_vecs.len() - 1);
                return new_recursive(entity, right, type_ids, untyped_vecs, lookup);
            }

            let mut sparse = SparseArray::new();
            sparse.insert(entity.index() as usize, 0);

            // We're at the bottom of the tuple
            Archetype {
                sparse,
                type_ids,
                lookup,

                entities: vec![entity],
                component_storages: untyped_vecs,
            }
        }

        new_recursive(
            entity,
            tuple,
            Vec::new(),
            Vec::new(),
            HashMap::with_hasher(crate::TypeIdHasherBuilder()),
        )
    }

    pub fn spawn<T: TupleEntry>(&mut self, entity: Entity, tuple: T) {
        self.entities.push(entity);
        self.sparse
            .insert(entity.index() as usize, self.entities.len() - 1);

        fn insert_recursive<T: TupleEntry>(archetype: &mut Archetype, entity: Entity, tuple: T) {
            if let Some((left, right)) = tuple.next() {
                let storage_idx = archetype.lookup[&TypeId::of::<T::Left>()];
                let storage = archetype.component_storages[storage_idx].get_mut();
                storage.push(left);

                insert_recursive(archetype, entity, right);
            }
        }

        insert_recursive(self, entity, tuple);
    }

    pub fn from_archetype(from: &mut Archetype) -> Archetype {
        Archetype {
            sparse: SparseArray::new(),

            lookup: from.lookup.clone(),
            type_ids: from.type_ids.clone(),

            entities: Vec::new(),
            component_storages: {
                let mut storages = Vec::with_capacity(from.component_storages.len());
                for storage in from.component_storages.iter_mut() {
                    let untyped_vec = UntypedVec::new_from_untyped_vec(storage.get_mut());
                    let cell = UnsafeCell::new(untyped_vec);
                    storages.push(cell);
                }
                storages
            },
        }
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if let Some(idx) = self.sparse.remove(entity.uindex()) {
            self.entities.remove(idx);
            for lock in self.component_storages.iter_mut() {
                let storage = lock.get_mut();
                storage.swap_remove(idx);
            }
        }
        false
    }
}

pub struct World {
    pub(crate) archetypes: Vec<Archetype>,
    entities: Entities,
    cache: Vec<(Vec<TypeId>, usize)>,

    entity_to_archetype: SparseArray<usize, PAGE_SIZE>,

    pub(crate) lock_lookup: HashMap<TypeId, usize>,
    pub(crate) locks: Vec<RwLock<()>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            entities: Entities::new(),
            cache: Vec::with_capacity(8),
            entity_to_archetype: SparseArray::with_capacity(32),

            lock_lookup: HashMap::new(),
            locks: Vec::new(),
        }
    }

    pub fn find_archetype_from_entity(&self, entity: Entity) -> Option<usize> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        self.entity_to_archetype.get(entity.uindex()).copied()
    }

    pub fn add_entity_to_sparse_array(&mut self, entity: Entity, archetype: usize) {
        if self.entities.is_alive(entity) {
            self.entity_to_archetype.insert(entity.uindex(), archetype);
        }
    }

    pub fn remove_entity_from_sparse_array(&mut self, entity: Entity) {
        if self.entities.is_alive(entity) {
            self.entity_to_archetype.remove(entity.uindex());
        }
    }

    pub fn query<T: QueryInfos>(&self) -> Query<T> {
        Query::<T>::new(self)
    }

    pub fn find_archetype(&mut self, type_ids: &[TypeId]) -> Option<usize> {
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

    pub fn query_archetypes<T: QueryInfos>(&self) -> impl Iterator<Item = usize> + '_ {
        let type_ids = T::type_ids();
        self.archetypes
            .iter()
            .enumerate()
            .filter(move |(_, archetype)| {
                type_ids.iter().all(|id| {
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

    pub fn spawn(&mut self) -> crate::entity_builder::EntityBuilder {
        let entity = self.entities.spawn();

        crate::entity_builder::EntityBuilder {
            entity,
            world: self,
            type_ids: Vec::new(),
            components: (),
        }
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        let archetype = self.find_archetype_from_entity(entity).unwrap();
        self.archetypes[archetype].despawn(entity);
        self.remove_entity_from_sparse_array(entity);
        self.entities.despawn(entity);
        true
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        if !self.entities.is_alive(entity) {
            return;
        }

        let current_archetype = self.find_archetype_from_entity(entity).unwrap();
        let type_ids = self.archetypes[current_archetype].type_ids.clone();
        let remove_index = type_ids
            .iter()
            .position(|&id| id == TypeId::of::<T>())
            .unwrap();

        let mut target_type_ids = type_ids.clone();
        target_type_ids.swap_remove(remove_index);

        let target_archetype_idx = self
            .find_archetype(&target_type_ids)
            .or_else(|| {
                let mut archetype =
                    Archetype::from_archetype(&mut self.archetypes[current_archetype]);

                // Remove type_id from archetype.type_ids
                archetype.type_ids.swap_remove(remove_index);

                // Remove type_id entry from archetype.lookup
                let untyped_vec_idx = archetype.lookup.remove_entry(&TypeId::of::<T>()).unwrap().1;

                // Remove untyped_vec from archetype.component_storages
                archetype.component_storages.remove(untyped_vec_idx);

                // Decrement indexes returned from archetype.lookup that are > the index of the removed untyped_vec
                for (_, idx) in archetype.lookup.iter_mut() {
                    if *idx > untyped_vec_idx {
                        *idx -= 1;
                    }
                }

                self.archetypes.push(archetype);
                Some(self.archetypes.len() - 1)
            })
            .unwrap();

        let (current_archetype, target_archetype) = {
            if current_archetype < target_archetype_idx {
                let (left, right) = self.archetypes.split_at_mut(target_archetype_idx);
                (
                    left.get_mut(current_archetype).unwrap(),
                    right.first_mut().unwrap(),
                )
            } else if current_archetype > target_archetype_idx {
                let (left, right) = self.archetypes.split_at_mut(current_archetype);
                (
                    right.first_mut().unwrap(),
                    left.get_mut(target_archetype_idx).unwrap(),
                )
            } else {
                panic!()
            }
        };

        let mut target_archetype_num_components = None;
        let entity_idx = *current_archetype.sparse.get(entity.uindex()).unwrap();
        for lock in current_archetype.component_storages.iter_mut() {
            let current_storage = lock.get_mut();
            let type_info = current_storage.get_type_info();

            match target_archetype.lookup.get(&type_info.id) {
                Some(storage_idx) => {
                    let target_storage = target_archetype
                        .component_storages
                        .get_mut(*storage_idx)
                        .unwrap()
                        .get_mut();
                    current_storage.swap_move_element_to_other_vec(target_storage, entity_idx);

                    if let Some(num_components) = &mut target_archetype_num_components {
                        assert!(*num_components == target_storage.len());
                    } else {
                        target_archetype_num_components = Some(target_storage.len());
                    }
                }
                None => {
                    current_storage.swap_remove(entity_idx);
                }
            }
        }

        current_archetype.entities.swap_remove(entity_idx);
        current_archetype.sparse.remove(entity.uindex()).unwrap();

        if current_archetype.entities.len() > 0 && current_archetype.entities.len() != entity_idx {
            let entity = current_archetype.entities[entity_idx];
            *current_archetype.sparse.get_mut(entity.uindex()).unwrap() = entity_idx;
        }

        assert!(target_archetype_num_components.unwrap() > 0);

        target_archetype.entities.push(entity);
        target_archetype.sparse.insert(
            entity.uindex(),
            target_archetype_num_components.unwrap() - 1,
        );

        *self.entity_to_archetype.get_mut(entity.uindex()).unwrap() = target_archetype_idx
    }

    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        if !self.lock_lookup.contains_key(&TypeId::of::<T>()) {
            self.lock_lookup.insert(TypeId::of::<T>(), self.locks.len());
            self.locks.push(RwLock::new(()));
        }

        if !self.entities.is_alive(entity) {
            return;
        }

        let current_archetype_idx = *self.entity_to_archetype.get(entity.uindex()).unwrap();
        let current_archetype = self.archetypes.get_mut(current_archetype_idx).unwrap();
        let current_type_ids = current_archetype.type_ids.clone();

        assert!(!current_type_ids.contains(&TypeId::of::<T>()));

        let mut target_type_ids = current_type_ids.clone();
        target_type_ids.push(TypeId::of::<T>());

        let target_archetype_idx = self
            .find_archetype(&target_type_ids)
            .or_else(|| {
                let mut archetype =
                    Archetype::from_archetype(&mut self.archetypes[current_archetype_idx]);

                archetype.type_ids.push(TypeId::of::<T>());
                archetype
                    .lookup
                    .insert(TypeId::of::<T>(), archetype.component_storages.len());
                archetype
                    .component_storages
                    .push(UnsafeCell::new(UntypedVec::new::<T>()));

                self.archetypes.push(archetype);
                Some(self.archetypes.len() - 1)
            })
            .unwrap();

        let (current_archetype, target_archetype) = {
            if current_archetype_idx < target_archetype_idx {
                let (left, right) = self.archetypes.split_at_mut(target_archetype_idx);
                (
                    left.get_mut(current_archetype_idx).unwrap(),
                    right.first_mut().unwrap(),
                )
            } else if current_archetype_idx > target_archetype_idx {
                let (left, right) = self.archetypes.split_at_mut(current_archetype_idx);
                (
                    right.first_mut().unwrap(),
                    left.get_mut(target_archetype_idx).unwrap(),
                )
            } else {
                panic!()
            }
        };

        let mut target_archetype_num_components = None;
        let entity_idx = *current_archetype.sparse.get(entity.uindex()).unwrap();
        for lock in current_archetype.component_storages.iter_mut() {
            let current_storage = lock.get_mut();
            let type_id = current_storage.get_type_info().id;
            let target_storage_idx = target_archetype.lookup[&type_id];
            let target_storage = target_archetype
                .component_storages
                .get_mut(target_storage_idx)
                .unwrap()
                .get_mut();

            current_storage.swap_move_element_to_other_vec(target_storage, entity_idx);

            if let Some(num_components) = &mut target_archetype_num_components {
                assert!(*num_components == target_storage.len());
            } else {
                target_archetype_num_components = Some(target_storage.len());
            }
        }

        let last_target_storage_idx = target_archetype.lookup[&TypeId::of::<T>()];
        let last_target_storage = target_archetype
            .component_storages
            .get_mut(last_target_storage_idx)
            .unwrap()
            .get_mut();

        last_target_storage.push(component);

        *self.entity_to_archetype.get_mut(entity.uindex()).unwrap() = target_archetype_idx;

        current_archetype.entities.swap_remove(entity_idx);
        current_archetype.sparse.remove(entity.uindex()).unwrap();

        if current_archetype.entities.len() > 0 && current_archetype.entities.len() != entity_idx {
            let entity = current_archetype.entities[entity_idx];
            *current_archetype.sparse.get_mut(entity.uindex()).unwrap() = entity_idx;
        }
        assert!(target_archetype_num_components.unwrap() > 0);

        target_archetype.entities.push(entity);
        target_archetype.sparse.insert(
            entity.uindex(),
            target_archetype_num_components.unwrap() - 1,
        );
    }

    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.entities.is_alive(entity).then_some(())?;

        let archetype_idx = self.find_archetype_from_entity(entity)?;
        let archetype = &mut self.archetypes[archetype_idx];

        let component_type_id = TypeId::of::<T>();
        let component_storage_idx = archetype.lookup[&component_type_id];

        let entity_idx = *archetype.sparse.get(entity.index() as usize)?;

        let component_storage = &mut archetype.component_storages[component_storage_idx];
        let component_storage = component_storage.get_mut();
        let component_storage = component_storage.as_slice_mut::<T>();
        Some(&mut component_storage[entity_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spawn;

    #[test]
    pub fn get() {
        let mut world = World::new();

        let entity = spawn!(&mut world, 10_u32, 12_u64, "Hello");
        let entity2 = spawn!(&mut world, 18_u32, "AWDAWDAWD", 16.0f32);

        let str_comp: &mut &str = world.get_component_mut(entity).unwrap();
        assert!(*str_comp == "Hello");

        let str_comp: &mut &str = world.get_component_mut(entity2).unwrap();
        assert!(*str_comp == "AWDAWDAWD");
    }

    #[test]
    pub fn entity_archetype_lookup() {
        let mut world = World::new();

        let entity = spawn!(&mut world, 10_u32, 12_u64);

        assert!(*world.entity_to_archetype.get(entity.uindex()).unwrap() == 0);
    }

    #[test]
    pub fn add_component() {
        let mut world = World::new();
        let entity = spawn!(&mut world, 1_u32);
        world.add_component(entity, 2_u64);

        assert!(world.archetypes.len() == 2);
        assert!(*world.entity_to_archetype.get(entity.uindex()).unwrap() == 1);

        assert!(world.archetypes[0].sparse.get(entity.uindex()).is_none());
        assert!(world.archetypes[0].entities.len() == 0);
        for lock in world.archetypes[0].component_storages.iter_mut() {
            let storage = lock.get_mut();
            assert!(storage.len() == 0);
        }

        assert!(*world.archetypes[1].sparse.get(entity.uindex()).unwrap() == 0);
        assert!(world.archetypes[1].entities.len() == 1);
        for lock in world.archetypes[1].component_storages.iter_mut() {
            let storage = lock.get_mut();
            assert!(storage.len() == 1);
        }

        let mut run_times = 0;
        let query = world.query::<(&u32, &u64)>();
        query.borrow().for_each_mut(|(left, right)| {
            assert!(*left == 1);
            assert!(*right == 2);
            run_times += 1;
        });
        assert!(run_times == 1);
    }

    #[test]
    pub fn add_component_then_spawn() {
        let mut world = World::new();
        let entity = spawn!(&mut world, 1_u32);
        world.add_component(entity, 2_u64);

        let entity2 = spawn!(&mut world, 3_u32, 4_u64);

        assert!(world.archetypes.len() == 2);
        assert!(*world.entity_to_archetype.get(entity.uindex()).unwrap() == 1);
        assert!(*world.entity_to_archetype.get(entity2.uindex()).unwrap() == 1);

        assert!(world.archetypes[0].sparse.get(entity.uindex()).is_none());
        assert!(world.archetypes[0].sparse.get(entity2.uindex()).is_none());
        assert!(world.archetypes[0].entities.len() == 0);
        for lock in world.archetypes[0].component_storages.iter_mut() {
            let storage = lock.get_mut();
            assert!(storage.len() == 0);
        }

        assert!(*world.archetypes[1].sparse.get(entity.uindex()).unwrap() == 0);
        assert!(*world.archetypes[1].sparse.get(entity2.uindex()).unwrap() == 1);
        assert!(world.archetypes[1].entities.len() == 2);
        for lock in world.archetypes[1].component_storages.iter_mut() {
            let storage = lock.get_mut();
            assert!(storage.len() == 2);
        }

        let mut run_times = 0;
        let mut checks = vec![(1, 2), (3, 4)].into_iter();
        let query = world.query::<(&u32, &u64)>();
        query.borrow().for_each_mut(|(left, right)| {
            assert!(checks.next().unwrap() == (*left, *right));
            run_times += 1;
        });
        assert!(run_times == 2);
    }

    #[test]
    pub fn add_two() {
        struct A(f32);
        struct B(f32);

        let mut world = World::new();
        let entity_1 = spawn!(&mut world, A(1.));
        let entity_2 = spawn!(&mut world, A(1.));

        assert!(world.archetypes[0].entities[0] == entity_1);
        assert!(world.archetypes[0].entities[1] == entity_2);
        assert!(*world.archetypes[0].sparse.get(entity_1.uindex()).unwrap() == 0);
        assert!(*world.archetypes[0].sparse.get(entity_2.uindex()).unwrap() == 1);

        world.add_component(entity_1, B(2.));

        assert!(world.archetypes[0].entities.len() == 1);
        assert!(world.archetypes[0].entities[0] == entity_2);
        dbg!(*world.archetypes[0].sparse.get(entity_2.uindex()).unwrap());
        assert!(*world.archetypes[0].sparse.get(entity_2.uindex()).unwrap() == 0);

        world.add_component(entity_2, B(2.));
    }

    #[test]
    pub fn add_multiple() {
        struct A(f32);
        struct B(f32);

        let mut world = World::new();
        let mut entities = Vec::with_capacity(500);

        for _ in 0..10 {
            entities.push(spawn!(&mut world, A(1.)));
        }

        for &entity in entities.iter() {
            world.add_component(entity, B(1.));
        }
        for &entity in entities.iter() {
            world.remove_component::<B>(entity);
        }
    }
}
