use std::{alloc::Layout, ptr::NonNull};
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, realloc},
    collections::HashMap,
    mem::{ManuallyDrop, MaybeUninit},
};

use crate::{
    untyped_vec::{TypeInfo, UntypedVec},
    world::{AddRemoveCache, Archetype, ComponentMeta},
    EcsId, World,
};

pub struct EntityBuilder<'a> {
    data: NonNull<u8>,
    len: usize,
    cap: usize,
    comp_ids: Vec<EcsId>,

    entity: EcsId,
    component_meta: ComponentMeta,

    num_components: usize,

    world: &'a mut World,
}

impl<'a> Drop for EntityBuilder<'a> {
    fn drop(&mut self) {
        // If it never allocated, don't drop
        // self.data will never be null if capacity is non-zero
        if self.cap != 0 {
            unsafe {
                dealloc(
                    self.data.as_ptr(),
                    // Safe as long as cap != 0, which should never happen
                    Layout::from_size_align(self.cap, 1).unwrap(),
                )
            };
            self.len = 0;
            self.cap = 0;
        }
    }
}

impl<'a> EntityBuilder<'a> {
    pub(crate) fn new(world: &'a mut World, entity: EcsId, component_meta: ComponentMeta) -> Self {
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
        if cap == 0 {
            return Self::new(world, entity, component_meta);
        }

        let layout = std::alloc::Layout::from_size_align(cap, 1).unwrap();
        let data = unsafe { alloc(layout) };
        if data.is_null() {
            handle_alloc_error(layout);
        }

        Self {
            data: NonNull::new(data).unwrap(),
            cap,
            len: 0,

            comp_ids: Vec::with_capacity(cap / 8),

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
        assert!(new_size > 0, "Cannot reallocate a capacity of zero");

        if self.cap == 0 {
            self.cap = new_size;
            let layout = std::alloc::Layout::from_size_align(self.cap, 1).unwrap();

            let new_ptr = unsafe { alloc(layout) };

            if new_ptr.is_null() {
                handle_alloc_error(layout);
            }

            self.data = NonNull::new(new_ptr).unwrap();
        } else {
            let layout = std::alloc::Layout::from_size_align(self.cap, 1).unwrap();
            let new_ptr = unsafe {
                realloc(
                    // self.data will never be null here
                    self.data.as_ptr(),
                    layout,
                    new_size,
                )
            };

            // if realloc returns null, reallocation failed, but the old pointer is still valid
            if new_ptr.is_null() {
                handle_alloc_error(layout);
            }

            self.cap = new_size;
            self.data = NonNull::new(new_ptr).unwrap();
        }
    }

    /// Adds an entity as a dataless component
    ///
    /// This method will panic if a component with the ID of component_id expects data. Entities by default expect no data.
    pub fn with_dynamic(mut self, component_id: EcsId) -> Self {
        assert!(
            self.world
                .get_entity_meta(component_id)
                .unwrap()
                .component_meta
                .type_id
                == Some(std::any::TypeId::of::<()>())
        );

        self.comp_ids.push(component_id);
        self.num_components += 1;

        self
    }

    /// Safety:
    ///  data behind ``component`` must not be used again.
    ///  data behind ``component`` must be a valid instance of the type given by ``component_id``
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
                self.data.as_ptr().offset(self.len as isize) as *mut _,
                component_size,
            );
        }
        self.len += component_size;
        self.num_components += 1;

        self
    }

    pub fn with<C: 'static>(self, component: C) -> Self {
        let mut component = ManuallyDrop::new(component);
        let component_id = self.world.get_or_create_type_id_ecsid::<C>();
        unsafe { self.with_dynamic_with_data(&mut component as *mut _ as *mut _, component_id) }
    }

    pub fn build(mut self) -> EcsId {
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
                let comp_storage_index = archetype.lookup[&comp_id];
                unsafe {
                    archetype.component_storages[comp_storage_index]
                        .get_mut()
                        .push_raw(data_ptr.cast());
                    data_ptr = data_ptr.offset(component_meta.layout.size() as isize);
                }

                assert!(
                    archetype.component_storages[comp_storage_index]
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
                if !self.world.lock_lookup.contains_key(id) {
                    self.world
                        .lock_lookup
                        .insert(id.clone(), self.world.locks.len());
                    self.world.locks.push(std::sync::RwLock::new(()));
                }
            }

            let archetype = self.create_archetype();
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
                    comp_id,
                    component_meta.layout,
                    component_meta.drop_fn,
                ))
            };
            unsafe { untyped_vec.push_raw(data_ptr.cast()) };
            component_storages.push(std::cell::UnsafeCell::new(untyped_vec));

            data_ptr = unsafe { data_ptr.offset(component_meta.layout.size() as isize) };
        }

        self.comp_ids.sort();
        component_storages.sort_by(|storage_1, storage_2| {
            let storage_1 = unsafe { &*storage_1.get() };
            let storage_2 = unsafe { &*storage_2.get() };

            Ord::cmp(
                &storage_1.get_type_info().comp_id,
                &storage_2.get_type_info().comp_id,
            )
        });

        let mut lookup = HashMap::with_capacity_and_hasher(
            self.num_components,
            crate::utils::TypeIdHasherBuilder(),
        );
        for (n, &id) in self.comp_ids.iter().enumerate() {
            // Don't insert the same component type twice
            lookup
                .insert(id, n)
                .expect_none("Component type already added");
        }

        assert!(self
            .comp_ids
            .iter()
            .zip(
                component_storages
                    .iter()
                    .map(|storage| unsafe { &*storage.get() })
            )
            .all(|(comp_id, storage)| *comp_id == storage.get_type_info().comp_id));

        Archetype {
            entities: vec![self.entity],
            lookup,
            comp_ids: std::mem::replace(&mut self.comp_ids, Vec::new()),
            component_storages,
            add_remove_cache: AddRemoveCache::new(),
        }
    }
}
