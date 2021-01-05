pub use bitset_iterator::BitsetIterator;

mod bitset_iterator {

    use std::{borrow::BorrowMut, marker::PhantomData, slice::Iter};
    pub struct BitsetIterator<'a, Iters: BorrowMut<[(Iter<'a, usize>, fn(usize) -> usize)]>> {
        phantom: PhantomData<&'a [usize]>,
        iters: Iters,

        bit_length: u32,
        index: usize,

        bits_remaining: u32,
        current_bits: usize,
    }

    impl<'a, Iters: BorrowMut<[(Iter<'a, usize>, fn(usize) -> usize)]>> BitsetIterator<'a, Iters> {
        pub(crate) fn new(iters: Iters, bit_length: u32) -> Self {
            Self {
                phantom: PhantomData,
                iters,

                bit_length,
                index: 0,

                bits_remaining: 0,
                current_bits: 0,
            }
        }
    }

    impl<'a, Iters: BorrowMut<[(Iter<'a, usize>, fn(usize) -> usize)]>> Iterator
        for BitsetIterator<'a, Iters>
    {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if self.bits_remaining == 0 {
                    // We have to initialise filtered to a proper value so we hand write the first iteration of the loop :(
                    let mut iter = self.iters.borrow_mut().iter_mut();
                    let (first_iter, first_map) = iter.next()?;
                    let mut filtered: usize = first_map(*first_iter.next()?);

                    for (iter, map) in iter {
                        filtered &= map(*iter.next()?);
                    }

                    self.bits_remaining = usize::BITS;
                    self.current_bits = filtered;
                }

                let zeros = self.current_bits.trailing_zeros();

                // Right shifting leaves zeros in its place so we know we've run out of ones when we hit usize::BITS Number of zeros
                if zeros == usize::BITS {
                    self.index += self.bits_remaining as usize;
                    self.bits_remaining = 0;
                    continue;
                }

                self.bits_remaining -= zeros + 1;
                self.current_bits >>= zeros + 1;
                self.index += zeros as usize + 1;

                if self.index > self.bit_length as usize {
                    // Hack but make sure that calling next() after None is returned continues to return None by
                    // setting an empty iterator and making the subsequent Next calls try and call next() on it
                    self.current_bits = 0;
                    // Indexing by 0 should always be fine since an empty slice will never make it this far
                    self.iters.borrow_mut()[0].0 = [].iter();
                    return None;
                }

                return Some(self.index - 1);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::BitsetIterator;

        #[test]
        fn empty_bitset() {
            let map: fn(_) -> _ = |x| x;
            let data = vec![];
            let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], 0);

            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn single_bitset() {
            let map: fn(_) -> _ = |x| x;
            let data = vec![0b0000_1011];
            let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], usize::BITS);

            assert_eq!(bitset_iter.next(), Some(0));
            assert_eq!(bitset_iter.next(), Some(1));
            assert_eq!(bitset_iter.next(), Some(3));
            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn gapped_bitset() {
            let map: fn(_) -> _ = |x| x;
            let data = vec![0, 0b101];
            let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], usize::BITS * 2);

            assert_eq!(bitset_iter.next(), Some(64));
            assert_eq!(bitset_iter.next(), Some(66));
            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn triple_bitset() {
            let map: fn(_) -> _ = |x| x;
            let data1 = vec![0b1010_1011];
            let data2 = vec![0b0110_1110];
            let data3 = vec![0b1110_0110];

            let mut bitset_iter = BitsetIterator::new(
                [
                    (data1.iter(), map),
                    (data2.iter(), map),
                    (data3.iter(), map),
                ],
                usize::BITS,
            );

            assert_eq!(bitset_iter.next(), Some(1));
            assert_eq!(bitset_iter.next(), Some(5));
            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn map_bitset() {
            let invert_map: fn(usize) -> _ = |x| !x;
            let map: fn(_) -> _ = |x| x;

            let data1 = vec![0b1010_1011];
            let data2 = vec![0b0111_0110];

            let mut bitset_iter = BitsetIterator::new(
                [(data1.iter(), map), (data2.iter(), invert_map)],
                usize::BITS,
            );

            assert_eq!(bitset_iter.next(), Some(0));
            assert_eq!(bitset_iter.next(), Some(3));
            assert_eq!(bitset_iter.next(), Some(7));
            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn all_ones() {
            let map: fn(_) -> _ = |x| x;
            let data = vec![usize::MAX];
            let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], usize::BITS);

            for n in 0..usize::BITS {
                assert_eq!(bitset_iter.next(), Some(n as _));
            }
            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn bit_length() {
            let map: fn(_) -> _ = |x| x;
            let data = vec![0b101];
            let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], 2);

            assert_eq!(bitset_iter.next(), Some(0));
            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn long_bit_length() {
            let map: fn(_) -> _ = |x| x;
            let data = vec![0b0, usize::MAX];
            let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], usize::BITS * 2);

            for n in 0..usize::BITS {
                let n = n + 64;
                assert_eq!(bitset_iter.next(), Some(n as _));
            }
            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn incorrect_bit_length() {
            let map: fn(_) -> _ = |x| x;
            let data = vec![0b101];
            let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], 2000);

            assert_eq!(bitset_iter.next(), Some(0));
            assert_eq!(bitset_iter.next(), Some(2));
            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn returns_none_continuously_incorrect_bit_length() {
            let map: fn(_) -> _ = |x| x;
            let data = vec![0b101];
            let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], 2000);

            assert_eq!(bitset_iter.next(), Some(0));
            assert_eq!(bitset_iter.next(), Some(2));
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn returns_none_continuously_bit_length() {
            let map: fn(_) -> _ = |x| x;
            let data = vec![0b101];
            let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], 3);

            assert_eq!(bitset_iter.next(), Some(0));
            assert_eq!(bitset_iter.next(), Some(2));
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
        }

        #[test]
        fn returns_none_continuously() {
            let map: fn(_) -> _ = |x| x;
            let data = vec![usize::MAX];
            // the iterator will end because of there being no more iterator left not because of the bit_length
            let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], usize::BITS);

            for n in 0..usize::BITS {
                assert_eq!(bitset_iter.next(), Some(n as _));
            }

            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
            bitset_iter.next().unwrap_none();
        }
    }
}

pub struct Bitvec {
    data: Vec<usize>,
    /// Length in bits of the bitvec
    len: usize,
}

impl Bitvec {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
        }
    }

    fn with_capacity(bit_cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(bit_cap / usize::BITS as usize),
            len: 0,
        }
    }

    fn get_bit(&self, index: usize) -> Option<bool> {
        if index >= self.len {
            return None;
        }

        let data_idx = index / usize::BITS as usize;
        let bit_idx = index % usize::BITS as usize;

        Some(((self.data[data_idx] >> bit_idx) & 1) == 1)
    }

    fn set_bit(&mut self, index: usize, value: bool) {
        let data_idx = index / usize::BITS as usize;
        let bit_idx = index % usize::BITS as usize;

        if index >= self.len {
            self.len = index + 1;

            if self.data.len() < data_idx + 1 {
                self.data.resize_with(data_idx + 1, || 0);
            }
        }

        let bits = &mut self.data[data_idx];
        let mask = 1 << bit_idx;
        *bits &= !mask;
        *bits |= (value as usize) << bit_idx;
    }
}

pub struct Bitsetsss {
    bitsets: Vec<Option<Bitvec>>,
}

use crate::EcsId;
impl Bitsetsss {
    fn new() -> Self {
        Self {
            bitsets: Vec::new(),
        }
    }

    fn with_capacity(cap: usize) -> Self {
        Self {
            bitsets: Vec::with_capacity(cap),
        }
    }

    fn insert_bitvec(&mut self, comp_id: EcsId) {
        if let None = self.bitsets.get(comp_id.uindex()) {
            self.bitsets
                .resize_with(comp_id.uindex() + 1, || Some(Bitvec::new()));
            return;
        }

        if let bitset @ None = &mut self.bitsets[comp_id.uindex()] {
            *bitset = Some(Bitvec::new());
            return;
        }

        panic!("Attempted to insert a bitvec that already existed")
    }

    fn get_bitvec(&self, comp_id: EcsId) -> Option<&Bitvec> {
        self.bitsets.get(comp_id.uindex())?.as_ref()
    }

    fn set_bit(&mut self, entity: EcsId, index: usize, value: bool) {
        let bitvec = (&mut self.bitsets[entity.uindex()])
            .get_or_insert_with(|| Bitvec::with_capacity(index));
        bitvec.set_bit(index, value);
    }
}

#[cfg(test)]
mod tests {
    use super::{BitsetIterator, Bitsetsss};
    use crate::EcsId;

    #[test]
    fn insert_one() {
        let mut bitsets = Bitsetsss::new();
        let key = EcsId::new(0, 0);
        bitsets.insert_bitvec(key);

        let bitvec = bitsets.get_bitvec(key).unwrap();
        assert_eq!(bitvec.data.len(), 0);
    }

    #[test]
    fn set_bit() {
        let mut bitsets = Bitsetsss::new();
        let key = EcsId::new(0, 0);
        bitsets.insert_bitvec(key);
        bitsets.set_bit(key, 0, true);
        bitsets.set_bit(key, 3, true);

        let bitvec = bitsets.get_bitvec(key).unwrap();

        assert_eq!(bitvec.data[0], 0b1001);
        assert_eq!(bitvec.len, 4);
    }

    #[test]
    fn set_bit_far() {
        let mut bitsets = Bitsetsss::new();
        let key = EcsId::new(0, 0);
        bitsets.insert_bitvec(key);
        bitsets.set_bit(key, usize::BITS as usize, true);

        let bitvec = bitsets.get_bitvec(key).unwrap();
        assert_eq!(bitvec.data[0], 0b0);
        assert_eq!(bitvec.data[1], 0b1);
        assert_eq!(bitvec.len, (usize::BITS + 1) as _);
    }

    #[test]
    fn get_bit() {
        let mut bitsets = Bitsetsss::new();
        let key = EcsId::new(0, 0);
        bitsets.insert_bitvec(key);
        bitsets.set_bit(key, 3, true);

        let bitvec = bitsets.get_bitvec(key).unwrap();
        assert!(bitvec.get_bit(3).unwrap());
    }

    #[test]
    fn bitset_iterator() {
        let mut bitsets = Bitsetsss::new();

        let key1 = EcsId::new(0, 0);
        bitsets.insert_bitvec(key1);
        bitsets.set_bit(key1, 1, true);
        bitsets.set_bit(key1, 2, true);

        let key2 = EcsId::new(1, 0);
        bitsets.insert_bitvec(key2);
        bitsets.set_bit(key2, 2, true);
        bitsets.set_bit(key2, 3, true);

        let bitvec1 = bitsets.get_bitvec(key1).unwrap();
        let bitvec2 = bitsets.get_bitvec(key2).unwrap();

        let map: fn(_) -> _ = |x| x;

        use super::BitsetIterator;
        let mut bitset_iter =
            BitsetIterator::new([(bitvec1.data.iter(), map), (bitvec2.data.iter(), map)], 4);

        assert_eq!(bitset_iter.next(), Some(2));
        bitset_iter.next().unwrap_none();
    }
}
