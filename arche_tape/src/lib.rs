#![allow(clippy::bool_comparison)]
#![feature(
    unsafe_block_in_unsafe_fn,
    exact_size_is_empty,
    int_bits_const,
    option_unwrap_none
)]
#![deny(unsafe_op_in_unsafe_fn)]

mod archetype_iter;

pub mod entities;
pub mod entity_builder;
pub mod world;

pub(crate) mod array_vec;
pub(crate) mod query;

#[macro_export]
macro_rules! spawn {
    (&mut $world:ident, $($c:expr),* $(,)?) => {
        $world.spawn()
            $(.with($c))*
            .build()
    };
}

pub use entities::EcsId;
pub use world::World;

mod tests;

pub mod utils {
    use std::convert::TryInto;
    use std::hash::BuildHasher;
    use std::hash::Hasher;

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
