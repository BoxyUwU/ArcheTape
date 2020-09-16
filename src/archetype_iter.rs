use super::world::{Archetype, World};
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::slice::{Iter, IterMut};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

pub enum EitherIter<'a, T> {
    Mut(IterMut<'a, T>),
    Immut(Iter<'a, T>),
}

pub enum RwLockEitherGuard<'a> {
    WriteGuard(RwLockWriteGuard<'a, Box<dyn Any>>),
    ReadGuard(RwLockReadGuard<'a, Box<dyn Any>>),
}

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
        let mut guards = vec![];

        for archetype in archetypes
            .into_iter()
            .map(|idx| self.world.archetypes.get(idx).unwrap())
        {
            for guard in T::borrow_guards(archetype).into_iter() {
                guards.push(guard)
            }
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
            pub fn iter(&'b mut self) -> QueryIter<'b, ($($x,)*), ($(EitherIter<'b, $x::Of>,)*)> {
                QueryIter::<'b, ($($x,)*), ($(EitherIter<'b, $x::Of>,)*)>::from_borrow(self)
            }
        }

        impl<'g: 'b, 'b, $($x: Borrow<'b>,)*> IntoIterator for &'b mut QueryBorrow<'b, 'g, ($($x,)*)> {
            type Item = ($($x,)*);
            type IntoIter = QueryIter<'b, Self::Item, ($(EitherIter<'b, $x::Of>,)*)>;

            fn into_iter(self) -> QueryIter<'b, Self::Item, ($(EitherIter<'b, $x::Of>,)*)> {
                QueryBorrow::<'b, 'g, ($($x,)*)>::iter(self)
            }
        }

        impl<'a, $($x: Borrow<'a>,)*> Iters<'a, ($($x,)*)> for ($(EitherIter<'a, $x::Of>,)*) {
            fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> ($(EitherIter<'a, $x::Of>,)*) {
                let mut iter = locks.iter_mut();
                ($(
                    {
                        let guard = iter.next().unwrap();
                        <$x as Borrow<'a>>::either_iter_from_guard(guard)
                    },
                )*)
            }

            #[allow(non_snake_case)]
            #[inline(always)]
            fn next(&mut self) -> Option<($($x,)*)> {
                let ($($x,)*) = self;

                Some(($(
                    <$x as Borrow<'a>>::borrow_from_iter($x)?,
                )*))
            }

            fn new_empty() -> Self {
                ($(
                    <$x as Borrow<'a>>::either_iter_empty(),
                )*)
            }
        }
    };
}

pub trait Iters<'a, T: QueryInfos> {
    fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self;
    fn next(&mut self) -> Option<T>;
    fn new_empty() -> Self;
}

pub struct QueryIter<'a, T: QueryInfos, U: Iters<'a, T>> {
    iters: Vec<U>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: QueryInfos, U: Iters<'a, T>> QueryIter<'a, T, U> {
    pub fn from_borrow<'guard: 'a>(
        borrows: &'a mut QueryBorrow<'a, 'guard, T>,
    ) -> QueryIter<'a, T, U> {
        let mut iters = vec![];
        let arity = T::arity();

        assert!(borrows.lock_guards.len() % arity == 0);
        for chunk in borrows.lock_guards.chunks_exact_mut(arity) {
            iters.push(<U as Iters<'a, T>>::iter_from_guards(chunk));
        }

        QueryIter {
            iters,
            phantom: PhantomData,
        }
    }
}

impl<'a, T: QueryInfos, U: Iters<'a, T>> Iterator for QueryIter<'a, T, U> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let iter = self.iters.last_mut()?;
            match iter.next() {
                Some(item) => return Some(item),
                None => self.iters.pop(),
            };
        }
    }
}

pub trait Borrow<'b>: Sized {
    type Of: 'static;

    fn either_iter_from_guard<'a, 'guard: 'a>(
        guard: &'a mut RwLockEitherGuard<'guard>,
    ) -> EitherIter<'a, Self::Of>;

    fn borrow_from_iter<'a>(iter: &'a mut EitherIter<'b, Self::Of>) -> Option<Self>;

    fn guards_from_archetype<'guard>(archetype: &'guard Archetype) -> RwLockEitherGuard<'guard>;

    fn either_iter_empty<'a>() -> EitherIter<'a, Self::Of>;
}

impl<'b, T: 'static> Borrow<'b> for &'b T {
    type Of = T;

    fn either_iter_from_guard<'a, 'guard: 'a>(
        guard: &'a mut RwLockEitherGuard<'guard>,
    ) -> EitherIter<'a, Self::Of> {
        match guard {
            RwLockEitherGuard::ReadGuard(guard) => {
                EitherIter::Immut(guard.downcast_ref::<Vec<T>>().unwrap().iter())
            }
            _ => unreachable!(),
        }
    }

    fn borrow_from_iter<'a>(iter: &'a mut EitherIter<'b, Self::Of>) -> Option<Self> {
        match iter {
            EitherIter::Immut(iter) => iter.next(),
            _ => unreachable!(),
        }
    }

    fn guards_from_archetype<'guard>(archetype: &'guard Archetype) -> RwLockEitherGuard<'guard> {
        RwLockEitherGuard::ReadGuard(archetype.data.get::<Vec<T>>().unwrap().guard)
    }

    fn either_iter_empty<'a>() -> EitherIter<'a, Self::Of> {
        EitherIter::Immut([].iter())
    }
}
impl<'b, T: 'static> Borrow<'b> for &'b mut T {
    type Of = T;

    fn either_iter_from_guard<'a, 'guard: 'a>(
        guard: &'a mut RwLockEitherGuard<'guard>,
    ) -> EitherIter<'a, Self::Of> {
        match guard {
            RwLockEitherGuard::WriteGuard(guard) => {
                EitherIter::Mut(guard.downcast_mut::<Vec<T>>().unwrap().iter_mut())
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn borrow_from_iter<'a>(iter: &'a mut EitherIter<'b, Self::Of>) -> Option<Self> {
        match iter {
            EitherIter::Mut(iter) => iter.next(),
            _ => unreachable!(),
        }
    }

    fn guards_from_archetype<'guard>(archetype: &'guard Archetype) -> RwLockEitherGuard<'guard> {
        RwLockEitherGuard::WriteGuard(archetype.data.get_mut::<Vec<T>>().unwrap().guard)
    }

    fn either_iter_empty<'a>() -> EitherIter<'a, Self::Of> {
        EitherIter::Mut([].iter_mut())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterator() {
        let mut world = World::new();

        world.spawn((10_u32, 12_u64));
        world.spawn((15_u32, 14_u64));
        world.spawn((20_u32, 16_u64));

        let query = world.query::<(&mut u32, &u64)>();

        let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
        for (left, right) in &mut query.borrow() {
            assert_eq!((*left, *right), checks.next().unwrap());
        }
        assert!(checks.next().is_none());
    }

    #[test]
    fn subset_iterator() {
        let mut world = World::new();

        world.spawn((10_u32, 12_u64));
        world.spawn((15_u32, 14_u64));
        world.spawn((20_u32, 16_u64));

        let query = world.query::<(&mut u32,)>();

        let mut checks = vec![10, 15, 20].into_iter();
        for (left,) in &mut query.borrow() {
            assert_eq!(*left, checks.next().unwrap());
        }
        assert!(checks.next().is_none());
    }

    #[test]
    fn multi_archetype_iterator() {
        let mut world = World::new();

        world.spawn((10_u32, 12_u64));
        world.spawn((15_u32, 14_u64));
        world.spawn((20_u32, 16_u64));

        world.spawn((11_u32, 12_u64, 99_u128));
        world.spawn((16_u32, 14_u64, 99_u128));
        world.spawn((21_u32, 16_u64, 99_u128));

        let query = world.query::<(&mut u32,)>();

        let mut checks = vec![11, 16, 21, 10, 15, 20].into_iter();
        for (left,) in &mut query.borrow() {
            assert_eq!(*left, checks.next().unwrap());
        }
        assert!(checks.next().is_none());
    }
}
