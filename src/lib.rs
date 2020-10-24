#![feature(exact_size_is_empty)]
#![feature(min_const_generics)]
#![feature(const_in_array_repeat_expressions)]
#![feature(unsafe_cell_get_mut)]
#![feature(bool_to_option)]

pub use ellecs_macro::spawn;

pub mod archetype_iter;
pub mod entities;
pub mod entity_builder;
pub mod sparse_array;
pub mod untyped_vec;
pub mod world;

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
