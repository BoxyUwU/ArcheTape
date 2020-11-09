use crate::entities::EcsId;
use crate::world::{Archetype, World};
use std::any::TypeId;
use std::collections::HashMap;

pub trait TupleEntry: 'static {
    type Left: 'static;
    type Right: TupleEntry + 'static;

    fn next(self) -> Option<(Self::Left, Self::Right)>;

    fn spawn_fn(
        self,
        archetype: &mut Archetype,
        ecs_id_to_type_id: &HashMap<TypeId, EcsId, crate::utils::TypeIdHasherBuilder>,
    );

    fn collect_comp_ids(&self, ids: &mut Vec<EcsId>, world: &mut World);
}

impl<T: 'static, U: TupleEntry + 'static> TupleEntry for (T, U) {
    type Left = T;
    type Right = U;

    fn next(self) -> Option<(Self::Left, Self::Right)> {
        Some(self)
    }

    #[inline(always)]
    fn spawn_fn(
        self,
        archetype: &mut Archetype,
        type_id_to_ecs_id: &HashMap<TypeId, EcsId, crate::utils::TypeIdHasherBuilder>,
    ) {
        let (left, right) = self;

        let comp_id = &type_id_to_ecs_id[&TypeId::of::<Self::Left>()];
        let storage_idx = archetype.lookup[comp_id];
        let storage = archetype.component_storages[storage_idx].get_mut();

        {
            use core::mem::ManuallyDrop;
            use core::mem::MaybeUninit;
            let mut left = ManuallyDrop::new(left);
            // Safe as long as type_id_to_ecs_id and archetype.lookup are correct
            unsafe { storage.push_raw({ &mut left } as *mut _ as *mut MaybeUninit<u8>) }
        }

        Self::Right::spawn_fn(right, archetype, type_id_to_ecs_id);
    }

    fn collect_comp_ids(&self, ids: &mut Vec<EcsId>, world: &mut World) {
        let comp_id = world.get_or_create_type_id_ecsid::<Self::Left>();
        ids.push(comp_id);
        Self::Right::collect_comp_ids(&self.1, ids, world);
    }
}

impl TupleEntry for () {
    type Left = ();
    type Right = ();

    fn next(self) -> Option<(Self::Left, Self::Right)> {
        None
    }

    fn spawn_fn(
        self,
        archetype: &mut Archetype,
        _: &HashMap<TypeId, EcsId, crate::utils::TypeIdHasherBuilder>,
    ) {
        // This makes sure the same component was not added twice
        // TODO overwrite old component instead?
        let entities_len = archetype.entities.len();
        for storage in archetype
            .component_storages
            .iter_mut()
            .map(|cell| cell.get_mut())
        {
            let storage_len = storage.raw_len();
            let type_size = storage.get_type_info().layout.size();
            if type_size == 0 {
                assert!(storage_len == entities_len);
            } else {
                assert!(storage_len == entities_len * type_size);
            }
        }
    }

    fn collect_comp_ids(&self, _: &mut Vec<EcsId>, _: &mut World) {}
}

pub struct EntityBuilder<'w, T = ()>
where
    T: TupleEntry,
{
    pub(crate) entity: crate::entities::EcsId,
    pub(crate) world: &'w mut World,
    pub(crate) components_len: usize,
    pub(crate) components: T,
}

impl<'w, T: TupleEntry> EntityBuilder<'w, T> {
    #[must_use]
    pub fn with<C: 'static>(self, component: C) -> EntityBuilder<'w, (C, T)> {
        EntityBuilder {
            entity: self.entity,
            world: self.world,
            components_len: self.components_len + 1,
            components: (component, self.components),
        }
    }

    pub fn build(self) -> crate::entities::EcsId {
        let Self {
            world,
            entity,
            components_len,
            components,
        } = self;

        let mut comp_ids = Vec::with_capacity(components_len);
        components.collect_comp_ids(&mut comp_ids, world);

        if let Some(archetype_idx) = world.find_archetype_dynamic(&comp_ids) {
            let archetype = &mut world.archetypes[archetype_idx.0];
            archetype.entities.push(entity);
            components.spawn_fn(archetype, &world.type_id_to_ecs_id);

            let entities_len = archetype.entities.len();
            let entity_meta = crate::world::EntityMeta {
                instance_meta: crate::world::InstanceMeta {
                    archetype: archetype_idx,
                    index: entities_len - 1,
                },
                component_meta: crate::world::ComponentMeta::unit(),
            };
            world.set_entity_meta(entity, entity_meta);
        } else {
            // We only need to create the locks if the archetype wasnt created
            for id in &comp_ids {
                if !world.lock_lookup.contains_key(id) {
                    world.lock_lookup.insert(id.clone(), world.locks.len());
                    world.locks.push(std::sync::RwLock::new(()));
                }
            }

            let archetype = Archetype::new(entity, components, comp_ids);
            world.archetypes.push(archetype);

            use crate::world::ArchIndex;
            let entity_meta = crate::world::EntityMeta {
                instance_meta: crate::world::InstanceMeta {
                    archetype: ArchIndex(world.archetypes.len() - 1),
                    index: 0,
                },
                component_meta: crate::world::ComponentMeta::unit(),
            };
            world.set_entity_meta(entity, entity_meta);
        }

        entity
    }
}
