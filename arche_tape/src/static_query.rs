use crate::{utils::EitherGuard, world::Archetype, Component, EcsId, FetchType, World};
use std::{any::TypeId, marker::PhantomData};

// If we remove the 'static bound here we are required to manually annotate 'static lifetimes for StaticQuery's in
// arguments of functions even though QueryTuple has a 'static bound in its trait definition
pub struct StaticQuery<'a, Q: QueryTuple + 'static> {
    world: &'a World,
    _guards: <Q as QueryTupleGATs<'a>>::Guards,
    fetches: Option<Q::Fetches>,
    _p: PhantomData<Q>,
}

pub struct StaticQueryIter<'a, Q: QueryTuple + 'static> {
    fetches: Option<&'a Q::Fetches>,
    archetypes: <Q as QueryTupleGATs<'a>>::ArchetypeIter,
    intra_iter: IntraArchetypeIter<'a, Q>,
}

struct IntraArchetypeIter<'a, Q: QueryTuple> {
    remaining: usize,
    ptrs: Q::Ptrs,
    _p: PhantomData<(Q, &'a Archetype)>,
}

pub trait QueryTupleGATs<'a>: 'static {
    type Guards: 'a;
    type ArchetypeIter: 'a;
}
pub trait QueryTuple: Sized + for<'a> QueryTupleGATs<'a> + 'static {
    type Ptrs: Copy;
    type Fetches;

    fn new(world: &World) -> StaticQuery<Self>;
}

macro_rules! impl_query_tuple {
    ($($T:ident)* $N:literal) => {
        impl<$($T: for<'a> QueryParam<'a>),*> QueryTuple for ($($T,)*) {
            type Ptrs = [*mut u8; $N];
            type Fetches = [crate::FetchType; $N];

            fn new(world: &World) -> StaticQuery<Self> {
                StaticQuery::<($($T,)*)>::new(world)
            }
        }

        impl<'a, $($T: for<'b> QueryParam<'b>),*> QueryTupleGATs<'a> for ($($T,)*) {
            type Guards = [EitherGuard<'a>; $N];
            type ArchetypeIter = crate::world::ArchetypeIter<'a, $N>;
        }

        impl<'a, $($T: for<'b> QueryParam<'b>,)*> StaticQuery<'a, ($($T,)*)> {
            #[allow(non_snake_case)]
            pub(crate) fn new(world: &'a World) -> Self {
                let fetches = (|| {
                    Some([$(
                        $T::fetch_type(world)?,
                    )*])
                })();

                let guards = match &fetches {
                    Some([$($T,)*]) => {
                        [$(
                            match $T {
                                FetchType::Mut(id) => EitherGuard::Write(world.locks[world.lock_lookup[id]].write().unwrap()),
                                FetchType::Immut(id) => EitherGuard::Read(world.locks[world.lock_lookup[id]].read().unwrap()),
                                FetchType::EcsId => EitherGuard::None,
                            },
                        )*]
                    }
                    None => {
                        const NONE_GUARD: EitherGuard = EitherGuard::None;
                        [NONE_GUARD; $N]
                    }
                };

                Self {
                    fetches,
                    world,

                    _guards: guards,
                    _p: PhantomData,
                }
            }

            #[allow(non_snake_case)]
            pub fn get(&mut self, entity: EcsId) -> Option<($(<$T as QueryParam<'_>>::Returns,)*)> {
                if !self.world.is_alive(entity) {
                    return None;
                }
                let meta = self.world.get_entity_meta(entity)?.instance_meta.clone();
                let archetype = &self.world.archetypes[meta.archetype.0];

                assert!(meta.index < archetype.entities.len());

                let [$($T,)*] = self.fetches.as_ref()?;
                let [$(mut $T,)*] = [$($T::create_ptr(archetype, $T)?,)*];
                $(
                    $T::offset_ptr(&mut $T, meta.index);
                )*
                Some(($($T::cast_ptr($T),)*))
            }

            #[allow(unused_variables, non_snake_case)]
            pub fn iter(&mut self) -> StaticQueryIter<($($T,)*)> {
                let identity: fn(_) -> _ = |x| x;
                let archetype_iter: crate::world::ArchetypeIter<$N> = match &self.fetches {
                    Some([$($T,)*]) => {
                        let mut bitlength = self.world.entities_bitvec.len as u32;
                        let iters = [$(
                            match $T {
                                FetchType::EcsId => {
                                    (self.world.entities_bitvec.data.iter(), identity)
                                }
                                FetchType::Immut(id) | FetchType::Mut(id) => {
                                    let bitvec = self.world.archetype_bitset.get_bitvec(*id).unwrap();
                                    bitlength = u32::min(bitlength, bitvec.len as u32);
                                    (bitvec.data.iter(), identity)
                                }
                            },
                        )*];
                        self.world.query_archetypes(iters, bitlength)
                    }
                    None => {
                        let iters: [_; $N] =
                            [$(
                                ({
                                    let $T = ();
                                    self.world.entities_bitvec.data.iter()
                                }, identity ),
                            )*];
                        self.world.query_archetypes(iters, 0)
                    }
                };

                StaticQueryIter {
                    fetches: self.fetches.as_ref(),
                    archetypes: archetype_iter,
                    intra_iter: IntraArchetypeIter::<($($T,)*)>::unit(),
                }
            }
        }

        impl<'a, $($T: for<'b> QueryParam<'b>,)*> Iterator for StaticQueryIter<'a, ($($T,)*)> {
                type Item = ($(<$T as QueryParam<'a>>::Returns,)*);

                #[allow(non_snake_case, unused_assignments)]
                #[inline(always)]
                fn next(&mut self) -> Option<Self::Item> {
                    loop {
                        match self.intra_iter.next() {
                            Some([$($T,)*]) => {
                                return Some((
                                    $($T::cast_ptr($T),)*
                                ));
                            }
                            None => {
                                let archetype = self.archetypes.next()?;
                                let mut ptrs = [std::ptr::null_mut::<u8>(); $N];

                                let fetches = self.fetches.as_ref().unwrap();
                                let mut n = 0;
                                $({
                                    let fetch = &fetches[n];
                                    let ptr = $T::create_ptr(archetype, fetch).unwrap();
                                    ptrs[n] = ptr;
                                    n += 1;
                                })*

                                self.intra_iter = IntraArchetypeIter {
                                    remaining: archetype.entities.len(),
                                    ptrs,
                                    _p: PhantomData,
                                };
                            },
                        }
                    }
                }
        }

        impl<'a, $($T: for<'b> QueryParam<'b>,)*> IntraArchetypeIter<'a, ($($T,)*)> {
            fn unit() -> Self {
                Self {
                    remaining: 0,
                    ptrs: [0x0 as _; $N],
                    _p: PhantomData,
                }
            }
        }

        impl<'a, $($T: for<'b> QueryParam<'b>,)*> Iterator for IntraArchetypeIter<'a, ($($T,)*)> {
            type Item = [*mut u8; $N];

            #[allow(unused_assignments)]
            fn next(&mut self) -> Option<Self::Item> {
                if self.remaining == 0 {
                    return None;
                }
                self.remaining -= 1;

                let ptrs = self.ptrs;

                let mut n = 0;
                $(
                    $T::offset_ptr(&mut self.ptrs[n], 1);
                    n += 1;
                )*

                Some(ptrs)
            }
        }
    };
}

impl_query_tuple!(A B C D E F G H J K 10);
impl_query_tuple!(A B C D E F G H J 9);
impl_query_tuple!(A B C D E F G H 8);
impl_query_tuple!(A B C D E F G 7);
impl_query_tuple!(A B C D E F 6);
impl_query_tuple!(A B C D E 5);
impl_query_tuple!(A B C D 4);
impl_query_tuple!(A B C 3);
impl_query_tuple!(A B 2);
impl_query_tuple!(A 1);

pub trait QueryParam<'a>: 'static {
    type Returns: 'a;

    fn fetch_type(world: &World) -> Option<FetchType>;
    fn create_ptr(archetype: &Archetype, fetch: &FetchType) -> Option<*mut u8>;
    fn offset_ptr(ptr: &mut *mut u8, elements: usize);
    fn cast_ptr(ptr: *mut u8) -> Self::Returns;
}

impl<'a, T: Component> QueryParam<'a> for &'static mut T {
    type Returns = &'a mut T;

    fn fetch_type(world: &World) -> Option<FetchType> {
        let id = *world.type_id_to_ecs_id.get(&TypeId::of::<T>())?;
        Some(FetchType::Mut(id))
    }

    fn create_ptr(archetype: &Archetype, fetch: &FetchType) -> Option<*mut u8> {
        let &storage_idx = archetype.comp_lookup.get(&fetch.get_id().unwrap())?;
        let storage = unsafe { &mut *archetype.component_storages[storage_idx].1.get() };
        unsafe { Some(storage.as_mut_ptr()) }
    }

    fn offset_ptr(ptr: &mut *mut u8, elements: usize) {
        *ptr = unsafe { ((*ptr) as *mut T).add(elements) as *mut u8 };
    }

    fn cast_ptr(ptr: *mut u8) -> Self::Returns {
        unsafe { &mut *(ptr as *mut T) }
    }
}
impl<'a, T: Component> QueryParam<'a> for &'static T {
    type Returns = &'a T;

    fn fetch_type(world: &World) -> Option<FetchType> {
        let id = *world.type_id_to_ecs_id.get(&TypeId::of::<T>())?;
        Some(FetchType::Immut(id))
    }

    fn create_ptr(archetype: &Archetype, fetch: &FetchType) -> Option<*mut u8> {
        let &storage_idx = archetype.comp_lookup.get(&fetch.get_id().unwrap())?;
        let storage = unsafe { &*archetype.component_storages[storage_idx].1.get() };
        unsafe { Some(storage.as_immut_ptr() as *mut u8) }
    }

    fn offset_ptr(ptr: &mut *mut u8, elements: usize) {
        *ptr = unsafe { ((*ptr) as *mut T).add(elements) as *mut u8 };
    }

    fn cast_ptr(ptr: *mut u8) -> Self::Returns {
        unsafe { &*(ptr as *mut T) }
    }
}

pub struct EcsIds;
impl<'a> QueryParam<'a> for EcsIds {
    type Returns = EcsId;

    fn fetch_type(_: &World) -> Option<FetchType> {
        Some(FetchType::EcsId)
    }

    fn create_ptr(archetype: &Archetype, _: &FetchType) -> Option<*mut u8> {
        Some(archetype.entities.as_ptr() as *mut EcsId as *mut u8)
    }

    fn offset_ptr(ptr: &mut *mut u8, elements: usize) {
        *ptr = unsafe { ((*ptr) as *mut EcsId).add(elements) as *mut u8 };
    }

    fn cast_ptr(ptr: *mut u8) -> Self::Returns {
        unsafe { *(ptr as *mut EcsId) }
    }
}
