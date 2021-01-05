use std::{borrow::BorrowMut, marker::PhantomData, slice::Iter};
pub struct BitsetIterator<'a, Iters: BorrowMut<[(Iter<'a, usize>, fn(usize) -> usize)]>> {
    phantom: PhantomData<&'a [usize]>,
    iters: Iters,

    index: usize,

    bits_remaining: u32,
    current_bits: usize,
}

impl<'a, Iters: BorrowMut<[(Iter<'a, usize>, fn(usize) -> usize)]>> BitsetIterator<'a, Iters> {
    fn new(iters: Iters) -> Self {
        Self {
            phantom: PhantomData,
            iters,

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
            self.current_bits = usize::wrapping_shr(self.current_bits, zeros + 1);
            self.index += { zeros + 1 } as usize;

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
        let mut bitset_iter = BitsetIterator::new([(data.iter(), map)]);

        bitset_iter.next().unwrap_none();
    }

    #[test]
    fn single_bitset() {
        let map: fn(_) -> _ = |x| x;
        let data = vec![0b0000_1011];
        let mut bitset_iter = BitsetIterator::new([(data.iter(), map)]);

        assert_eq!(bitset_iter.next(), Some(0));
        assert_eq!(bitset_iter.next(), Some(1));
        assert_eq!(bitset_iter.next(), Some(3));
        bitset_iter.next().unwrap_none();
    }

    #[test]
    fn gapped_bitset() {
        let map: fn(_) -> _ = |x| x;
        let data = vec![0, 0b101];
        let mut bitset_iter = BitsetIterator::new([(data.iter(), map)]);

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

        let mut bitset_iter = BitsetIterator::new([
            (data1.iter(), map),
            (data2.iter(), map),
            (data3.iter(), map),
        ]);

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

        let mut bitset_iter =
            BitsetIterator::new([(data1.iter(), map), (data2.iter(), invert_map)]);

        assert_eq!(bitset_iter.next(), Some(0));
        assert_eq!(bitset_iter.next(), Some(3));
        assert_eq!(bitset_iter.next(), Some(7));
        bitset_iter.next().unwrap_none();
    }

    #[test]
    fn all_ones() {
        let map: fn(_) -> _ = |x| x;
        let data = vec![usize::MAX];
        let mut bitset_iter = BitsetIterator::new([(data.iter(), map)]);

        for n in 0..usize::BITS {
            assert_eq!(bitset_iter.next(), Some(n as _));
        }
        bitset_iter.next().unwrap_none();
    }
}
