use std::any::Any;
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

pub struct QueryBorrow<'guard, T: QueryInfos> {
    lock_guards: Vec<RwLockEitherGuard<'guard>>,
    phantom: PhantomData<T>,
}

pub trait QueryInfos {}

macro_rules! impl_query_infos {
    ($($x:ident)*) => {
        impl<'b, $($x: Borrow<'b>,)*> QueryInfos for ($($x,)*) { }

        impl<'g: 'b, 'b, $($x: Borrow<'b>,)*> QueryBorrow<'g, ($($x,)*)> { }

        impl<'a, $($x: Borrow<'a>,)*> Iters<'a, ($($x,)*)> for ($(EitherIter<'a, $x::Of>,)*) {
            fn iter_from_guards<'guard: 'a>(locks: &'a mut Vec<RwLockEitherGuard<'guard>>) -> ($(EitherIter<'a, $x::Of>,)*) {
                let mut iter = locks.iter_mut();
                ($(
                    {
                        let guard = iter.next().unwrap();
                        <$x as Borrow<'a>>::either_iter_from_guard(guard)
                    },
                )*)
            }

            #[allow(non_snake_case)]
            fn next(&mut self) -> Option<($($x,)*)> {
                let ($($x,)*) = self;

                Some(($(
                    <$x as Borrow<'a>>::borrow_from_iter($x)?,
                )*))
            }
        }
    };
}

pub trait Iters<'a, T: QueryInfos> {
    fn iter_from_guards<'guard: 'a>(locks: &'a mut Vec<RwLockEitherGuard<'guard>>) -> Self;
    fn next(&mut self) -> Option<T>;
}

pub struct QueryIter<'a, T: QueryInfos, U: Iters<'a, T>> {
    iters: U,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: QueryInfos, U: Iters<'a, T>> QueryIter<'a, T, U> {
    pub fn from_borrows<'guard: 'a>(
        borrows: &'a mut QueryBorrow<'guard, T>,
    ) -> QueryIter<'a, T, U> {
        let iters = <U as Iters<'a, T>>::iter_from_guards(&mut borrows.lock_guards);
        QueryIter {
            iters,
            phantom: PhantomData,
        }
    }
}

impl<'a, T: QueryInfos, U: Iters<'a, T>> Iterator for QueryIter<'a, T, U> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iters.next()
    }
}

pub trait Borrow<'b>: Sized {
    type Of;

    fn either_iter_from_guard<'a, 'guard: 'a>(
        guard: &'a mut RwLockEitherGuard<'guard>,
    ) -> EitherIter<'a, Self::Of>;

    fn borrow_from_iter<'a>(iter: &'a mut EitherIter<'b, Self::Of>) -> Option<Self>;
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

    fn borrow_from_iter<'a>(iter: &'a mut EitherIter<'b, Self::Of>) -> Option<Self> {
        match iter {
            EitherIter::Mut(iter) => iter.next(),
            _ => unreachable!(),
        }
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
    use super::super::*;
    use super::*;
    use world::Archetype;

    #[test]
    fn testttt() {
        let mut archetype = Archetype::new::<(u32, u64)>();
        archetype.add((10_u32, 12_u64)).unwrap();
        archetype.add((15_u32, 14_u64)).unwrap();
        archetype.add((20_u32, 16_u64)).unwrap();

        let mut query_borrow = QueryBorrow::<'_, (&mut u32, &u64)> {
            lock_guards: vec![
                RwLockEitherGuard::WriteGuard(archetype.data.get_mut::<Vec<u32>>().unwrap().guard),
                RwLockEitherGuard::ReadGuard(archetype.data.get::<Vec<u64>>().unwrap().guard),
            ],
            phantom: PhantomData,
        };

        let query_iter =
            QueryIter::<'_, _, (EitherIter<_>, EitherIter<_>)>::from_borrows(&mut query_borrow);

        for (n, (left, right)) in query_iter.enumerate() {
            println!("{:?}, {:?}", left, right);

            if n == 0 {
                assert!(*left == 10);
                assert!(*right == 12);
            } else if n == 1 {
                assert!(*left == 15);
                assert!(*right == 14);
            } else if n == 2 {
                assert!(*left == 20);
                assert!(*right == 16);
            } else {
                panic!("awd");
            }
        }
    }

    #[test]
    fn testttttttt() {
        let mut archetype = Archetype::new::<(u32, u64)>();
        archetype.add((10_u32, 12_u64)).unwrap();
        archetype.add((15_u32, 14_u64)).unwrap();
        archetype.add((20_u32, 16_u64)).unwrap();

        let mut query_borrow = QueryBorrow::<'_, (&mut u32,)> {
            lock_guards: vec![RwLockEitherGuard::WriteGuard(
                archetype.data.get_mut::<Vec<u32>>().unwrap().guard,
            )],
            phantom: PhantomData,
        };

        let query_iter = QueryIter::<'_, _, (EitherIter<_>,)>::from_borrows(&mut query_borrow);

        for (n, (left,)) in query_iter.enumerate() {
            println!("{:?}", left);

            if n == 0 {
                assert!(*left == 10);
            } else if n == 1 {
                assert!(*left == 15);
            } else if n == 2 {
                assert!(*left == 20);
            } else {
                panic!("awd");
            }
        }
    }
}
