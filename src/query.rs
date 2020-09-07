use super::anymap::{AnyMap, AnyMapBorrow, AnyMapBorrowMut};
use super::world::Archetype;
use std::any::{Any, TypeId};
use std::error::Error;
use std::marker::PhantomData;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

pub struct Query<'a, T> {
    pub borrows: Vec<RwLockEitherGuard<'a, Box<dyn Any>>>,
    phantom: PhantomData<T>,
}

macro_rules! impl_query {
    ($($x:ident) *) => {
        #[allow(non_snake_case)]
        impl<'a, $($x: Borrow),*> Query<'a, ($($x,)*)> {
            fn type_ids() -> Vec<TypeId> {
                vec![$(<$x as Borrow>::type_id(),)*]
            }

            fn new(archetype: &'a Archetype) -> Result<Query<'a, ($($x,)*)>, Box<dyn Error + 'a>> {
                let type_ids = vec![$(<$x as Borrow>::type_id(),)*];
                if type_ids != archetype.type_ids {
                    return Err("Components did not match archetype".into());
                }

                let borrows = vec![$($x::borrow_from_archetype(archetype)?,)*];

                Ok(Self {
                    borrows,
                    phantom: PhantomData,
                })
            }
        }
    };
}

impl_query!(A B C D E F G H I J);
impl_query!(A B C D E F G H I);
impl_query!(A B C D E F G H);
impl_query!(A B C D E F G);
impl_query!(A B C D E F);
impl_query!(A B C D E);
impl_query!(A B C D);
impl_query!(A B C);
impl_query!(A B);
impl_query!(A);

pub enum RwLockEitherGuard<'a, T: 'static> {
    WriteGuard(RwLockWriteGuard<'a, T>),
    ReadGuard(RwLockReadGuard<'a, T>),
}

pub trait Borrow {
    fn borrow_from_archetype(
        archetype: &Archetype,
    ) -> Result<RwLockEitherGuard<'_, Box<dyn Any>>, Box<dyn Error + '_>>;

    fn type_id() -> TypeId;
}

impl<T: 'static> Borrow for &T {
    fn borrow_from_archetype(
        archetype: &Archetype,
    ) -> Result<RwLockEitherGuard<'_, Box<dyn Any>>, Box<dyn Error + '_>> {
        let guard = archetype.data.get::<Vec<T>>()?.guard;
        Ok(RwLockEitherGuard::ReadGuard(guard))
    }

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

impl<T: 'static> Borrow for &mut T {
    fn borrow_from_archetype(
        archetype: &Archetype,
    ) -> Result<RwLockEitherGuard<'_, Box<dyn Any>>, Box<dyn Error + '_>> {
        let guard = archetype.data.get_mut::<Vec<T>>()?.guard;
        Ok(RwLockEitherGuard::WriteGuard(guard))
    }

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testttt() {
        let mut archetype = Archetype::new::<(u32, u64)>();
        archetype.add((100_u32, 120_u64)).unwrap();
        let query = Query::<'_, (&u32, &u64)>::new(&archetype).unwrap();
        match &query.borrows[0] {
            RwLockEitherGuard::ReadGuard(guard) => {
                assert!(100_u32 == guard.downcast_ref::<Vec<u32>>().unwrap()[0]);
            }
            _ => panic!(""),
        }

        match &query.borrows[1] {
            RwLockEitherGuard::ReadGuard(guard) => {
                assert!(120_u64 == guard.downcast_ref::<Vec<u64>>().unwrap()[0]);
            }
            _ => panic!(""),
        }
    }
}
