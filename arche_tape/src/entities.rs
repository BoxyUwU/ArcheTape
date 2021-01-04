#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct EcsIdGen(u32);
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct EcsIdIndex(u32);

impl std::fmt::Debug for EcsIdGen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EcsId generation: {:#010X}", self.0)
    }
}

impl std::fmt::Display for EcsIdGen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Generation {:#010X}", self.0)
    }
}

impl std::fmt::Debug for EcsIdIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EcsId index: {:#010X}", self.0)
    }
}

impl std::fmt::Display for EcsIdIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index {:#010X}", self.0)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EcsId(EcsIdGen, EcsIdIndex);

impl std::fmt::Display for EcsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.generation(), self.index())
    }
}

impl std::hash::Hash for EcsId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        u64::hash(&self.as_u64(), state)
    }
}

impl EcsId {
    pub fn generation(&self) -> EcsIdGen {
        self.0
    }

    pub fn index(&self) -> EcsIdIndex {
        self.1
    }

    pub fn uindex(&self) -> usize {
        self.index().0 as usize
    }

    pub(crate) fn new(index: u32, generation: u32) -> EcsId {
        Self(EcsIdGen(generation), EcsIdIndex(index))
    }

    pub fn as_u64(&self) -> u64 {
        let gen = self.generation().0;
        let gen = { gen as u64 } << 32;

        let idx = self.index().0 as u64;

        gen | idx
    }
}

pub struct Entities {
    /// the bool is whether the entity is alive
    /// the u32 is the generation of the entity
    pub(crate) generations: Vec<(bool, u32)>,
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
            Some(idx) => {
                let (alive, gen) = &mut self.generations[idx];
                assert!(*alive == false);
                *gen = gen.wrapping_add(1);
                *alive = true;
                idx
            }
            None => {
                self.generations.push((true, 0));
                self.generations.len() - 1
            }
        };

        if idx > u32::MAX as usize {
            todo!("Handle running out of entity ids");
        }

        let &mut (_, gen) = &mut self.generations[idx];
        EcsId::new(idx as u32, gen)
    }

    /// Returns true if entity was despawned
    pub fn despawn(&mut self, to_despawn: EcsId) -> bool {
        if self.is_alive(to_despawn) {
            let (alive, _) = &mut self.generations[to_despawn.uindex()];
            *alive = false;
            self.despawned.push(to_despawn.uindex());
            true
        } else {
            false
        }
    }

    pub fn is_alive(&self, entity: EcsId) -> bool {
        let &(alive, stored_generation) = self
            .generations
            .get(entity.uindex())
            .expect(format!("could not get generation for {}", entity).as_ref());
        let generation = entity.generation().0;
        alive && generation == stored_generation
    }
}

#[cfg(test)]
mod tests {
    use crate::{spawn, World};

    use super::*;

    #[test]
    pub fn spawn_one() {
        let mut entities = Entities::new();

        assert_eq!(EcsId(EcsIdGen(0), EcsIdIndex(0)), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);
    }

    #[test]
    pub fn spawn_multiple() {
        let mut entities = Entities::new();

        assert_eq!(EcsId(EcsIdGen(0), EcsIdIndex(0)), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        assert_eq!(EcsId(EcsIdGen(0), EcsIdIndex(1)), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 2);

        assert_eq!(EcsId(EcsIdGen(0), EcsIdIndex(2)), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 3);

        assert_eq!(EcsId(EcsIdGen(0), EcsIdIndex(3)), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 4);
    }

    #[test]
    pub fn spawn_one_despawn_one() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(EcsId(EcsIdGen(0), EcsIdIndex(0)), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        entities.despawn(entity);
        assert!(entities.despawned.len() == 1);
        assert!(entities.generations.len() == 1);
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.is_alive(entity) == false);
    }

    #[test]
    #[should_panic(expected = "could not get generation for Generation 0xFFFFFFFF, Index 0xFFFFFFFF")]
    pub fn despawn_invalid() {
        let entities = Entities::new();
        let invalid_id = EcsId(EcsIdGen(u32::MAX), EcsIdIndex(u32::MAX));
        let _ = entities.is_alive(invalid_id);
    }

    #[test]
    pub fn reuse_despawned() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(EcsId(EcsIdGen(0), EcsIdIndex(0)), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        entities.despawn(entity);
        assert!(entities.despawned.len() == 1);
        assert!(entities.generations.len() == 1);
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.is_alive(entity) == false);

        let entity2 = entities.spawn();
        assert_eq!(EcsId::new(0, 1), entity2);
        assert!(entity != entity2);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);
        assert!(entities.generations.get(0).unwrap().1 == 1);
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
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.is_alive(entity) == false);

        assert!(entities.spawn() == EcsId::new(0, 1));
    }

    #[test]
    pub fn reuse_despawn2() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(EcsId(EcsIdGen(0), EcsIdIndex(0)), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        let entity2 = entities.spawn();
        assert_eq!(EcsId::new(1, 0), entity2);
        assert!(entity != entity2);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 2);
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.generations.get(1).unwrap().1 == 0);
        assert!(entities.is_alive(entity) == true);
        assert!(entities.is_alive(entity2) == true);

        assert!(entities.despawn(entity) == true);
        assert!(entities.is_alive(entity) == false);
        assert!(entities.is_alive(entity2) == true);
        assert!(entities.generations.len() == 2);
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.generations.get(1).unwrap().1 == 0);
        assert!(entities.despawned.len() == 1);
        assert!(*entities.despawned.get(0).unwrap() == 0);

        let entity3 = entities.spawn();
        assert!(entities.is_alive(entity) == false);
        assert!(entities.is_alive(entity2) == true);
        assert!(entities.is_alive(entity3) == true);

        assert!(entities.generations.len() == 2);
        assert!(entities.generations.get(0).unwrap().1 == 1);
        assert!(entities.generations.get(1).unwrap().1 == 0);

        assert!(entities.despawned.len() == 0);
    }

    #[test]
    pub fn generation_wraps() {
        let mut entities = Entities::new();

        entities.generations.push((false, u32::MAX));
        entities.despawned.push(0);

        let entity = entities.spawn();

        assert!(entities.is_alive(entity));
        assert!(entity == EcsId(EcsIdGen(0), EcsIdIndex(0)));
        assert!(entities.generations.len() == 1);
        assert!(entities.generations.get(0).unwrap().1 == 0);
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
