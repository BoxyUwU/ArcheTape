#![feature(exact_size_is_empty)]
#![feature(min_const_generics)]
#![feature(const_in_array_repeat_expressions)]
#![feature(unsafe_cell_get_mut)]
#![feature(bool_to_option)]

pub use ellecs_macro::spawn;

pub mod archetype_iter;
pub mod array_vec;
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

pub fn index_twice_mut<T>(idx_1: usize, idx_2: usize, slice: &mut [T]) -> (&mut T, &mut T) {
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

pub fn index_twice<T>(idx_1: usize, idx_2: usize, slice: &[T]) -> (&T, &T) {
    if idx_1 < idx_2 {
        let (left, right) = slice.split_at(idx_2);
        (left.get(idx_1).unwrap(), right.first().unwrap())
    } else if idx_1 > idx_2 {
        let (left, right) = slice.split_at(idx_1);
        (right.first().unwrap(), left.get(idx_2).unwrap())
    } else {
        panic!()
    }
}
