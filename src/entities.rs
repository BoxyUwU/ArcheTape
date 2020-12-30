use std::fmt::Display;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct EcsId(u64);

impl Display for EcsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(gen {}, index {})", self.generation(), self.index())
    }
}

impl EcsId {
    // Upper 32 bits
    const GENERATION_MASK: u64 = !Self::INDEX_MASK;
    // Lower 32 bits
    const INDEX_MASK: u64 = u64::MAX >> 32;

    pub fn generation(&self) -> u32 {
        ((self.0 & Self::GENERATION_MASK) >> 32) as u32
    }

    pub fn index(&self) -> u32 {
        (self.0 & Self::INDEX_MASK) as u32
    }

    pub fn uindex(&self) -> usize {
        (self.0 & Self::INDEX_MASK) as usize
    }

    pub(crate) fn new(index: u32, generation: u32) -> EcsId {
        EcsId(index as u64 | ((generation as u64) << 32))
    }
}

pub struct Entities {
    pub(crate) generations: Vec<u32>,
    pub(crate) despawned: Vec<usize>,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            generations: Vec::with_capacity(4096),
            despawned: Vec::with_capacity(512),
        }
    }

    pub fn spawn(&mut self) -> EcsId {
        let idx = match self.despawned.pop() {
            Some(idx) => idx,
            None => {
                self.generations.push(0);
                self.generations.len() - 1
            }
        };

        assert!(
            idx <= u32::MAX as usize,
            format!("Out of generation indexes, tried to use index {}", idx)
        );
        let generation = self.generations[idx];
        EcsId::new(idx as u32, generation)
    }

    /// Returns true if entity was despawned
    pub fn despawn(&mut self, to_despawn: EcsId) -> bool {
        if self.is_alive(to_despawn) {
            let generation = &mut self.generations[to_despawn.uindex()];
            *generation = generation.wrapping_add(1);
            self.despawned.push(to_despawn.index() as usize);
            true
        } else {
            false
        }
    }

    pub fn is_alive(&self, entity: EcsId) -> bool {
        let generation = entity.generation();
        let stored_generation = self
            .generations
            .get(entity.uindex())
            .expect(format!("could not get generation for {}", entity).as_ref());
        match generation.cmp(stored_generation) {
            std::cmp::Ordering::Less => {
                false
            }
            std::cmp::Ordering::Equal => {
                true
            }
            std::cmp::Ordering::Greater => {
                panic!("Stored generation {} greater than entity generation {} for entity {}", stored_generation, generation, entity);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{World, spawn};

    use super::*;

    #[test]
    pub fn spawn_one() {
        let mut entities = Entities::new();

        assert_eq!(EcsId(0), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);
    }

    #[test]
    pub fn spawn_multiple() {
        let mut entities = Entities::new();

        assert_eq!(EcsId(0), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        assert_eq!(EcsId(1), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 2);

        assert_eq!(EcsId(2), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 3);

        assert_eq!(EcsId(3), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 4);
    }

    #[test]
    pub fn spawn_one_despawn_one() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(EcsId(0), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        entities.despawn(entity);
        assert!(entities.despawned.len() == 1);
        assert!(entities.generations.len() == 1);
        assert!(*entities.generations.get(0).unwrap() == 1);
        assert!(entities.is_alive(entity) == false);
    }

    #[test]
    #[should_panic(expected = "could not get generation for (gen 4294967295, index 4294967295)")]
    pub fn despawn_invalid() {
        let entities = Entities::new();
        let invalid_id = EcsId(u64::MAX);
        let _ = entities.is_alive(invalid_id);
    }

    #[test]
    pub fn reuse_despawned() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(EcsId(0), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        entities.despawn(entity);
        assert!(entities.despawned.len() == 1);
        assert!(entities.generations.len() == 1);
        assert!(*entities.generations.get(0).unwrap() == 1);
        assert!(entities.is_alive(entity) == false);

        let entity2 = entities.spawn();
        assert_eq!(EcsId::new(0, 1), entity2);
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
        assert_eq!(EcsId(0), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        let entity2 = entities.spawn();
        assert_eq!(EcsId::new(1, 0), entity2);
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

        entities.generations.push(u32::MAX);
        entities.despawned.push(0);

        let entity = entities.spawn();
        entities.despawn(entity);

        let entity = entities.spawn();

        assert!(entities.is_alive(entity));
        assert!(entity == EcsId(0));
        assert!(entities.generations.len() == 1);
        assert!(*entities.generations.get(0).unwrap() == 0);
        assert!(entities.despawned.len() == 0);
    }

    #[test]
    pub fn build_with_zst() -> () {
        struct Zero;

        let mut world = World::new();
        let entity = spawn!(&mut world, Zero);
        assert!(world.is_alive(entity));
    }
}
