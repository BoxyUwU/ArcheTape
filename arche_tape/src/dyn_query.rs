use crate::utils::EitherGuard;
use crate::{world::Archetype, EcsId, World};
use std::marker::PhantomData;

struct IntraArchetypeIter<'a, const N: usize> {
    remaining: usize,

    ptrs: [*mut u8; N],
    offsets: [usize; N],

    phantom: PhantomData<&'a mut Archetype>,
}

impl<'a, const N: usize> IntraArchetypeIter<'a, N> {
    /// Empty iterator
    fn unit() -> Self {
        Self {
            remaining: 0,
            ptrs: [0x0 as _; N],
            offsets: [0; N],
            phantom: PhantomData,
        }
    }

    fn new(length: usize, ptrs: [*mut u8; N], offsets: [usize; N]) -> Self {
        Self {
            remaining: length,
            ptrs,
            offsets,
            phantom: PhantomData,
        }
    }
}

impl<'a, const N: usize> Iterator for IntraArchetypeIter<'a, N> {
    type Item = [*mut u8; N];

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let ptrs = self.ptrs;

        for (ptr, offset) in self.ptrs.iter_mut().zip(self.offsets.iter()) {
            unsafe { *ptr = ptr.add(*offset) }
        }
        self.remaining -= 1;

        Some(ptrs)
    }

    fn for_each<F: FnMut(Self::Item)>(self, mut f: F) {
        let mut ptrs = self.ptrs;
        for _ in 0..self.remaining {
            for (ptr, offset) in ptrs.iter_mut().zip(self.offsets.iter()) {
                unsafe { *ptr = ptr.add(*offset) }
            }
            f(ptrs);
        }
    }
}

pub struct QueryIter<'a, I: Iterator<Item = &'a Archetype>, const N: usize> {
    comp_ids: [Option<EcsId>; N],
    create_ptr: [fn(&Archetype, Option<EcsId>) -> (*mut u8, usize); N],
    archetype_iter: I,
    intra_iter: IntraArchetypeIter<'a, N>,
}

impl<'a, I, const N: usize> Iterator for QueryIter<'a, I, N>
where
    I: Iterator<Item = &'a Archetype>,
{
    type Item = [*mut u8; N];

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.intra_iter.next() {
                None => {
                    let archetype = self.archetype_iter.next()?;

                    let mut ptrs = [0x0 as _; N];
                    let mut offsets = [0; N];
                    for n in 0..N {
                        let (ptr, offset) = self.create_ptr[n](archetype, self.comp_ids[n]);
                        ptrs[n] = ptr;
                        offsets[n] = offset;
                    }

                    self.intra_iter =
                        IntraArchetypeIter::new(archetype.entities.len(), ptrs, offsets);
                }
                ptrs @ Some(_) => return ptrs,
            }
        }
    }
}

pub enum FetchType {
    EcsId,
    Mut(EcsId),
    Immut(EcsId),
}

impl FetchType {
    pub(crate) fn make_create_ptr_fn(&self) -> fn(&Archetype, Option<EcsId>) -> (*mut u8, usize) {
        match self {
            FetchType::EcsId => |archetype, _| {
                (
                    archetype.entities.as_ptr() as *mut EcsId as *mut u8,
                    core::mem::size_of::<EcsId>(),
                )
            },
            FetchType::Immut(_) => |archetype, id| {
                let storage_idx = archetype.lookup[&id.unwrap()];
                let storage = unsafe { &*archetype.component_storages[storage_idx].1.get() };
                let size = storage.get_type_info().layout.size();
                (unsafe { storage.as_immut_ptr() as *mut u8 }, size)
            },
            FetchType::Mut(_) => |archetype, id| {
                let storage_idx = archetype.lookup[&id.unwrap()];
                let storage = unsafe { &mut *archetype.component_storages[storage_idx].1.get() };
                let size = storage.get_type_info().layout.size();
                (unsafe { storage.as_mut_ptr() }, size)
            },
        }
    }
}

pub struct DynamicQuery<'a, const N: usize> {
    world: &'a World,
    _guards: [EitherGuard<'a>; N],
    fetches: [FetchType; N],

    /// If set to true it means that some of the EcsId's used were not alive/existing
    incomplete: bool,
}

impl<'a, const N: usize> DynamicQuery<'a, N> {
    pub(crate) fn new(world: &'a World, fetches: [FetchType; N]) -> Self {
        let mut incomplete = false;

        const NONE: EitherGuard = EitherGuard::None;
        let mut guards = [NONE; N];

        for (fetch, guard) in fetches.iter().zip(guards.iter_mut()) {
            let ecs_id = match fetch {
                FetchType::EcsId => continue,
                FetchType::Immut(id) | FetchType::Mut(id) => id,
            };

            if let Some(&idx) = world.lock_lookup.get(ecs_id) {
                let lock = &world.locks[idx];
                match fetch {
                    FetchType::Mut(_) => *guard = EitherGuard::Write(lock.write().unwrap()),
                    FetchType::Immut(_) => *guard = EitherGuard::Read(lock.read().unwrap()),
                    _ => (),
                }
            } else {
                incomplete = true;
            }
        }

        Self {
            world,
            _guards: guards,
            fetches,
            incomplete,
        }
    }

    pub fn iter(&mut self) -> QueryIter<'_, impl Iterator<Item = &'_ Archetype>, N> {
        const NONE_ID: Option<EcsId> = None;
        let mut ecs_ids = [NONE_ID; N];
        for (fetch, ecs_id) in self.fetches.iter().zip(ecs_ids.iter_mut()) {
            if let FetchType::Immut(id) | FetchType::Mut(id) = fetch {
                *ecs_id = Some(*id);
            }
        }

        const DEFAULT_FN: fn(&Archetype, Option<EcsId>) -> (*mut u8, usize) = |_, _| panic!();
        let mut create_ptr = [DEFAULT_FN; N];
        for (fetch, func) in self.fetches.iter().zip(create_ptr.iter_mut()) {
            *func = fetch.make_create_ptr_fn();
        }

        let archetype_iter = if self.incomplete {
            let bit_length = 0;
            let neg_fn: fn(_) -> _ = |x: usize| !x;

            use std::convert::TryInto;
            let iters: Box<[_; N]> = vec![(self.world.entities_bitvec.data.iter(), neg_fn); N]
                .into_boxed_slice()
                .try_into()
                .unwrap();
            let iters = *iters;

            self.world.query_archetypes(iters, bit_length)
        } else {
            let identity_fn: fn(_) -> _ = |x| x;

            let mut bit_length = self.world.entities_bitvec.len as u32;
            let boxed_iters = ecs_ids
                .iter()
                .map(|id| match id {
                    None => (self.world.entities_bitvec.data.iter(), identity_fn),
                    Some(id) => {
                        let bitvec = self.world.archetype_bitset.get_bitvec(*id).unwrap();
                        if { bitvec.len as u32 } < bit_length {
                            bit_length = bitvec.len as u32;
                        }

                        (bitvec.data.iter(), identity_fn)
                    }
                })
                .collect::<Box<[_]>>();
            use std::convert::TryInto;
            let iters: Box<[_; N]> = boxed_iters.try_into().unwrap();
            let iters = *iters;

            self.world.query_archetypes(iters, bit_length)
        };

        QueryIter {
            comp_ids: ecs_ids,
            create_ptr,
            archetype_iter,
            intra_iter: IntraArchetypeIter::unit(),
        }
    }
}
