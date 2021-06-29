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

#[derive(Copy, Clone, Debug, Ord, PartialOrd)]
pub struct EcsId(EcsIdGen, EcsIdIndex);

impl std::fmt::Display for EcsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.generation(), self.index())
    }
}

impl PartialEq for EcsId {
    fn eq(&self, other: &EcsId) -> bool {
        self.as_u64() == other.as_u64()
    }
}
impl Eq for EcsId {}

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

impl Default for Entities {
    fn default() -> Self {
        Self {
            generations: Vec::with_capacity(4096),
            despawned: Vec::with_capacity(512),
        }
    }
}
impl Entities {
    pub fn new() -> Self {
        Default::default()
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
            .unwrap_or_else(|| panic!("could not get generation for {}", entity));
        let generation = entity.generation().0;
        alive && generation == stored_generation
    }
}
