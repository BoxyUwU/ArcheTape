use super::archetype_iter::{Query, QueryInfos};
use super::entities::{EcsId, Entities};
use super::untyped_vec::UntypedVec;
use crate::array_vec::ArrayVec;
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::RwLock;

const CACHE_SIZE: usize = 4;
pub struct AddRemoveCache {
    cache: ArrayVec<(TypeId, usize), CACHE_SIZE>,
    lookup: HashMap<TypeId, usize, crate::TypeIdHasherBuilder>,
}

impl AddRemoveCache {
    fn new() -> Self {
        Self {
            cache: ArrayVec::new(),
            lookup: HashMap::with_capacity_and_hasher(16, crate::TypeIdHasherBuilder()),
        }
    }

    pub fn lookup_id(&mut self, type_id: TypeId) -> Option<usize> {
        for (id, idx) in self.cache.as_slice() {
            if *id == type_id {
                return Some(*idx);
            }
        }

        if let Some(idx) = self.lookup.get(&type_id) {
            self.cache.push_start((type_id, *idx));
            return Some(*idx);
        }

        None
    }

    pub fn insert_id(&mut self, id: TypeId, archetype: usize) {
        self.cache.push_start((id, archetype));
        self.lookup.insert(id, archetype);
    }
}
pub struct Archetype {
    /// A lookup of a component's TypeId to the index into component_storages/type_ids
    pub(crate) lookup: HashMap<TypeId, usize, crate::TypeIdHasherBuilder>,

    /// This vec effectively acts like a component strage and as such should have its elements ordered the same as a component in component_storages
    pub(crate) entities: Vec<EcsId>,

    /// Component storages are sorted such that lower type_ids are first, this means that when adding/removing components we dont need to
    /// go through the lookup hashmap on the other archetype, we can just zip two iterators over component storages and skip the index
    /// for the removed/added type
    pub(crate) component_storages: Vec<UnsafeCell<UntypedVec>>,

    /// The order of this vec is guaranteed to be the same as the order of component storages,
    ///
    /// E.G. if there's an element TypeId::of::<T>() in this vec, then at the same index in component_storages will be the storage for component T
    pub(crate) type_ids: Vec<TypeId>,

    pub(crate) add_remove_cache: AddRemoveCache,
}

use crate::entity_builder::TupleEntry;
impl Archetype {
    pub fn new<T: TupleEntry>(entity: EcsId, tuple: T) -> Self {
        fn new_recursive<T: TupleEntry>(
            entity: EcsId,
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

            let type_ids_len = type_ids.len();

            // We're at the bottom of the tuple
            Archetype {
                type_ids,
                lookup: HashMap::with_capacity_and_hasher(
                    type_ids_len,
                    crate::TypeIdHasherBuilder(),
                ),

                entities: vec![entity],
                component_storages: untyped_vecs,

                add_remove_cache: AddRemoveCache::new(),
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
            add_remove_cache: AddRemoveCache::new(),
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

    pub fn despawn(&mut self, entity: EcsId, entity_idx: usize) -> bool {
        assert!(self.entities[entity_idx] == entity);
        self.entities.swap_remove(entity_idx);
        for storage in self.component_storages.iter_mut().map(UnsafeCell::get_mut) {
            storage.swap_remove(entity_idx);
        }
        false
    }

    pub fn try_find_next_archetype(&mut self, id: TypeId) -> Option<usize> {
        self.add_remove_cache.lookup_id(id)
    }

    pub fn insert_archetype_cache(&mut self, id: TypeId, archetype: usize) {
        self.add_remove_cache.insert_id(id, archetype);
    }
}

#[derive(Clone, Debug)]
pub struct EntityMeta {
    /// Metadata for the instance of this EcsId
    pub(crate) instance_meta: InstanceMeta,
    /// Metadata for when this EcsId is used as a component
    pub(crate) component_meta: ComponentMeta,
}

#[derive(Clone, Debug)]
pub struct ArchIndex(pub usize);
#[derive(Clone, Debug)]
pub struct InstanceMeta {
    pub(crate) archetype: ArchIndex,
    pub(crate) index: usize,
}

#[derive(Clone, Debug)]
pub struct ComponentMeta {
    pub(crate) layout: core::alloc::Layout,
    /// Used as a safety check for rust types
    pub(crate) type_id: Option<TypeId>,
    /// Used for debug printing
    pub(crate) name: Option<String>,
}

impl ComponentMeta {
    /// Creates a ComponentMeta with the type_id and layout of the generic
    pub fn from_generic<T: 'static>() -> Self {
        Self {
            layout: core::alloc::Layout::new::<T>(),
            type_id: Some(TypeId::of::<T>()),
            name: Some(core::any::type_name::<T>().to_owned()),
        }
    }

    /// Creates a unit ComponentMeta, used for when the EcsId should hold no data when added as a component
    pub fn unit() -> Self {
        pub struct NoData;

        Self {
            layout: core::alloc::Layout::new::<NoData>(),
            type_id: Some(TypeId::of::<NoData>()),
            name: Some("No data".to_owned()),
        }
    }
}

pub struct World {
    pub archetypes: Vec<Archetype>,
    entities: Entities,
    cache: Vec<(Vec<TypeId>, usize)>,

    entity_meta: Vec<Option<EntityMeta>>,

    pub(crate) lock_lookup: HashMap<TypeId, usize>,
    pub(crate) locks: Vec<RwLock<()>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            entities: Entities::new(),
            cache: Vec::with_capacity(8),

            entity_meta: Vec::with_capacity(32),

            lock_lookup: HashMap::new(),
            locks: Vec::new(),
        }
    }

    pub fn get_entity_meta(&self, entity: EcsId) -> Option<&EntityMeta> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        self.entity_meta.get(entity.uindex())?.as_ref()
    }

    pub fn set_entity_meta(&mut self, entity: EcsId, meta: EntityMeta) {
        if self.entities.is_alive(entity) {
            let new_meta = Some(meta);
            match self.entity_meta.get_mut(entity.uindex()) {
                Some(old_meta) => *old_meta = new_meta,
                None => {
                    self.entity_meta.resize_with(entity.uindex(), || None);
                    self.entity_meta.push(new_meta);
                }
            }
        }
    }

    pub fn remove_entity_meta(&mut self, entity: EcsId) {
        if self.entities.is_alive(entity) {
            if let Some(meta) = self.entity_meta.get_mut(entity.uindex()) {
                *meta = None;
            }
        }
    }

    pub fn query<T: QueryInfos>(&self) -> Query<T> {
        Query::<T>::new(self)
    }

    pub fn find_archetype(&mut self, type_ids: &[TypeId]) -> Option<ArchIndex> {
        for (cached_type_id, archetype) in self.cache.iter() {
            if *cached_type_id == type_ids {
                return Some(*archetype).map(ArchIndex);
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

        position.map(ArchIndex)
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

    pub fn despawn(&mut self, entity: EcsId) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        let InstanceMeta { archetype, index } =
            self.get_entity_meta(entity).unwrap().instance_meta.clone();

        self.archetypes[archetype.0].despawn(entity, index);
        self.remove_entity_meta(entity);
        self.entities.despawn(entity);
        true
    }

    pub fn remove_component<T: 'static>(&mut self, entity: EcsId) {
        if !self.entities.is_alive(entity) {
            return;
        }

        let (current_archetype_idx, entity_idx) = {
            let meta = self.get_entity_meta(entity).unwrap();
            (
                meta.instance_meta.archetype.clone(),
                meta.instance_meta.index,
            )
        };
        let current_archetype = &mut self.archetypes[current_archetype_idx.0];
        // Note, this is important, caching will give us *wrong* results if we try and remove a component that isnt in this archetype
        assert!(current_archetype.type_ids.contains(&TypeId::of::<T>()));

        let target_archetype_idx = current_archetype
            .try_find_next_archetype(TypeId::of::<T>())
            .or_else(|| {
                // Iterate every archeype to see if one exists
                // TODO MAYBE: technically we dont need to iterate everything, we can calculate the exact archetype.type_ids the
                // target archetype will have so we could store a hashmap of that -> archetype_idx in world to avoid this O(n) lookup

                let current_archetype = &self.archetypes[current_archetype_idx.0];
                let idx = self.find_archetype_without_id_no_cache(
                    &current_archetype.type_ids,
                    TypeId::of::<T>(),
                );

                if let Some(idx) = idx {
                    let current_archetype = &mut self.archetypes[current_archetype_idx.0];
                    current_archetype.insert_archetype_cache(TypeId::of::<T>(), idx);
                }

                idx
            })
            .map(|idx| ArchIndex(idx))
            .unwrap_or_else(|| {
                // Create a new archetype

                let archetype = Archetype::from_archetype_without::<T>(
                    &mut self.archetypes[current_archetype_idx.0],
                );

                self.archetypes.push(archetype);

                let archetypes_len = self.archetypes.len();
                let current_archetype = &mut self.archetypes[current_archetype_idx.0];
                current_archetype.insert_archetype_cache(TypeId::of::<T>(), archetypes_len - 1);
                ArchIndex(archetypes_len - 1)
            });

        let (current_archetype, target_archetype) = crate::index_twice_mut(
            current_archetype_idx.0,
            target_archetype_idx.0,
            &mut self.archetypes,
        );

        let mut skipped_storage = None;
        for ((_, current_storage), target_storage) in current_archetype
            .component_storages
            .iter_mut()
            .map(|current_storage| current_storage.get_mut())
            .enumerate()
            .filter(|(n, current_storage)| {
                if current_storage.get_type_info().id == TypeId::of::<T>() {
                    assert!(skipped_storage.is_none());
                    skipped_storage = Some(*n);
                    false
                } else {
                    true
                }
            })
            .zip(
                target_archetype
                    .component_storages
                    .iter_mut()
                    .map(|target_storage| target_storage.get_mut()),
            )
        {
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

        target_archetype.entities.push(entity);
        {
            let entity_meta = &mut self.entity_meta[entity.uindex()];
            let instance_meta = InstanceMeta {
                archetype: target_archetype_idx,
                index: target_archetype.entities.len() - 1,
            };
            match entity_meta {
                Some(meta) => meta.instance_meta = instance_meta,
                None => {
                    *entity_meta = Some(EntityMeta {
                        instance_meta,
                        component_meta: ComponentMeta::unit(),
                    })
                }
            };
        }

        current_archetype.entities.swap_remove(entity_idx);
        if let Some(&swapped_entity) = current_archetype.entities.get(entity_idx) {
            self.entity_meta[swapped_entity.uindex()]
                .as_mut()
                .unwrap()
                .instance_meta
                .index = entity_idx;
        }
    }

    pub fn add_component<T: 'static>(&mut self, entity: EcsId, component: T) {
        if !self.entities.is_alive(entity) {
            return;
        }

        let (current_archetype_idx, entity_idx) = {
            let meta = self.get_entity_meta(entity).unwrap();
            (
                meta.instance_meta.archetype.clone(),
                meta.instance_meta.index,
            )
        };
        let current_archetype = &mut self.archetypes[current_archetype_idx.0];
        // Note, this is important, caching will give us *wrong* results if we try and add a component that is in this archetype
        assert!(!current_archetype.type_ids.contains(&TypeId::of::<T>()));

        let target_archetype_idx = current_archetype
            .try_find_next_archetype(TypeId::of::<T>())
            .or_else(|| {
                // Iterate every archeype to see if one exists
                // TODO MAYBE: technically we dont need to iterate everything, we can calculate the exact archetype.type_ids the
                // target archetype will have so we could store a hashmap of that -> archetype_idx in world to avoid this O(n) lookup

                let current_archetype = &self.archetypes[current_archetype_idx.0];
                let idx = self.find_archetype_with_id_no_cache(
                    &current_archetype.type_ids,
                    Some(TypeId::of::<T>()),
                );

                if let Some(idx) = idx {
                    let current_archetype = &mut self.archetypes[current_archetype_idx.0];
                    current_archetype.insert_archetype_cache(TypeId::of::<T>(), idx);
                }

                idx
            })
            .map(|idx| ArchIndex(idx))
            .unwrap_or_else(|| {
                // Create a new archetype

                self.lock_lookup.insert(TypeId::of::<T>(), self.locks.len());
                self.locks.push(RwLock::new(()));

                let archetype = Archetype::from_archetype_with::<T>(
                    &mut self.archetypes[current_archetype_idx.0],
                );

                self.archetypes.push(archetype);

                let archetypes_len = self.archetypes.len();
                let current_archetype = &mut self.archetypes[current_archetype_idx.0];
                current_archetype.insert_archetype_cache(TypeId::of::<T>(), archetypes_len - 1);
                ArchIndex(archetypes_len - 1)
            });

        let (current_archetype, target_archetype) = crate::index_twice_mut(
            current_archetype_idx.0,
            target_archetype_idx.0,
            &mut self.archetypes,
        );

        let mut skipped_idx = None;
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

        target_archetype.entities.push(entity);
        {
            let entity_meta = &mut self.entity_meta[entity.uindex()];
            let instance_meta = InstanceMeta {
                archetype: target_archetype_idx,
                index: target_archetype.entities.len() - 1,
            };
            match entity_meta {
                Some(meta) => meta.instance_meta = instance_meta,
                None => {
                    *entity_meta = Some(EntityMeta {
                        instance_meta,
                        component_meta: ComponentMeta::unit(),
                    })
                }
            }
        }

        current_archetype.entities.swap_remove(entity_idx);
        if let Some(&removed_entity) = current_archetype.entities.get(entity_idx) {
            self.entity_meta[removed_entity.uindex()]
                .as_mut()
                .unwrap()
                .instance_meta
                .index = entity_idx;
        }
    }

    pub fn get_component_mut<T: 'static>(&mut self, entity: EcsId) -> Option<&mut T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let (archetype_idx, entity_idx) = {
            let meta = self.get_entity_meta(entity)?;
            (
                meta.instance_meta.archetype.clone(),
                meta.instance_meta.index,
            )
        };
        let archetype = &mut self.archetypes[archetype_idx.0];

        let component_type_id = TypeId::of::<T>();
        let component_storage_idx = archetype.lookup[&component_type_id];

        let component_storage = archetype.component_storages[component_storage_idx]
            .get_mut()
            .as_slice_mut::<T>();
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

        let entity_meta = world.entity_meta[entity.uindex()].clone().unwrap();
        assert!(entity_meta.instance_meta.index == 0);
        assert!(entity_meta.instance_meta.archetype.0 == 0);
    }

    #[test]
    pub fn add_component() {
        let mut world = World::new();
        let entity = spawn!(&mut world, 1_u32);
        world.add_component(entity, 2_u64);

        assert!(world.archetypes.len() == 2);
        let entity_meta = world.entity_meta[entity.uindex()].clone().unwrap();
        assert!(entity_meta.instance_meta.archetype.0 == 1);
        assert!(entity_meta.instance_meta.index == 0);

        assert!(world.archetypes[0].entities.len() == 0);
        for lock in world.archetypes[0].component_storages.iter_mut() {
            let storage = lock.get_mut();
            assert!(storage.len() == 0);
        }

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
        assert!(world.archetypes[1].entities[0] == entity);
        assert!(world.archetypes[1].entities[1] == entity2);

        let entity_meta = world.entity_meta[entity.uindex()].clone().unwrap();
        assert!(entity_meta.instance_meta.archetype.0 == 1);
        assert!(entity_meta.instance_meta.index == 0);

        let entity_meta = world.entity_meta[entity2.uindex()].clone().unwrap();
        assert!(entity_meta.instance_meta.archetype.0 == 1);
        assert!(entity_meta.instance_meta.index == 1);

        assert!(world.archetypes[0].entities.len() == 0);
        for lock in world.archetypes[0].component_storages.iter_mut() {
            let storage = lock.get_mut();
            assert!(storage.len() == 0);
        }

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

        let entity_1_meta = world.entity_meta[entity_1.uindex()].clone().unwrap();
        assert!(world.archetypes[0].entities[0] == entity_1);
        assert!(entity_1_meta.instance_meta.archetype.0 == 0);
        assert!(entity_1_meta.instance_meta.index == 0);

        let entity_2_meta = world.entity_meta[entity_2.uindex()].clone().unwrap();
        assert!(world.archetypes[0].entities[1] == entity_2);
        assert!(entity_2_meta.instance_meta.archetype.0 == 0);
        assert!(entity_2_meta.instance_meta.index == 1);

        world.add_component(entity_1, B(2.));

        let entity_1_meta = world.entity_meta[entity_1.uindex()].clone().unwrap();
        assert!(world.archetypes[1].entities[0] == entity_1);
        assert!(world.archetypes[1].entities.len() == 1);
        assert!(entity_1_meta.instance_meta.archetype.0 == 1);
        assert!(entity_1_meta.instance_meta.index == 0);

        let entity_2_meta = world.entity_meta[entity_2.uindex()].clone().unwrap();
        assert!(world.archetypes[0].entities[0] == entity_2);
        assert!(world.archetypes[0].entities.len() == 1);
        assert!(entity_2_meta.instance_meta.archetype.0 == 0);
        assert!(entity_2_meta.instance_meta.index == 0);

        world.add_component(entity_2, B(2.));
        assert!(world.archetypes[0].entities.len() == 0);
        assert!(world.archetypes[1].entities.len() == 2);

        let entity_1_meta = world.entity_meta[entity_1.uindex()].clone().unwrap();
        assert!(world.archetypes[1].entities[0] == entity_1);
        assert!(entity_1_meta.instance_meta.archetype.0 == 1);
        assert!(entity_1_meta.instance_meta.index == 0);

        let entity_2_meta = world.entity_meta[entity_2.uindex()].clone().unwrap();
        assert!(world.archetypes[1].entities[1] == entity_2);
        assert!(entity_2_meta.instance_meta.archetype.0 == 1);
        assert!(entity_2_meta.instance_meta.index == 1);
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
