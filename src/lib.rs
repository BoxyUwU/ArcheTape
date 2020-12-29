#![feature(exact_size_is_empty)]
#![feature(min_const_generics)]
#![feature(const_in_array_repeat_expressions)]
#![feature(unsafe_block_in_unsafe_fn)]
#![feature(unsafe_cell_get_mut)]
#![feature(option_unwrap_none)]
#![feature(option_expect_none)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod entities;
pub mod entity_builder;
pub mod world;

pub(crate) mod archetype_iter;
pub(crate) mod array_vec;
pub(crate) mod untyped_vec;

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
        if idx_1 < idx_2 {
            let (left, right) = slice.split_at_mut(idx_2);
            (left.get_mut(idx_1).unwrap(), right.first_mut().unwrap())
        } else if idx_1 > idx_2 {
            let (left, right) = slice.split_at_mut(idx_1);
            (right.first_mut().unwrap(), left.get_mut(idx_2).unwrap())
        } else {
            panic!()
        }
    }
}
