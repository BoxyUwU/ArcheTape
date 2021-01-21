use crate::{utils::EitherGuard, world::Archetype, Component, EcsId, FetchType, World};
use std::{any::TypeId, marker::PhantomData};

pub struct StaticQuery<'a, Q: QueryTuple + GuardAssocType<'a>> {
    world: &'a World,
    _guards: Q::Guards,
    fetches: Q::Fetches,
    // Signifies that some of the EcsIds being fetched do not exist
    incomplete: bool,
    _p: PhantomData<Q>,
}

pub struct QueryIter<'a, Q: QueryTuple, I: Iterator<Item = &'a Archetype>> {
    fetches: &'a Q::Fetches,
    archetypes: I,
    intra_iter: IntraArchetypeIter<'a, Q>,
}

struct IntraArchetypeIter<'a, Q: QueryTuple> {
    remaining: usize,
    ptrs: Q::Ptrs,
    _p: PhantomData<(Q, &'a Archetype)>,
}

pub trait GuardAssocType<'a> {
    type Guards: 'a;
}
pub trait QueryTuple {
    type Ptrs: Copy;
    type Fetches;

    fn new<'a>(world: &'a World) -> StaticQuery<'a, Self>
    where
        Self: GuardAssocType<'a> + Sized;

    fn iter<'a, 'b>(
        q: &'a mut StaticQuery<'b, Self>,
    ) -> QueryIter<'a, Self, Box<dyn Iterator<Item = &'a Archetype> + 'a>>
    where
        Self: GuardAssocType<'b> + Sized;
}

macro_rules! impl_query_tuple {
    ($($T:ident)* $N:literal) => {
        #[allow(unused, non_snake_case)]
        impl<$($T: for<'a> QueryParam<'a>),*> QueryTuple for ($($T,)*) {
            type Ptrs = [*mut u8; $N];
            type Fetches = [Option<crate::FetchType>; $N];

            fn new<'a>(world: &'a World) -> StaticQuery<'a, Self> where Self: GuardAssocType<'a> + Sized {
                StaticQuery::<($($T,)*)>::new(world)
            }

            fn iter<'a, 'b>(
                q: &'a mut StaticQuery<'b, Self>,
            ) -> QueryIter<'a, Self, Box<dyn Iterator<Item = &'a Archetype> + 'a>>
            where
                Self: GuardAssocType<'b> + Sized {
                    q.iter()
                }
        }

        impl<'a, $($T: for<'b> QueryParam<'b>),*> GuardAssocType<'a> for ($($T,)*) {
            type Guards = [EitherGuard<'a>; $N];
        }

        impl<'a, $($T: for<'b> QueryParam<'b>,)*> StaticQuery<'a, ($($T,)*)> {
            pub(crate) fn new(world: &'a World) -> Self {
                let mut incomplete = false;
                let fetches = [$(
                    $T::fetch_type(world),
                )*];

                if let Some(_) = fetches.iter().find(|f| f.is_none()) {
                    incomplete = true;
                }

                let guards: [EitherGuard; $N] = if incomplete == false {
                    let guards = fetches.iter().map(|f| match f {
                        Some(FetchType::Mut(id)) => EitherGuard::Write(world.locks[world.lock_lookup[id]].write().unwrap()),
                        Some(FetchType::Immut(id)) => EitherGuard::Read(world.locks[world.lock_lookup[id]].read().unwrap()),
                        Some(FetchType::EcsId) => EitherGuard::None,
                        None => EitherGuard::None,
                    }).collect::<Box<[EitherGuard]>>();

                    match std::convert::TryInto::<Box<[EitherGuard; $N]>>::try_into(guards) {
                        Ok(boxed) => *boxed,
                        Err(_) => unreachable!(),
                    }
                } else {
                    const NONE: EitherGuard = EitherGuard::None;
                    [NONE; $N]
                };

                Self {
                    _guards: guards,
                    fetches,
                    world,
                    incomplete,
                    _p: PhantomData,
                }
            }

            pub fn iter(&mut self) -> QueryIter<($($T,)*), Box<dyn Iterator<Item = &Archetype> + '_>> {
                let archetype_iter = if self.incomplete {
                    let identity: fn(_) -> _ = |x| x;
                    use std::convert::TryInto;
                    let iters: Box<[_; $N]> = vec![(self.world.entities_bitvec.data.iter(), identity); $N].into_boxed_slice().try_into().unwrap();
                    let iters = *iters;
                    self.world.query_archetypes(iters, 0)
                } else {
                    let identity_fn: fn(_) -> _ = |x| x;

                    let mut bit_length = self.world.entities_bitvec.len as u32;
                    let boxed_iters = self.fetches
                        .iter()
                        .map(|f| match f {
                            Some(FetchType::EcsId) => {
                                (self.world.entities_bitvec.data.iter(), identity_fn)
                            }
                            Some(FetchType::Immut(id)) | Some(FetchType::Mut(id)) => {
                                let bitvec = self.world.archetype_bitset.get_bitvec(*id).unwrap();
                                if (bitvec.len as u32) < bit_length {
                                    bit_length = bitvec.len as u32;
                                }

                                (bitvec.data.iter(), identity_fn)
                            }
                            None => unreachable!(),
                        })
                        .collect::<Box<[_]>>();
                    use std::convert::TryInto;
                    let iters: Box<[_; $N]> = boxed_iters.try_into().unwrap();
                    let iters = *iters;

                    self.world.query_archetypes(iters, bit_length)
                };

                QueryIter {
                    fetches: &self.fetches,
                    archetypes: Box::new(archetype_iter),
                    intra_iter: IntraArchetypeIter::<($($T,)*)>::unit(),
                }
            }
        }

        impl<'a, $($T: for<'b> QueryParam<'b>,)* I> Iterator for QueryIter<'a, ($($T,)*), I>
            where I: Iterator<Item = &'a Archetype> {
                type Item = ($(<$T as QueryParam<'a>>::Returns,)*);

                #[allow(unused_assignments)]
                fn next(&mut self) -> Option<Self::Item> {
                    loop {
                        match self.intra_iter.next() {
                            Some(ptrs) => {
                                let mut n = 0;
                                return Some(($({
                                    n+=1;
                                    $T::cast_ptr(ptrs[n-1])
                                },)*));
                            }
                            None => {
                                let archetype = self.archetypes.next()?;
                                let mut ptrs = [0x0 as *mut u8; $N];

                                let mut n = 0;
                                $({
                                    let fetch = (&self.fetches[n]).as_ref().unwrap();
                                    let ptr = $T::create_ptr(archetype, fetch);
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
                    $T::next_ptr(&mut self.ptrs[n]);
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

pub trait QueryParam<'a> {
    type Returns: 'a;

    fn fetch_type(world: &World) -> Option<FetchType>;
    fn create_ptr(archetype: &Archetype, fetch: &FetchType) -> *mut u8;
    fn next_ptr(ptr: &mut *mut u8);
    fn cast_ptr(ptr: *mut u8) -> Self::Returns;
}

impl<'a, T: Component> QueryParam<'a> for &'static mut T {
    type Returns = &'a mut T;

    fn fetch_type(world: &World) -> Option<FetchType> {
        let id = *world.type_id_to_ecs_id.get(&TypeId::of::<T>())?;
        Some(FetchType::Mut(id))
    }

    fn create_ptr(archetype: &Archetype, fetch: &FetchType) -> *mut u8 {
        let storage_idx = archetype.lookup[&fetch.get_id().unwrap()];
        let storage = unsafe { &mut *archetype.component_storages[storage_idx].1.get() };
        unsafe { storage.as_mut_ptr() }
    }

    fn next_ptr(ptr: &mut *mut u8) {
        *ptr = unsafe { ((*ptr) as *mut T).add(1) as *mut u8 };
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

    fn create_ptr(archetype: &Archetype, fetch: &FetchType) -> *mut u8 {
        let storage_idx = archetype.lookup[&fetch.get_id().unwrap()];
        let storage = unsafe { &*archetype.component_storages[storage_idx].1.get() };
        unsafe { storage.as_immut_ptr() as *mut u8 }
    }

    fn next_ptr(ptr: &mut *mut u8) {
        *ptr = unsafe { ((*ptr) as *mut T).add(1) as *mut u8 };
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

    fn create_ptr(archetype: &Archetype, _: &FetchType) -> *mut u8 {
        archetype.entities.as_ptr() as *mut EcsId as *mut u8
    }

    fn next_ptr(ptr: &mut *mut u8) {
        *ptr = unsafe { ((*ptr) as *mut EcsId).add(1) as *mut u8 };
    }

    fn cast_ptr(ptr: *mut u8) -> Self::Returns {
        unsafe { *(ptr as *mut EcsId) }
    }
}
