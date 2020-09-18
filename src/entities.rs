#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Entity(u64);

impl Entity {
    const GENERATION_MASK: u64 = !Self::INDEX_MASK;
    const INDEX_MASK: u64 = u64::MAX >> 16;

    pub fn generation(&self) -> u16 {
        ((self.0 & Self::GENERATION_MASK) >> 48) as u16
    }

    pub fn index(&self) -> u64 {
        self.0 & Self::INDEX_MASK
    }

    pub fn uindex(&self) -> usize {
        (self.0 & Self::INDEX_MASK) as usize
    }

    pub(crate) fn new(index: u64, generation: u16) -> Entity {
        debug_assert!(index & Self::GENERATION_MASK == 0);
        Entity(index | ((generation as u64) << 48))
    }
}

pub struct Entities {
    pub(crate) generations: Vec<u16>,
    pub(crate) despawned: Vec<usize>,
}

impl Entities {
    pub(crate) fn new() -> Self {
        Self {
            generations: Vec::with_capacity(4096),
            despawned: Vec::with_capacity(512),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let idx = match self.despawned.pop() {
            Some(idx) => idx,
            None => {
                self.generations.push(0);
                self.generations.len() - 1
            }
        };

        let generation = *self.generations.get_mut(idx).unwrap();
        Entity::new(idx as u64, generation)
    }

    /// Returns true if entity was despawned
    pub fn despawn(&mut self, to_despawn: Entity) -> bool {
        let generation = to_despawn.generation();
        let index = to_despawn.uindex();

        let entity = self.generations.get_mut(index).unwrap();

        if *entity != generation {
            return false;
        };

        *entity = (*entity).wrapping_add(1);
        self.despawned.push(index);
        true
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        *self.generations.get(entity.uindex()).unwrap() == entity.generation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn spawn_one() {
        let mut entities = Entities::new();

        assert_eq!(Entity(0), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);
    }

    #[test]
    pub fn spawn_multiple() {
        let mut entities = Entities::new();

        assert_eq!(Entity(0), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        assert_eq!(Entity(1), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 2);

        assert_eq!(Entity(2), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 3);

        assert_eq!(Entity(3), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 4);
    }

    #[test]
    pub fn spawn_one_despawn_one() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(Entity(0), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        entities.despawn(entity);
        assert!(entities.despawned.len() == 1);
        assert!(entities.generations.len() == 1);
        assert!(*entities.generations.get(0).unwrap() == 1);
        assert!(entities.is_alive(entity) == false);
    }

    #[test]
    pub fn reuse_despawned() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(Entity(0), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        entities.despawn(entity);
        assert!(entities.despawned.len() == 1);
        assert!(entities.generations.len() == 1);
        assert!(*entities.generations.get(0).unwrap() == 1);
        assert!(entities.is_alive(entity) == false);

        let entity2 = entities.spawn();
        assert_eq!(Entity::new(0, 1), entity2);
        assert!(entity != entity2);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);
        assert!(*entities.generations.get(0).unwrap() == 1);
        assert!(entities.is_alive(entity) == false);
        assert!(entities.is_alive(entity2) == true);
    }

    #[test]
    pub fn double_despawn() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        entities.despawn(entity);

        assert!(entities.despawn(entity) == false);
        assert!(entities.despawned.len() == 1);
        assert!(entities.generations.len() == 1);
        assert!(*entities.generations.get(0).unwrap() == 1);
        assert!(entities.is_alive(entity) == false);
    }

    #[test]
    pub fn reuse_despawn2() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(Entity(0), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        let entity2 = entities.spawn();
        assert_eq!(Entity::new(1, 0), entity2);
        assert!(entity != entity2);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 2);
        assert!(*entities.generations.get(0).unwrap() == 0);
        assert!(*entities.generations.get(1).unwrap() == 0);
        assert!(entities.is_alive(entity) == true);
        assert!(entities.is_alive(entity2) == true);

        assert!(entities.despawn(entity) == true);
        assert!(entities.is_alive(entity) == false);
        assert!(entities.is_alive(entity2) == true);
        assert!(entities.generations.len() == 2);
        assert!(*entities.generations.get(0).unwrap() == 1);
        assert!(*entities.generations.get(1).unwrap() == 0);
        assert!(entities.despawned.len() == 1);
        assert!(*entities.despawned.get(0).unwrap() == 0);

        let entity3 = entities.spawn();
        assert!(entities.is_alive(entity) == false);
        assert!(entities.is_alive(entity2) == true);
        assert!(entities.is_alive(entity3) == true);

        assert!(entities.generations.len() == 2);
        assert!(*entities.generations.get(0).unwrap() == 1);
        assert!(*entities.generations.get(1).unwrap() == 0);

        assert!(entities.despawned.len() == 0);
    }

    #[test]
    pub fn generation_wraps() {
        let mut entities = Entities::new();

        for n in 0..=u16::MAX {
            let entity = entities.spawn();
            entities.despawn(entity);

            assert!(entity == Entity::new(0, n));
            assert!(entities.is_alive(entity) == false);
            assert!(entities.generations.len() == 1);
            assert!(*entities.generations.get(0).unwrap() == n.wrapping_add(1));
            assert!(entities.despawned.len() == 1);
            assert!(*entities.despawned.get(0).unwrap() == 0);
        }

        let entity = entities.spawn();
        assert!(entities.is_alive(entity));
        assert!(entity == Entity(0));
        assert!(entities.generations.len() == 1);
        assert!(*entities.generations.get(0).unwrap() == 0);
        assert!(entities.despawned.len() == 0);
    }
}
