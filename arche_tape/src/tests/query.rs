use crate::{EcsId, EcsIds, StaticQuery, World};

#[test]
fn for_each_mut() {
    let mut world = World::new();

    spawn!(&mut world, 10_u32, 12_u64);
    spawn!(&mut world, 15_u32, 14_u64);
    spawn!(&mut world, 20_u32, 16_u64);

    let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
    for (left, right) in world.query::<(&mut u32, &u64)>().iter() {
        assert_eq!(checks.next().unwrap(), (*left, *right));
    }

    assert_eq!(checks.next(), None);
}

#[test]
fn for_each_iterator() {
    let mut world = World::new();

    spawn!(&mut world, 10_u32, 12_u64);
    spawn!(&mut world, 15_u32, 14_u64);
    spawn!(&mut world, 20_u32, 16_u64);

    let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
    for (left, right) in world.query::<(&mut u32, &u64)>().iter() {
        assert_eq!((*left, *right), checks.next().unwrap())
    }

    assert!(checks.next().is_none());
}

#[test]
fn for_each_subset_iterator() {
    let mut world = World::new();

    spawn!(&mut world, 10_u32, 12_u64);
    spawn!(&mut world, 15_u32, 14_u64);
    spawn!(&mut world, 20_u32, 16_u64);

    let mut checks = vec![10, 15, 20].into_iter();
    for (left,) in world.query::<(&mut u32,)>().iter() {
        assert_eq!(*left, checks.next().unwrap())
    }

    assert!(checks.next().is_none());
}

#[test]
fn for_each_multi_archetype_iterator() {
    let mut world = World::new();

    spawn!(&mut world, 10_u32, 12_u64);
    spawn!(&mut world, 15_u32, 14_u64);
    spawn!(&mut world, 20_u32, 16_u64);

    spawn!(&mut world, 11_u32, 12_u64, 99_u128);
    spawn!(&mut world, 16_u32, 14_u64, 99_u128);
    spawn!(&mut world, 21_u32, 16_u64, 99_u128);

    let mut checks = vec![10, 15, 20, 11, 16, 21].into_iter();
    for (left,) in world.query::<(&mut u32,)>().iter() {
        assert_eq!(*left, checks.next().unwrap())
    }

    assert!(checks.next().is_none());
}

#[test]
fn query_param_in_func() {
    let mut world = World::new();
    spawn!(&mut world, 10_u32, 12_u64);
    let query = world.query::<(&u32, &u64)>();

    fn func(mut query: StaticQuery<(&u32, &u64)>) {
        let mut ran = false;
        for (left, right) in query.iter() {
            assert!(*left == 10);
            assert!(*right == 12);
            ran = true;
        }
        assert!(ran);
    }

    func(query);
}

#[test]
fn entity_query() {
    let mut world = World::new();

    spawn!(&mut world, 1_u32, 12_u64);

    let mut checks = vec![(EcsId::new(0, 0), 1, 12)].into_iter();
    for (e, left, right) in world.query::<(EcsIds, &u32, &u64)>().iter() {
        assert!(checks.next().unwrap() == (e, *left, *right));
    }

    assert!(checks.next().is_none());
}

#[test]
fn query_get() {
    let mut world = World::new();

    let a = spawn!(&mut world, 41_u32, 9_u64);
    let b = spawn!(&mut world, 8_u32);
    let c = spawn!(&mut world, 12_u64);
    let d = spawn!(&mut world, 1_u32, 128_u64, 28_u128);

    let mut q = world.query::<(&mut u32,)>();

    assert_eq!(q.get(a), Some((&mut 41,)));
    assert_eq!(q.get(b), Some((&mut 8,)));
    assert_eq!(q.get(c), None);
    assert_eq!(q.get(d), Some((&mut 1,)));
}
