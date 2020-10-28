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
    /// Indexed by entity.index() and returns the index that its components are at inside of the UntypedVecs and
    /// also the index the entity is located at in the entities vec
    pub(crate) sparse: SparseArray<usize, PAGE_SIZE>,

    /// A lookup of a component's TypeId to the index into component_storages/type_ids
    pub(crate) lookup: HashMap<TypeId, usize, crate::TypeIdHasherBuilder>,

    /// This vec effectively acts like a component strage and as such should have its elements ordered the same as a component in component_storages
    pub(crate) entities: Vec<Entity>,

    /// Component storages are sorted such that lower type_ids are first, this means that when adding/removing components we dont need to
    /// go through the lookup hashmap on the other archetype, we can just zip two iterators over component storages and skip the index
    /// for the removed/added type
    pub(crate) component_storages: Vec<UnsafeCell<UntypedVec>>,

    /// The order of this vec is guaranteed to be the same as the order of component storages,
    ///
    /// E.G. if there's an element TypeId::of::<T>() in this vec, then at the same index in component_storages will be the storage for component T
    pub(crate) type_ids: Vec<TypeId>,
}

use crate::entity_builder::TupleEntry;
impl Archetype {
    pub fn new<T: TupleEntry>(entity: Entity, tuple: T) -> Self {
        fn new_recursive<T: TupleEntry>(
            entity: Entity,
            tuple: T,
            mut type_ids: Vec<TypeId>,
            mut untyped_vecs: Vec<UnsafeCell<UntypedVec>>,
        ) -> Archetype {
            if let Some((left, right)) = tuple.next() {
                type_ids.push(TypeId::of::<T::Left>());
                let mut untyped_vec = UntypedVec::new::<T::Left>();
                untyped_vec.push(left);
                untyped_vecs.push(UnsafeCell::new(untyped_vec));
                return new_recursive(entity, right, type_ids, untyped_vecs);
            }

            let mut sparse = SparseArray::new();
            sparse.insert(entity.index() as usize, 0);

            let type_ids_len = type_ids.len();

            // We're at the bottom of the tuple
            Archetype {
                sparse,
                type_ids,
                lookup: HashMap::with_capacity_and_hasher(
                    type_ids_len,
                    crate::TypeIdHasherBuilder(),
                ),

                entities: vec![entity],
                component_storages: untyped_vecs,
            }
        }

        let mut base_archetype = new_recursive(entity, tuple, Vec::new(), Vec::new());

        // TODO there's no need to sort twice they should have the same ordering
        base_archetype.type_ids.sort();
        base_archetype
            .component_storages
            .sort_by(|storage_1, storage_2| {
                let storage_1 = unsafe { &*storage_1.get() };
                let storage_2 = unsafe { &*storage_2.get() };

                Ord::cmp(&storage_1.get_type_info().id, &storage_2.get_type_info().id)
            });

        base_archetype.lookup.clear();
        for (n, &id) in base_archetype.type_ids.iter().enumerate() {
            base_archetype.lookup.insert(id, n);
        }

        debug_assert!(base_archetype
            .type_ids
            .iter()
            .zip(
                base_archetype
                    .component_storages
                    .iter()
                    .map(|storage| unsafe { &*storage.get() })
            )
            .all(|(type_id, storage)| *type_id == storage.get_type_info().id));

        base_archetype
    }

    pub fn from_archetype(from: &mut Archetype) -> Archetype {
        Archetype {
            sparse: SparseArray::new(),

            lookup: from.lookup.clone(),
            type_ids: from.type_ids.clone(),

            entities: Vec::new(),
            component_storages: {
                // Capacity + 1 incase this gets fed into a from_archetype_with call
                let mut storages = Vec::with_capacity(from.component_storages.len() + 1);
                for storage in from.component_storages.iter_mut() {
                    let untyped_vec = UntypedVec::new_from_untyped_vec(storage.get_mut());
                    let cell = UnsafeCell::new(untyped_vec);
                    storages.push(cell);
                }
                storages
            },
        }
    }

    pub fn from_archetype_with<T: 'static>(from: &mut Archetype) -> Archetype {
        let mut base_archetype = Archetype::from_archetype(from);

        assert!(base_archetype.lookup.get(&TypeId::of::<T>()).is_none());

        let with_id = TypeId::of::<T>();

        base_archetype.type_ids.push(with_id);
        base_archetype
            .component_storages
            .push(UnsafeCell::new(UntypedVec::new::<T>()));

        // TODO there's no need to sort twice they should have the same ordering
        base_archetype.type_ids.sort();
        base_archetype
            .component_storages
            .sort_by(|storage_1, storage_2| {
                let storage_1 = unsafe { &*storage_1.get() };
                let storage_2 = unsafe { &*storage_2.get() };

                Ord::cmp(&storage_1.get_type_info().id, &storage_2.get_type_info().id)
            });

        base_archetype.lookup.clear();
        for (n, &id) in base_archetype.type_ids.iter().enumerate() {
            base_archetype.lookup.insert(id, n);
        }

        debug_assert!(base_archetype
            .type_ids
            .iter()
            .zip(
                base_archetype
                    .component_storages
                    .iter()
                    .map(|storage| unsafe { &*storage.get() })
            )
            .all(|(type_id, storage)| *type_id == storage.get_type_info().id));

        base_archetype
    }

    pub fn from_archetype_without<T: 'static>(from: &mut Archetype) -> Archetype {
        let mut base_archetype = Archetype::from_archetype(from);

        assert!(base_archetype.lookup.get(&TypeId::of::<T>()).is_some());

        let remove_idx = base_archetype.lookup[&TypeId::of::<T>()];
        base_archetype.type_ids.remove(remove_idx);
        base_archetype.component_storages.remove(remove_idx);

        // TODO there's no need to sort twice they should have the same ordering
        base_archetype.type_ids.sort();
        base_archetype
            .component_storages
            .sort_by(|storage_1, storage_2| {
                let storage_1 = unsafe { &*storage_1.get() };
                let storage_2 = unsafe { &*storage_2.get() };

                Ord::cmp(&storage_1.get_type_info().id, &storage_2.get_type_info().id)
            });

        base_archetype.lookup.clear();
        for (n, &id) in base_archetype.type_ids.iter().enumerate() {
            base_archetype.lookup.insert(id, n);
        }

        debug_assert!(base_archetype
            .type_ids
            .iter()
            .zip(
                base_archetype
                    .component_storages
                    .iter()
                    .map(|storage| unsafe { &*storage.get() })
            )
            .all(|(type_id, storage)| *type_id == storage.get_type_info().id));

        base_archetype
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
    pub archetypes: Vec<Archetype>,
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

        let position = self.archetypes.iter().position(|archetype| {
            archetype.type_ids.len() == type_ids.len()
                && type_ids.iter().all(|id| archetype.type_ids.contains(id))
        });

        if let Some(position) = position {
            if self.cache.len() > 8 {
                self.cache.pop();
            }
            self.cache.insert(0, (Vec::from(type_ids), position));
        }

        position
    }

    pub fn find_archetype_with_id_no_cache(
        &self,
        type_ids: &[TypeId],
        extra_id: Option<TypeId>,
    ) -> Option<usize> {
        let is_extra = if extra_id.is_some() { 1 } else { 0 };
        let position = self.archetypes.iter().position(|archetype| {
            archetype.type_ids.len() == type_ids.len() + is_extra
                && type_ids.iter().all(|id| archetype.type_ids.contains(id))
                && {
                    if let Some(extra_id) = extra_id {
                        archetype.type_ids.contains(&extra_id)
                    } else {
                        true
                    }
                }
        });

        position
    }

    pub fn find_archetype_without_id_no_cache(
        &self,
        type_ids: &[TypeId],
        without_id: TypeId,
    ) -> Option<usize> {
        let position = self.archetypes.iter().position(|archetype| {
            archetype.type_ids.len() == type_ids.len() - 1
                && archetype
                    .type_ids
                    .iter()
                    .all(|id| *id != without_id && type_ids.contains(id))
        });

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
            components_len: 0,
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

        let current_archetype_idx = self.find_archetype_from_entity(entity).unwrap();
        let current_type_ids = &self.archetypes[current_archetype_idx].type_ids;

        let target_archetype_idx = self
            .find_archetype_without_id_no_cache(current_type_ids, TypeId::of::<T>())
            .or_else(|| {
                let archetype = Archetype::from_archetype_without::<T>(
                    &mut self.archetypes[current_archetype_idx],
                );

                self.archetypes.push(archetype);
                Some(self.archetypes.len() - 1)
            })
            .unwrap();

        let (current_archetype, target_archetype) = crate::index_twice_mut(
            current_archetype_idx,
            target_archetype_idx,
            &mut self.archetypes,
        );

        let mut skipped_storage = None;
        let entity_idx = *current_archetype.sparse.get(entity.uindex()).unwrap();
        for ((_, current_storage), target_storage) in current_archetype
            .component_storages
            .iter_mut()
            .enumerate()
            .filter(|(n, current_storage)| {
                let current_storage = unsafe { &*current_storage.get() };
                if current_storage.get_type_info().id == TypeId::of::<T>() {
                    assert!(skipped_storage.is_none());
                    skipped_storage = Some(*n);
                    false
                } else {
                    true
                }
            })
            .zip(target_archetype.component_storages.iter_mut())
        {
            let current_storage = current_storage.get_mut();
            let target_storage = target_storage.get_mut();

            current_storage.swap_move_element_to_other_vec(target_storage, entity_idx)
        }

        if skipped_storage.is_none() {
            assert!(
                current_archetype
                    .component_storages
                    .last_mut()
                    .unwrap()
                    .get_mut()
                    .get_type_info()
                    .id
                    == TypeId::of::<T>()
            );
            skipped_storage = Some(current_archetype.component_storages.len() - 1);
        }

        current_archetype.component_storages[skipped_storage.unwrap()]
            .get_mut()
            .swap_remove(entity_idx);

        *self.entity_to_archetype.get_mut(entity.uindex()).unwrap() = target_archetype_idx;

        current_archetype.entities.swap_remove(entity_idx);
        current_archetype.sparse.remove(entity.uindex()).unwrap();
        if current_archetype.entities.len() > 0 && current_archetype.entities.len() != entity_idx {
            let entity = current_archetype.entities[entity_idx];
            *current_archetype.sparse.get_mut(entity.uindex()).unwrap() = entity_idx;
        }

        target_archetype.entities.push(entity);
        let target_archetype_components_len = target_archetype.entities.len();
        debug_assert!(target_archetype
            .component_storages
            .iter_mut()
            .map(|storage| storage.get_mut())
            .all(|storage| {
                assert!(target_archetype_components_len == storage.len());
                true
            }));
        target_archetype
            .sparse
            .insert(entity.uindex(), target_archetype_components_len - 1);
    }

    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        if !self.entities.is_alive(entity) {
            return;
        }

        let current_archetype_idx = *self.entity_to_archetype.get(entity.uindex()).unwrap();
        let current_archetype = self.archetypes.get(current_archetype_idx).unwrap();
        let current_type_ids = &current_archetype.type_ids;
        assert!(!current_type_ids.contains(&TypeId::of::<T>()));

        let target_archetype_idx = self
            .find_archetype_with_id_no_cache(current_type_ids, Some(TypeId::of::<T>()))
            .or_else(|| {
                self.lock_lookup.insert(TypeId::of::<T>(), self.locks.len());
                self.locks.push(RwLock::new(()));

                let archetype = Archetype::from_archetype_with::<T>(
                    &mut self.archetypes[current_archetype_idx],
                );

                self.archetypes.push(archetype);
                Some(self.archetypes.len() - 1)
            })
            .unwrap();

        let (current_archetype, target_archetype) = crate::index_twice_mut(
            current_archetype_idx,
            target_archetype_idx,
            &mut self.archetypes,
        );

        let mut skipped_idx = None;
        let entity_idx = *current_archetype.sparse.get(entity.uindex()).unwrap();
        for (current_storage, (_, target_storage)) in current_archetype
            .component_storages
            .iter_mut()
            .map(|current_storage| current_storage.get_mut())
            .zip(
                target_archetype
                    .component_storages
                    .iter_mut()
                    .map(|target_storage| target_storage.get_mut())
                    .enumerate()
                    .filter(|(n, target_storage)| {
                        if target_storage.get_type_info().id == TypeId::of::<T>() {
                            assert!(skipped_idx.is_none());
                            skipped_idx = Some(*n);
                            false
                        } else {
                            true
                        }
                    }),
            )
        {
            current_storage.swap_move_element_to_other_vec(target_storage, entity_idx)
        }

        if skipped_idx.is_none() {
            assert!(
                target_archetype
                    .component_storages
                    .last_mut()
                    .unwrap()
                    .get_mut()
                    .get_type_info()
                    .id
                    == TypeId::of::<T>()
            );
            skipped_idx = Some(target_archetype.component_storages.len() - 1);
        }

        target_archetype.component_storages[skipped_idx.unwrap()]
            .get_mut()
            .push(component);

        *self.entity_to_archetype.get_mut(entity.uindex()).unwrap() = target_archetype_idx;

        current_archetype.entities.swap_remove(entity_idx);
        current_archetype.sparse.remove(entity.uindex()).unwrap();
        if current_archetype.entities.len() > 0 && current_archetype.entities.len() != entity_idx {
            let entity = current_archetype.entities[entity_idx];
            *current_archetype.sparse.get_mut(entity.uindex()).unwrap() = entity_idx;
        }

        target_archetype.entities.push(entity);

        let target_archetype_components_len = target_archetype.entities.len();
        debug_assert!(target_archetype
            .component_storages
            .iter_mut()
            .map(|storage| storage.get_mut())
            .all(|storage| {
                assert!(target_archetype_components_len == storage.len());
                true
            }));
        target_archetype
            .sparse
            .insert(entity.uindex(), target_archetype_components_len - 1);
    }

    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

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
