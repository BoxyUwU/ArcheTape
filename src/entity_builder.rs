use crate::world::{Archetype, World};
use std::any::TypeId;

pub trait TupleEntry: 'static {
    type Left: 'static;
    type Right: TupleEntry + 'static;

    fn next(self) -> Option<(Self::Left, Self::Right)>;

    fn spawn_fn(self, archetype: &mut Archetype);

    fn collect_type_ids(&self, ids: &mut Vec<TypeId>);
}

impl<T: 'static, U: TupleEntry + 'static> TupleEntry for (T, U) {
    type Left = T;
    type Right = U;

    fn next(self) -> Option<(Self::Left, Self::Right)> {
        Some(self)
    }

    #[inline(always)]
    fn spawn_fn(self, archetype: &mut Archetype) {
        let (left, right) = self;

        let storage_idx = archetype.lookup[&TypeId::of::<Self::Left>()];
        let storage = archetype.component_storages[storage_idx].get_mut();
        storage.push(left);

        Self::Right::spawn_fn(right, archetype);
    }

    fn collect_type_ids(&self, ids: &mut Vec<TypeId>) {
        ids.push(TypeId::of::<Self::Left>());
        Self::Right::collect_type_ids(&self.1, ids);
    }
}

impl TupleEntry for () {
    type Left = ();
    type Right = ();

    fn next(self) -> Option<(Self::Left, Self::Right)> {
        None
    }

    fn spawn_fn(self, archetype: &mut Archetype) {
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

    fn collect_type_ids(&self, _: &mut Vec<TypeId>) {}
}

pub struct EntityBuilder<'w, T = ()>
where
    T: TupleEntry,
{
    pub(crate) entity: crate::entities::Entity,
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

    pub fn build(self) -> crate::entities::Entity {
        let Self {
            world,
            entity,
            components_len,
            components,
        } = self;

        let mut type_ids = Vec::with_capacity(components_len);
        components.collect_type_ids(&mut type_ids);

        if let Some(archetype_idx) = world.find_archetype(&type_ids) {
            let archetype = &mut world.archetypes[archetype_idx];
            archetype.entities.push(entity);
            archetype
                .sparse
                .insert(entity.index() as usize, archetype.entities.len() - 1);

            components.spawn_fn(archetype);
            world.add_entity_to_sparse_array(entity, archetype_idx);
        } else {
            let archetype = crate::world::Archetype::new(entity, components);
            world.archetypes.push(archetype);
            world.add_entity_to_sparse_array(entity, world.archetypes.len() - 1);

            // We only need to create the locks if the archetype wasnt created
            for id in &type_ids {
                if !world.lock_lookup.contains_key(id) {
                    world.lock_lookup.insert(*id, world.locks.len());
                    world.locks.push(std::sync::RwLock::new(()));
                }
            }
        }

        entity
    }
}
