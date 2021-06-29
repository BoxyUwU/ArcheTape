use std::ptr::NonNull;
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout},
    collections::HashMap,
    mem::{ManuallyDrop, MaybeUninit},
};

use crate::{
    world::{AddRemoveCache, Archetype, ComponentMeta},
    EcsId, World,
};
use untyped_vec::{TypeInfo, UntypedVec};

pub struct EntityBuilder<'a> {
    data: NonNull<u8>,
    cap: usize,
    len: usize,
    comp_ids: Vec<EcsId>,

    entity: EcsId,
    component_meta: ComponentMeta,

    num_components: usize,

    world: &'a mut World,
}

impl<'a> Drop for EntityBuilder<'a> {
    fn drop(&mut self) {
        // If it never allocated, don't drop
        if self.cap != 0 {
            if self.world.entity_builder_reuse.is_none() {
                let mut comp_ids = std::mem::take(&mut self.comp_ids);
                comp_ids.clear();

                let ptr = std::mem::replace(&mut self.data, NonNull::dangling());
                let cap = self.cap;
                self.cap = 0;
                self.len = 0;

                self.world.entity_builder_reuse = Some((comp_ids, ptr, cap));
                return;
            }

            // We only ever use the global allocator for `self.data`
            // The size of the memory currently allocated is always kept in sync with self.cap
            // The size of the memory must also be non-zero, which is checked above
            // The align is always 1
            unsafe {
                dealloc(
                    self.data.as_ptr(),
                    Layout::from_size_align(self.cap, 1).unwrap(),
                )
            };
            self.len = 0;
            self.cap = 0;
        }
    }
}

impl<'a> EntityBuilder<'a> {
    pub(crate) fn new_from_world_cache(
        world: &'a mut World,
        entity: EcsId,
        component_meta: ComponentMeta,
    ) -> Self {
        let (comp_ids, data, cap) = world.entity_builder_reuse.take().unwrap();

        Self {
            data,
            cap,
            len: 0,
            comp_ids,
            component_meta,
            entity,
            world,
            num_components: 0,
        }
    }

    pub(crate) fn new(world: &'a mut World, entity: EcsId, component_meta: ComponentMeta) -> Self {
        if world.entity_builder_reuse.is_some() {
            return Self::new_from_world_cache(world, entity, component_meta);
        }

        Self {
            data: NonNull::dangling(),
            cap: 0,
            len: 0,

            comp_ids: Vec::with_capacity(8),

            entity,
            component_meta,

            num_components: 0,

            world,
        }
    }

    pub(crate) fn with_capacity(
        world: &'a mut World,
        entity: EcsId,
        component_meta: ComponentMeta,
        cap: usize,
    ) -> Self {
        if world.entity_builder_reuse.is_some() {
            return Self::new_from_world_cache(world, entity, component_meta);
        }

        if cap == 0 {
            return Self::new(world, entity, component_meta);
        }

        let layout = Layout::from_size_align(cap, 1).unwrap();
        // Safe because layout size 0 is handed without allocating
        let ptr = unsafe { alloc(layout) };
        let data = NonNull::new(ptr).unwrap_or_else(|| handle_alloc_error(layout));

        Self {
            data,
            cap,
            len: 0,

            comp_ids: Vec::with_capacity(8),

            entity,
            component_meta,

            num_components: 0,

            world,
        }
    }

    fn realloc(&mut self, new_size: usize) {
        assert!(
            new_size < isize::MAX as usize,
            "Cannot allocate more than isize::MAX bytes"
        );
        assert!(new_size > 0, "Cannot reallocate to a capacity of zero");

        if self.cap == 0 {
            let layout = Layout::from_size_align(new_size, 1).unwrap();
            // Safe because new_size is asserted to be greater than zero
            let new_ptr = unsafe { alloc(layout) };
            self.data = NonNull::new(new_ptr).unwrap_or_else(|| handle_alloc_error(layout));
        } else {
            let layout = Layout::from_size_align(self.cap, 1).unwrap();
            // self.data is always allocated using the global allocator
            // Layout is always the same layout because cap is kept in sync and always > 0 here
            // new_size is asserted to be greater than 0
            let new_ptr = unsafe { realloc(self.data.as_ptr(), layout, new_size) };
            self.data = NonNull::new(new_ptr).unwrap_or_else(|| handle_alloc_error(layout));
        }
        self.cap = new_size;
    }

    /// Adds an entity as a dataless component
    ///
    /// This method will panic if a component with the ID of component_id expects data. Entities by default expect no data.
    #[must_use]
    pub fn with_dynamic(mut self, component_id: EcsId) -> Self {
        assert!(
            self.world
                .get_entity_meta(component_id)
                .unwrap()
                .component_meta
                .is_unit
        );

        self.comp_ids.push(component_id);
        self.num_components += 1;

        self
    }

    /// # Safety
    ///
    ///    data behind ``component`` must not be used again.
    ///    data behind ``component`` must be a valid instance of the type given by ``component_id``
    #[must_use]
    pub unsafe fn with_dynamic_with_data(
        mut self,
        component: *mut u8,
        component_id: EcsId,
    ) -> Self {
        self.comp_ids.push(component_id);
        let component_size = self
            .world
            .get_entity_meta(component_id)
            .expect("Dead entity may not be used as a component")
            .component_meta
            .layout
            .size();

        let required_size = self.len + component_size;
        if required_size > self.cap {
            let new_size = usize::max(required_size, self.cap * 2);
            self.realloc(new_size);
        }

        unsafe {
            std::ptr::copy_nonoverlapping::<MaybeUninit<u8>>(
                component as *mut _,
                self.data.as_ptr().add(self.len) as *mut _,
                component_size,
            );
        }
        self.len += component_size;
        self.num_components += 1;

        self
    }

    #[must_use]
    pub fn with<C: 'static>(self, component: C) -> Self {
        let mut component = ManuallyDrop::new(component);
        let component_id = self.world.get_or_create_type_id_ecsid::<C>();
        unsafe { self.with_dynamic_with_data(&mut component as *mut _ as *mut _, component_id) }
    }

    pub fn build(&mut self) -> EcsId {
        use crate::world::{ArchIndex, EntityMeta, InstanceMeta};
        if let Some(arch_index) = self.world.find_archetype_dynamic(&self.comp_ids) {
            self.world.archetypes[arch_index.0]
                .entities
                .push(self.entity);

            let mut data_ptr = self.data.as_ptr();
            for &comp_id in &self.comp_ids {
                let component_meta = self
                    .world
                    .get_entity_meta(comp_id)
                    .unwrap()
                    .component_meta
                    .clone();

                let archetype = &mut self.world.archetypes[arch_index.0];
                let comp_storage_index = archetype.comp_lookup[&comp_id];
                unsafe {
                    archetype.component_storages[comp_storage_index]
                        .1
                        .get_mut()
                        .push_raw(data_ptr.cast());
                    data_ptr = data_ptr.add(component_meta.layout.size());
                }

                assert!(
                    archetype.component_storages[comp_storage_index]
                        .1
                        .get_mut()
                        .len()
                        == archetype.entities.len()
                );
            }
            let entity_idx = self.world.archetypes[arch_index.0].entities.len() - 1;
            let entity_meta = EntityMeta {
                instance_meta: InstanceMeta {
                    archetype: arch_index,
                    index: entity_idx,
                },
                component_meta: self.component_meta.clone(),
            };
            self.world.set_entity_meta(self.entity, entity_meta);
        } else {
            for id in &self.comp_ids {
                use std::collections::hash_map::Entry;
                let entry = self.world.lock_lookup.entry(*id);
                if let Entry::Vacant(entry) = entry {
                    entry.insert(self.world.locks.len());
                    self.world.locks.push(std::sync::RwLock::new(()));
                }
            }

            let archetype = self.create_archetype();

            for id in archetype.comp_ids.iter() {
                self.world
                    .archetype_bitset
                    .set_bit(*id, self.world.archetypes.len(), true);
            }
            self.world.entities_bitvec.push_bit(true);

            self.world.archetypes.push(archetype);
            let (archetype_idx, entity_idx) = (ArchIndex(self.world.archetypes.len() - 1), 0);

            let entity_meta = EntityMeta {
                instance_meta: InstanceMeta {
                    archetype: archetype_idx,
                    index: entity_idx,
                },
                component_meta: self.component_meta.clone(),
            };
            self.world.set_entity_meta(self.entity, entity_meta);
        }

        self.entity
    }

    /// Creates an archetype and moves the built entity into it
    fn create_archetype(&mut self) -> Archetype {
        let mut component_storages = Vec::with_capacity(self.num_components);

        let mut data_ptr = self.data.as_ptr();
        for &comp_id in &self.comp_ids {
            let component_meta = &self.world.get_entity_meta(comp_id).unwrap().component_meta;
            let mut untyped_vec = unsafe {
                UntypedVec::new_from_raw(TypeInfo::new(
                    component_meta.layout,
                    component_meta.drop_fn,
                ))
            };
            unsafe { untyped_vec.push_raw(data_ptr.cast()) };
            component_storages.push((comp_id, std::cell::UnsafeCell::new(untyped_vec)));

            data_ptr = unsafe { data_ptr.add(component_meta.layout.size()) };
        }

        self.comp_ids.sort();
        component_storages.sort_by(|(id1, _), (id2, _)| Ord::cmp(&id1, &id2));

        let mut lookup = HashMap::with_capacity_and_hasher(
            self.num_components,
            crate::utils::TypeIdHasherBuilder(),
        );
        for (n, &id) in self.comp_ids.iter().enumerate() {
            // Dont add the same component twice
            assert!(
                lookup.insert(id, n).is_none(),
                "Attempted to add the same component twice in EntityBuilder"
            );
        }

        assert!(
            self.comp_ids
                .iter()
                .zip(component_storages.iter().map(|(id, _)| id))
                .all(|(id1, id2)| id1 == id2)
        );

        Archetype {
            entities: vec![self.entity],
            comp_lookup: lookup,
            comp_ids: std::mem::take(&mut self.comp_ids),
            component_storages,
            add_remove_cache: AddRemoveCache::new(),
        }
    }
}
