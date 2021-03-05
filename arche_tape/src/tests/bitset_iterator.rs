use crate::bitset_iterator::BitsetIterator;

#[test]
fn empty_bitset() {
    let map: fn(_) -> _ = |x| x;
    let data = vec![];
    let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], 0);

    assert!(matches!(bitset_iter.next(), None));
}

#[test]
fn single_bitset() {
    let map: fn(_) -> _ = |x| x;
    let data = vec![0b0000_1011];
    let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], usize::BITS);

    assert_eq!(bitset_iter.next(), Some(0));
    assert_eq!(bitset_iter.next(), Some(1));
    assert_eq!(bitset_iter.next(), Some(3));
    assert!(matches!(bitset_iter.next(), None));
}

#[test]
fn gapped_bitset() {
    let map: fn(_) -> _ = |x| x;
    let data = vec![0, 0b101];
    let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], usize::BITS * 2);

    assert_eq!(bitset_iter.next(), Some(64));
    assert_eq!(bitset_iter.next(), Some(66));
    assert!(matches!(bitset_iter.next(), None));
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
    assert!(matches!(bitset_iter.next(), None));
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
    assert!(matches!(bitset_iter.next(), None));
}

#[test]
fn all_ones() {
    let map: fn(_) -> _ = |x| x;
    let data = vec![usize::MAX];
    let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], usize::BITS);

    for n in 0..usize::BITS {
        assert_eq!(bitset_iter.next(), Some(n as _));
    }
    assert!(matches!(bitset_iter.next(), None));
}

#[test]
fn bit_length() {
    let map: fn(_) -> _ = |x| x;
    let data = vec![0b101];
    let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], 2);

    assert_eq!(bitset_iter.next(), Some(0));
    assert!(matches!(bitset_iter.next(), None));
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
    assert!(matches!(bitset_iter.next(), None));
}

#[test]
fn incorrect_bit_length() {
    let map: fn(_) -> _ = |x| x;
    let data = vec![0b101];
    let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], 2000);

    assert_eq!(bitset_iter.next(), Some(0));
    assert_eq!(bitset_iter.next(), Some(2));
    assert!(matches!(bitset_iter.next(), None));
}

#[test]
fn returns_none_continuously_incorrect_bit_length() {
    let map: fn(_) -> _ = |x| x;
    let data = vec![0b101];
    let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], 2000);

    assert_eq!(bitset_iter.next(), Some(0));
    assert_eq!(bitset_iter.next(), Some(2));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
}

#[test]
fn returns_none_continuously_bit_length() {
    let map: fn(_) -> _ = |x| x;
    let data = vec![0b101];
    let mut bitset_iter = BitsetIterator::new([(data.iter(), map)], 3);

    assert_eq!(bitset_iter.next(), Some(0));
    assert_eq!(bitset_iter.next(), Some(2));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
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

    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
    assert!(matches!(bitset_iter.next(), None));
}
