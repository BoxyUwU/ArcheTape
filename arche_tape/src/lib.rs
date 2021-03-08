#![allow(clippy::bool_comparison)]
#![feature(unsafe_block_in_unsafe_fn, exact_size_is_empty, int_bits_const)]
#![deny(unsafe_op_in_unsafe_fn)]

#[macro_export]
macro_rules! spawn {
    (&mut $world:ident, $($c:expr),* $(,)?) => {
        $world.spawn()
            $(.with($c))*
            .build()
    };
    (&mut $world:ident) => {
        $world.spawn().build()
    };
}

mod bitset_iterator;

pub mod entities;
pub mod entity_builder;
pub mod world;

pub(crate) mod array_vec;
pub(crate) mod dyn_query;
pub(crate) mod static_query;

pub use dyn_query::DynQuery;
pub use dyn_query::FetchType;
pub use entities::EcsId;
pub use static_query::EcsIds;
pub use static_query::StaticQuery;
pub use world::World;

#[cfg(test)]
mod tests {
    mod bitset_iterator;
    mod bitsetsss;
    mod dyn_query;
    mod entities;
    mod query;
    mod world;
}

pub(crate) mod utils {
    use std::hash::Hasher;
    use std::{convert::TryInto, sync::RwLockReadGuard};
    use std::{hash::BuildHasher, sync::RwLockWriteGuard};

    pub enum EitherGuard<'a> {
        Read(RwLockReadGuard<'a, ()>),
        Write(RwLockWriteGuard<'a, ()>),
        None,
    }

    #[derive(Default)]
    pub struct TypeIdHasher(u64);

    impl Hasher for TypeIdHasher {
        fn write(&mut self, bytes: &[u8]) {
            self.0 = u64::from_ne_bytes(bytes.try_into().unwrap());
        }
        fn finish(&self) -> u64 {
            self.0
        }
    }

    #[derive(Clone)]
    pub struct TypeIdHasherBuilder();

    impl BuildHasher for TypeIdHasherBuilder {
        type Hasher = TypeIdHasher;

        fn build_hasher(&self) -> Self::Hasher {
            TypeIdHasher::default()
        }
    }

    pub(crate) fn index_twice_mut<T>(
        idx_1: usize,
        idx_2: usize,
        slice: &mut [T],
    ) -> (&mut T, &mut T) {
        use std::cmp::Ordering;
        match Ord::cmp(&idx_1, &idx_2) {
            Ordering::Less => {
                let (left, right) = slice.split_at_mut(idx_2);
                (left.get_mut(idx_1).unwrap(), right.first_mut().unwrap())
            }
            Ordering::Greater => {
                let (left, right) = slice.split_at_mut(idx_1);
                (right.first_mut().unwrap(), left.get_mut(idx_2).unwrap())
            }
            Ordering::Equal => panic!(),
        }
    }
}

pub trait Component: 'static {}
impl<T: 'static> Component for T {}
