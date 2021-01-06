use crate::archetype_iter::Bitsetsss;
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
    assert_eq!(bitvec.len, (usize::BITS + 1) as usize);
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

    use crate::archetype_iter::BitsetIterator;
    let mut bitset_iter =
        BitsetIterator::new([(bitvec1.data.iter(), map), (bitvec2.data.iter(), map)], 4);

    assert_eq!(bitset_iter.next(), Some(2));
    bitset_iter.next().unwrap_none();
}
