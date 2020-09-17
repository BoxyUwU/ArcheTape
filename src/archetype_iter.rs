use super::world::{Archetype, World};
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::slice::{Iter, IterMut};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

pub enum RwLockEitherGuard<'a> {
    WriteGuard(RwLockWriteGuard<'a, Box<dyn Any>>),
    ReadGuard(RwLockReadGuard<'a, Box<dyn Any>>),
}

#[derive(Copy, Clone)]
pub struct Query<'a, T: QueryInfos + 'a> {
    world: &'a World,
    phantom: PhantomData<T>,
}

impl<'a, T: QueryInfos + 'a> Query<'a, T> {
    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            phantom: PhantomData,
        }
    }

    pub fn borrow(&self) -> QueryBorrow<'_, '_, T> {
        let archetypes = self.world.query_archetypes::<T>();
        let mut guards = Vec::with_capacity(16);

        for archetype in archetypes.map(|idx| self.world.archetypes.get(idx).unwrap()) {
            guards.extend(T::borrow_guards(archetype));
        }

        QueryBorrow {
            lock_guards: guards,
            phantom: PhantomData,
            phantom2: PhantomData,
        }
    }
}

pub struct QueryBorrow<'b, 'guard, T: QueryInfos + 'b> {
    lock_guards: Vec<RwLockEitherGuard<'guard>>,
    phantom: PhantomData<T>,
    phantom2: PhantomData<&'b ()>,
}

pub trait QueryInfos {
    fn arity() -> usize;
    fn type_ids() -> Vec<TypeId>;
    fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>>;
}

macro_rules! impl_query_infos {
    ($($x:ident)*) => {
        impl<'b, $($x: Borrow<'b>,)*> QueryInfos for ($($x,)*) {
            #[allow(unused, non_snake_case)]
            fn arity() -> usize {
                let mut count = 0;
                $(
                    let $x = ();
                    count += 1;
                )*
                count
            }

            fn type_ids() -> Vec<TypeId> {
                vec![$(TypeId::of::<$x::Of>(),)*]
            }

            fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
                vec![$(
                    $x::guards_from_archetype(archetype),
                )*]
            }
        }

        impl<'g: 'b, 'b, $($x: Borrow<'b>,)*> QueryBorrow<'b, 'g, ($($x,)*)> {
            pub fn into_for_each_mut<Func: FnMut(($($x,)*))>(&'b mut self, mut func: Func) {
                let arity = <($($x,)*) as QueryInfos>::arity();
                debug_assert!(self.lock_guards.len() % arity == 0);

                let mut iterators = Vec::with_capacity(self.lock_guards.len());

                for chunk in self.lock_guards.chunks_mut(arity) {
                    let iter = <($(
                        $x::Iter,
                    )*) as Iters<($($x,)*)>>::iter_from_guards(chunk);
                    let iter: ItersIterator<'_, ($($x,)*), _> = ItersIterator::new(iter);
                    iterators.push(iter);
                }

                for iter in iterators {
                    for item in iter {
                        func(item);
                    }
                }
            }
        }

        impl<'a, $($x: Borrow<'a>,)*> Iters<'a, ($($x,)*)> for ($($x::Iter,)*) {
            fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
                let mut iter = locks.iter_mut();
                let mut length = None;

                ($(
                    {
                        let guard = iter.next().unwrap();
                        let (len, iter) = <$x as Borrow<'a>>::iter_from_guard(guard);

                        // SAFETY, it's important that all iterators are the same length so that we can get_unchecked if the first iterator return Some()
                        if length.is_none() {
                            length = Some(len);
                        }
                        assert_eq!(length.unwrap(), len);

                        iter
                    },
                )*)
            }

            #[allow(non_snake_case)]
            #[inline(always)]
            fn next(&mut self) -> Option<($($x,)*)> {
                if self.0.is_empty() {
                    return None;
                } else {
                    let ($($x,)*) = self;

                    // SAFETY: is_next_some returned true which means that out iterator is Some(_).
                    // Because the length of all the iterators in Iters must be the same this means all the other iterators must return Some(_) too.
                    // See Iters::iter_from_guards
                    return Some(($(
                        unsafe { <$x as Borrow<'a>>::borrow_from_iter_unchecked($x) },
                    )*));
                }
            }

            fn new_empty() -> Self {
                ($(
                    <$x as Borrow<'a>>::iter_empty(),
                )*)
            }
        }
    };
}

impl_query_infos!(A B C D E F G H I J);
impl_query_infos!(A B C D E F G H I);
impl_query_infos!(A B C D E F G H);
impl_query_infos!(A B C D E F G);
impl_query_infos!(A B C D E F);
impl_query_infos!(A B C D E);
impl_query_infos!(A B C D);
impl_query_infos!(A B C);
impl_query_infos!(A B);
impl_query_infos!(A);

pub struct ItersIterator<'a, U: QueryInfos, T: Iters<'a, U>> {
    iter: T,
    phantom: PhantomData<&'a U>,
}

impl<'a, U: QueryInfos, T: Iters<'a, U>> Iterator for ItersIterator<'a, U, T> {
    type Item = U;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, U: QueryInfos, T: Iters<'a, U>> ItersIterator<'a, U, T> {
    pub fn new(iter: T) -> Self {
        Self {
            iter,
            phantom: PhantomData,
        }
    }
}

pub trait Iters<'a, T: QueryInfos> {
    fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self;
    fn next(&mut self) -> Option<T>;
    fn new_empty() -> Self;
}

// SAFETY: The length returned from iter_from_guards **must** be accurate as we rely on being able to call get_unchecked() if one iterator returns Some(_)
pub unsafe trait Borrow<'b>: Sized {
    type Of: 'static;
    type Iter: ExactSizeIterator;

    fn iter_from_guard<'guard: 'b>(guard: &'b mut RwLockEitherGuard<'guard>)
        -> (usize, Self::Iter);

    fn borrow_from_iter<'a>(iter: &'a mut Self::Iter) -> Option<Self>;

    unsafe fn borrow_from_iter_unchecked<'a>(iter: &'a mut Self::Iter) -> Self;

    fn guards_from_archetype<'guard>(archetype: &'guard Archetype) -> RwLockEitherGuard<'guard>;

    fn iter_empty<'a>() -> Self::Iter;
}

unsafe impl<'b, T: 'static> Borrow<'b> for &'b T {
    type Of = T;
    type Iter = Iter<'b, T>;

    fn iter_from_guard<'guard: 'b>(
        guard: &'b mut RwLockEitherGuard<'guard>,
    ) -> (usize, Self::Iter) {
        match guard {
            RwLockEitherGuard::ReadGuard(guard) => {
                let vec = guard.downcast_ref::<Vec<T>>().unwrap();
                (vec.len(), vec.iter())
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn borrow_from_iter<'a>(iter: &'a mut Self::Iter) -> Option<Self> {
        iter.next()
    }

    #[inline(always)]
    #[allow(unused_unsafe)]
    unsafe fn borrow_from_iter_unchecked<'a>(iter: &'a mut Self::Iter) -> Self {
        match iter.next() {
            Some(item) => return item,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    fn guards_from_archetype<'guard>(archetype: &'guard Archetype) -> RwLockEitherGuard<'guard> {
        RwLockEitherGuard::ReadGuard(archetype.data.get::<Vec<T>>().unwrap().guard)
    }

    fn iter_empty<'a>() -> Self::Iter {
        [].iter()
    }
}
unsafe impl<'b, T: 'static> Borrow<'b> for &'b mut T {
    type Of = T;
    type Iter = IterMut<'b, T>;

    fn iter_from_guard<'guard: 'b>(
        guard: &'b mut RwLockEitherGuard<'guard>,
    ) -> (usize, Self::Iter) {
        match guard {
            RwLockEitherGuard::WriteGuard(guard) => {
                let vec = guard.downcast_mut::<Vec<T>>().unwrap();
                (vec.len(), vec.iter_mut())
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn borrow_from_iter<'a>(iter: &'a mut Self::Iter) -> Option<Self> {
        iter.next()
    }

    #[inline(always)]
    #[allow(unused_unsafe)]
    unsafe fn borrow_from_iter_unchecked<'a>(iter: &'a mut Self::Iter) -> Self {
        match iter.next() {
            Some(item) => item,
            None => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    fn guards_from_archetype<'guard>(archetype: &'guard Archetype) -> RwLockEitherGuard<'guard> {
        RwLockEitherGuard::WriteGuard(archetype.data.get_mut::<Vec<T>>().unwrap().guard)
    }

    fn iter_empty<'a>() -> Self::Iter {
        [].iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_each_mut() {
        let mut world = World::new();

        world.spawn((10_u32, 12_u64));
        world.spawn((15_u32, 14_u64));
        world.spawn((20_u32, 16_u64));

        let query = world.query::<(&mut u32, &u64)>();
        let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
        query.borrow().into_for_each_mut(|(left, right)| {
            assert_eq!(checks.next().unwrap(), (*left, *right));
        });
        assert_eq!(checks.next(), None);
    }

    #[test]
    fn for_each_iterator() {
        let mut world = World::new();

        world.spawn((10_u32, 12_u64));
        world.spawn((15_u32, 14_u64));
        world.spawn((20_u32, 16_u64));

        let query = world.query::<(&mut u32, &u64)>();

        let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
        query
            .borrow()
            .into_for_each_mut(|(left, right)| assert_eq!((*left, *right), checks.next().unwrap()));
        assert!(checks.next().is_none());
    }

    #[test]
    fn for_each_subset_iterator() {
        let mut world = World::new();

        world.spawn((10_u32, 12_u64));
        world.spawn((15_u32, 14_u64));
        world.spawn((20_u32, 16_u64));

        let query = world.query::<(&mut u32,)>();

        let mut checks = vec![10, 15, 20].into_iter();
        query
            .borrow()
            .into_for_each_mut(|(left,)| assert_eq!(*left, checks.next().unwrap()));
        assert!(checks.next().is_none());
    }

    #[test]
    fn for_each_multi_archetype_iterator() {
        let mut world = World::new();

        world.spawn((10_u32, 12_u64));
        world.spawn((15_u32, 14_u64));
        world.spawn((20_u32, 16_u64));

        world.spawn((11_u32, 12_u64, 99_u128));
        world.spawn((16_u32, 14_u64, 99_u128));
        world.spawn((21_u32, 16_u64, 99_u128));

        let query = world.query::<(&mut u32,)>();

        let mut checks = vec![10, 15, 20, 11, 16, 21].into_iter();
        query
            .borrow()
            .into_for_each_mut(|(left,)| assert_eq!(*left, checks.next().unwrap()));
        assert!(checks.next().is_none());
    }
}
