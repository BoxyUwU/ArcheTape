use crate::world::World;
use std::any::TypeId;

pub trait TupleEntry: 'static {
    type Left: 'static;
    type Right: TupleEntry + 'static;

    fn next(self) -> Option<(Self::Left, Self::Right)>;
}

impl<T: 'static, U: TupleEntry + 'static> TupleEntry for (T, U) {
    type Left = T;
    type Right = U;

    fn next(self) -> Option<(Self::Left, Self::Right)> {
        Some(self)
    }
}

impl TupleEntry for () {
    type Left = ();
    type Right = ();

    fn next(self) -> Option<(Self::Left, Self::Right)> {
        None
    }
}

pub struct EntityBuilder<'w, T = ()>
where
    T: TupleEntry,
{
    pub(crate) entity: crate::entities::Entity,
    pub(crate) world: &'w mut World,
    pub(crate) type_ids: Vec<TypeId>,
    pub(crate) components: T,
}

impl<'w, T: TupleEntry> EntityBuilder<'w, T> {
    #[must_use]
    pub fn with<C: 'static>(mut self, component: C) -> EntityBuilder<'w, (C, T)> {
        // Extremely important that the same type doesn't appear twice in the tuple
        assert!(!self.type_ids.contains(&TypeId::of::<C>()));

        self.type_ids.push(TypeId::of::<C>());

        EntityBuilder {
            entity: self.entity,
            world: self.world,
            type_ids: self.type_ids,
            components: (component, self.components),
        }
    }

    pub fn build(self) -> crate::entities::Entity {
        let Self {
            world,
            entity,
            type_ids,
            components,
        } = self;

        if let Some(archetype_idx) = world.find_archetype(&type_ids) {
            world.archetypes[archetype_idx].spawn(entity, components);
            world.add_entity_to_sparse_array(entity, archetype_idx);
        } else {
            let archetype = crate::world::Archetype::new(entity, components);
            world.archetypes.push(archetype);
            world.add_entity_to_sparse_array(entity, world.archetypes.len() - 1);
        }

        for id in &type_ids {
            if !world.lock_lookup.contains_key(id) {
                world.lock_lookup.insert(*id, world.locks.len());
                world.locks.push(std::sync::RwLock::new(()));
            }
        }

        entity
    }
}
