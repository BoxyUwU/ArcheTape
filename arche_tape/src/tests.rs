#[cfg(test)]
mod entities {
    use crate::{entities::*, spawn, EcsId, World};

    #[test]
    pub fn spawn_one() {
        let mut entities = Entities::new();

        assert_eq!(EcsId::new(0, 0), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);
    }

    #[test]
    pub fn spawn_multiple() {
        let mut entities = Entities::new();

        assert_eq!(EcsId::new(0, 0), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        assert_eq!(EcsId::new(1, 0), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 2);

        assert_eq!(EcsId::new(2, 0), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 3);

        assert_eq!(EcsId::new(3, 0), entities.spawn());
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 4);
    }

    #[test]
    pub fn spawn_one_despawn_one() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(EcsId::new(0, 0), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        entities.despawn(entity);
        assert!(entities.despawned.len() == 1);
        assert!(entities.generations.len() == 1);
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.is_alive(entity) == false);
    }

    #[test]
    #[should_panic(
        expected = "could not get generation for Generation 0xFFFFFFFF, Index 0xFFFFFFFF"
    )]
    pub fn despawn_invalid() {
        let entities = Entities::new();
        let invalid_id = EcsId::new(u32::MAX, u32::MAX);
        let _ = entities.is_alive(invalid_id);
    }

    #[test]
    pub fn reuse_despawned() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(EcsId::new(0, 0), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        entities.despawn(entity);
        assert!(entities.despawned.len() == 1);
        assert!(entities.generations.len() == 1);
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.is_alive(entity) == false);

        let entity2 = entities.spawn();
        assert_eq!(EcsId::new(0, 1), entity2);
        assert!(entity != entity2);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);
        assert!(entities.generations.get(0).unwrap().1 == 1);
        assert!(entities.is_alive(entity) == false);
        assert!(entities.is_alive(entity2) == true);
    }

    #[test]
    pub fn double_despawn() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        entities.despawn(entity);

        assert!(entities.despawn(entity) == false);
        assert!(entities.despawned.len() == 1);
        assert!(entities.generations.len() == 1);
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.is_alive(entity) == false);

        assert!(entities.spawn() == EcsId::new(0, 1));
    }

    #[test]
    pub fn reuse_despawn2() {
        let mut entities = Entities::new();

        let entity = entities.spawn();
        assert_eq!(EcsId::new(0, 0), entity);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 1);

        let entity2 = entities.spawn();
        assert_eq!(EcsId::new(1, 0), entity2);
        assert!(entity != entity2);
        assert!(entities.despawned.len() == 0);
        assert!(entities.generations.len() == 2);
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.generations.get(1).unwrap().1 == 0);
        assert!(entities.is_alive(entity) == true);
        assert!(entities.is_alive(entity2) == true);

        assert!(entities.despawn(entity) == true);
        assert!(entities.is_alive(entity) == false);
        assert!(entities.is_alive(entity2) == true);
        assert!(entities.generations.len() == 2);
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.generations.get(1).unwrap().1 == 0);
        assert!(entities.despawned.len() == 1);
        assert!(*entities.despawned.get(0).unwrap() == 0);

        let entity3 = entities.spawn();
        assert!(entities.is_alive(entity) == false);
        assert!(entities.is_alive(entity2) == true);
        assert!(entities.is_alive(entity3) == true);

        assert!(entities.generations.len() == 2);
        assert!(entities.generations.get(0).unwrap().1 == 1);
        assert!(entities.generations.get(1).unwrap().1 == 0);

        assert!(entities.despawned.len() == 0);
    }

    #[test]
    pub fn generation_wraps() {
        let mut entities = Entities::new();

        entities.generations.push((false, u32::MAX));
        entities.despawned.push(0);

        let entity = entities.spawn();

        assert!(entities.is_alive(entity));
        assert!(entity == EcsId::new(0, 0));
        assert!(entities.generations.len() == 1);
        assert!(entities.generations.get(0).unwrap().1 == 0);
        assert!(entities.despawned.len() == 0);
    }

    #[test]
    pub fn build_with_zst() -> () {
        struct Zero;

        let mut world = World::new();
        let entity = spawn!(&mut world, Zero);
        assert!(world.is_alive(entity));
    }
}

#[cfg(test)]
mod world {
    use crate::{spawn, world::ComponentMeta, EcsId, World};

    #[test]
    pub fn get() {
        let mut world = World::new();

        let entity = spawn!(&mut world, 10_u32, 12_u64, "Hello");
        let entity2 = spawn!(&mut world, 18_u32, "AWDAWDAWD", 16.0f32);

        let str_comp: &mut &str = world.get_component_mut(entity).unwrap();
        assert!(*str_comp == "Hello");

        let str_comp: &mut &str = world.get_component_mut(entity2).unwrap();
        assert!(*str_comp == "AWDAWDAWD");
    }

    #[test]
    pub fn entity_archetype_lookup() {
        let mut world = World::new();

        let entity = spawn!(&mut world, 10_u32, 12_u64);

        let entity_meta = world.get_entity_meta(entity).unwrap();
        assert!(entity_meta.instance_meta.index == 0);
        assert!(entity_meta.instance_meta.archetype.0 == 1);
    }

    #[test]
    pub fn add_component() {
        let mut world = World::new();
        let entity = spawn!(&mut world, 1_u32);
        world.add_component(entity, 2_u64);

        assert!(world.archetypes.len() == 3);
        let entity_meta = world.get_entity_meta(entity).unwrap();
        assert!(entity_meta.instance_meta.archetype.0 == 2);
        assert!(entity_meta.instance_meta.index == 0);

        // The two component entities
        assert!(world.archetypes[0].entities.len() == 2);
        assert!(world.archetypes[0].component_storages.len() == 0);
        for (_, lock) in world.archetypes[0].component_storages.iter_mut() {
            let storage = lock.get_mut();
            assert!(storage.len() == 0);
        }

        // The first archetype entity was in
        assert!(world.archetypes[1].entities.len() == 0);
        assert!(world.archetypes[1].component_storages.len() == 1);
        for (_, lock) in world.archetypes[1].component_storages.iter_mut() {
            let storage = lock.get_mut();
            assert!(storage.len() == 0);
        }

        // The current archetype entity was in
        assert!(world.archetypes[2].entities.len() == 1);
        assert!(world.archetypes[2].component_storages.len() == 2);
        for (_, lock) in world.archetypes[2].component_storages.iter_mut() {
            let storage = lock.get_mut();
            assert!(storage.len() == 1);
        }

        let mut run_times = 0;
        let query = world.query::<(&u32, &u64)>();
        query.borrow().for_each_mut(|(left, right)| {
            assert!(*left == 1);
            assert!(*right == 2);
            run_times += 1;
        });
        assert!(run_times == 1);
    }

    #[test]
    pub fn add_component_then_spawn() {
        let mut world = World::new();
        let entity = spawn!(&mut world, 1_u32);
        world.add_component(entity, 2_u64);

        let entity2 = spawn!(&mut world, 3_u32, 4_u64);

        assert!(world.archetypes.len() == 3);

        // Component entities
        assert!(world.archetypes[0].entities.len() == 2);
        assert!(world.archetypes[0].component_storages.len() == 0);

        // Original first entity archetype
        assert!(world.archetypes[1].entities.len() == 0);
        assert!(world.archetypes[1].component_storages.len() == 1);
        assert!(world.archetypes[1].component_storages[0].1.get_mut().len() == 0);

        // Entity2 + Entity1 Archetpye
        assert!(world.archetypes[2].entities.len() == 2);
        assert!(world.archetypes[2].entities[0] == entity);
        assert!(world.archetypes[2].entities[1] == entity2);
        assert!(world.archetypes[2].component_storages.len() == 2);
        assert!(world.archetypes[2].component_storages[0].1.get_mut().len() == 2);
        assert!(world.archetypes[2].component_storages[1].1.get_mut().len() == 2);

        let entity_meta = world.get_entity_meta(entity).unwrap();
        assert!(entity_meta.instance_meta.archetype.0 == 2);
        assert!(entity_meta.instance_meta.index == 0);

        let entity_meta = world.get_entity_meta(entity2).unwrap();
        assert!(entity_meta.instance_meta.archetype.0 == 2);
        assert!(entity_meta.instance_meta.index == 1);

        let mut run_times = 0;
        let mut checks = vec![(1, 2), (3, 4)].into_iter();
        let query = world.query::<(&u32, &u64)>();
        query.borrow().for_each_mut(|(left, right)| {
            assert!(checks.next().unwrap() == (*left, *right));
            run_times += 1;
        });
        assert!(run_times == 2);
    }

    #[test]
    pub fn add_two() {
        struct A(f32);
        struct B(f32);

        let mut world = World::new();
        let entity_1 = spawn!(&mut world, A(1.));
        let entity_2 = spawn!(&mut world, A(1.));

        assert!(world.archetypes[0].entities.len() == 1);
        assert!(world.archetypes[0].component_storages.len() == 0);

        let entity_1_meta = world.get_entity_meta(entity_1).unwrap();
        assert!(world.archetypes[1].entities[0] == entity_1);
        assert!(entity_1_meta.instance_meta.archetype.0 == 1);
        assert!(entity_1_meta.instance_meta.index == 0);

        let entity_2_meta = world.get_entity_meta(entity_2).unwrap();
        assert!(world.archetypes[1].entities[1] == entity_2);
        assert!(entity_2_meta.instance_meta.archetype.0 == 1);
        assert!(entity_2_meta.instance_meta.index == 1);

        world.add_component(entity_1, B(2.));
        assert!(world.archetypes[0].entities.len() == 2);

        assert!(world.archetypes[1].entities[0] == entity_2);
        assert!(world.archetypes[1].entities.len() == 1);
        assert!(world.archetypes[2].entities[0] == entity_1);
        assert!(world.archetypes[2].entities.len() == 1);

        let entity_1_meta = world.get_entity_meta(entity_1).unwrap();
        assert!(entity_1_meta.instance_meta.archetype.0 == 2);
        assert!(entity_1_meta.instance_meta.index == 0);

        let entity_2_meta = world.get_entity_meta(entity_2).unwrap();
        assert!(entity_2_meta.instance_meta.archetype.0 == 1);
        assert!(entity_2_meta.instance_meta.index == 0);

        world.add_component(entity_2, B(2.));
        assert!(world.archetypes[0].entities.len() == 2);
        assert!(world.archetypes[1].entities.len() == 0);
        assert!(world.archetypes[2].entities.len() == 2);

        assert!(world.archetypes[2].entities[0] == entity_1);
        assert!(world.archetypes[2].entities[1] == entity_2);

        let entity_1_meta = world.get_entity_meta(entity_1).unwrap();
        assert!(entity_1_meta.instance_meta.archetype.0 == 2);
        assert!(entity_1_meta.instance_meta.index == 0);

        let entity_2_meta = world.get_entity_meta(entity_2).unwrap();
        assert!(entity_2_meta.instance_meta.archetype.0 == 2);
        assert!(entity_2_meta.instance_meta.index == 1);
    }

    #[test]
    pub fn add_multiple() {
        struct A(f32);
        struct B(f32);

        let mut world = World::new();
        let mut entities = Vec::with_capacity(500);

        for _ in 0..10 {
            entities.push(spawn!(&mut world, A(1.)));
        }

        for &entity in entities.iter() {
            world.add_component(entity, B(1.));
        }
        for &entity in entities.iter() {
            world.remove_component::<B>(entity);
        }
    }

    #[test]
    pub fn despawn_meta_update() {
        let mut world = World::new();

        let e1 = world.spawn().with(10_u32).build();
        let e2 = world.spawn().with(12_u32).build();
        let e3 = world.spawn().with(14_u32).build();

        assert!(world.despawn(e1));

        assert!(world.is_alive(e1) == false);
        assert!(world.get_entity_meta(e1).is_none());

        assert!(world.is_alive(e2));
        assert!(world.is_alive(e3));

        assert!(*world.get_component_mut::<u32>(e2).unwrap() == 12);
        assert!(*world.get_component_mut::<u32>(e3).unwrap() == 14);
    }

    #[test]
    pub fn despawn_component_entity() {
        // TODO: Removing entities when they despawn not yet implemented
        return;
        let mut world = World::new();

        unsafe {
            let component_entity = world
                .spawn_with_component_meta(ComponentMeta::from_generic::<u32>())
                .build();

            let e1 = world
                .spawn()
                .with_dynamic_with_data(&mut 10_u32 as *mut _ as *mut _, component_entity)
                .build();
            let e2 = world
                .spawn()
                .with_dynamic_with_data(&mut 10_u32 as *mut _ as *mut _, component_entity)
                .build();
            let e3 = world
                .spawn()
                .with_dynamic_with_data(&mut 10_u32 as *mut _ as *mut _, component_entity)
                .build();

            world.despawn(component_entity);

            assert!(world.archetypes.len() == 2);

            let assert_meta = |world: &mut World, entity: EcsId, archetype_idx, entity_idx| {
                let meta = world.get_entity_meta(entity).unwrap();
                assert!(meta.instance_meta.archetype.0 == archetype_idx);
                assert!(meta.instance_meta.index == entity_idx);
            };

            assert!(world.archetypes[0].entities == &[e1, e2, e3]);
            assert_meta(&mut world, e1, 0, 0);
            assert_meta(&mut world, e2, 0, 1);
            assert_meta(&mut world, e3, 0, 2);

            assert!(world.archetypes[1].entities.len() == 0);
            assert!(world.get_entity_meta(component_entity).is_none());
        }
    }

    // TODO: Boxy can you make the following tests actually work?
    // Currently they basically just want to not panic, but they should check capacity if possible
    #[test]
    pub fn spawn() -> () {
        let mut world = World::new();
        let entity = world.spawn().build();
        assert_eq!(entity, EcsId::new(0, 0));
    }

    #[test]
    pub fn spawn_with_capacity() -> () {
        let mut world = World::new();
        let entity = world.spawn_with_capacity(32).build();
        assert_eq!(entity, EcsId::new(0, 0));
    }

    #[test]
    pub fn spawn_with_capacity_zero() -> () {
        let mut world = World::new();
        let entity = world.spawn_with_capacity(0).build();
        assert_eq!(entity, EcsId::new(0, 0));
    }
}

#[cfg(test)]
mod archetype_iter {
    use crate::{entities::Entities, query::Query, spawn, EcsId, World};

    #[test]
    fn for_each_mut() {
        let mut world = World::new();

        spawn!(&mut world, 10_u32, 12_u64);
        spawn!(&mut world, 15_u32, 14_u64);
        spawn!(&mut world, 20_u32, 16_u64);

        let query = world.query::<(&mut u32, &u64)>();
        let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
        query.borrow().for_each_mut(|(left, right)| {
            assert_eq!(checks.next().unwrap(), (*left, *right));
        });
        assert_eq!(checks.next(), None);
    }

    #[test]
    fn for_each_iterator() {
        let mut world = World::new();

        spawn!(&mut world, 10_u32, 12_u64);
        spawn!(&mut world, 15_u32, 14_u64);
        spawn!(&mut world, 20_u32, 16_u64);

        let query = world.query::<(&mut u32, &u64)>();

        let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
        query
            .borrow()
            .for_each_mut(|(left, right)| assert_eq!((*left, *right), checks.next().unwrap()));
        assert!(checks.next().is_none());
    }

    #[test]
    fn for_each_subset_iterator() {
        let mut world = World::new();

        spawn!(&mut world, 10_u32, 12_u64);
        spawn!(&mut world, 15_u32, 14_u64);
        spawn!(&mut world, 20_u32, 16_u64);

        let query = world.query::<(&mut u32,)>();

        let mut checks = vec![10, 15, 20].into_iter();
        query
            .borrow()
            .for_each_mut(|(left,)| assert_eq!(*left, checks.next().unwrap()));
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

        let query = world.query::<(&mut u32,)>();

        let mut checks = vec![10, 15, 20, 11, 16, 21].into_iter();
        query
            .borrow()
            .for_each_mut(|(left,)| assert_eq!(*left, checks.next().unwrap()));
        assert!(checks.next().is_none());
    }

    #[test]
    fn query_param_in_func() {
        let mut world = World::new();
        spawn!(&mut world, 10_u32, 12_u64);
        let query = world.query::<(&u32, &u64)>();

        fn func(query: Query<(&u32, &u64)>) {
            let mut ran = false;
            query.borrow().for_each_mut(|(left, right)| {
                assert!(*left == 10);
                assert!(*right == 12);
                ran = true;
            });
            assert!(ran);
        }

        func(query);
    }

    #[test]
    fn entity_query() {
        let mut world = World::new();

        spawn!(&mut world, 1_u32, 12_u64);

        let query = world.query::<(Entities, &u32, &u64)>();

        let mut checks = vec![(EcsId::new(0, 0), 1, 12)].into_iter();
        query.borrow().for_each_mut(|(entity, left, right)| {
            assert!(checks.next().unwrap() == (entity, *left, *right));
        });
        assert!(checks.next().is_none());
    }
}

#[cfg(test)]
mod bitsetsss {
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
}

#[cfg(test)]
mod bitset_iterator {
    use crate::archetype_iter::BitsetIterator;

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

#[cfg(test)]
mod query {
    use crate::{EcsId, World, entities::Entities, query::*};

    #[test]
    fn for_each_mut() {
        let mut world = World::new();

        spawn!(&mut world, 10_u32, 12_u64);
        spawn!(&mut world, 15_u32, 14_u64);
        spawn!(&mut world, 20_u32, 16_u64);

        let query = world.query::<(&mut u32, &u64)>();
        let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
        query.borrow().for_each_mut(|(left, right)| {
            assert_eq!(checks.next().unwrap(), (*left, *right));
        });
        assert_eq!(checks.next(), None);
    }

    #[test]
    fn for_each_iterator() {
        let mut world = World::new();

        spawn!(&mut world, 10_u32, 12_u64);
        spawn!(&mut world, 15_u32, 14_u64);
        spawn!(&mut world, 20_u32, 16_u64);

        let query = world.query::<(&mut u32, &u64)>();

        let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
        query
            .borrow()
            .for_each_mut(|(left, right)| assert_eq!((*left, *right), checks.next().unwrap()));
        assert!(checks.next().is_none());
    }

    #[test]
    fn for_each_subset_iterator() {
        let mut world = World::new();

        spawn!(&mut world, 10_u32, 12_u64);
        spawn!(&mut world, 15_u32, 14_u64);
        spawn!(&mut world, 20_u32, 16_u64);

        let query = world.query::<(&mut u32,)>();

        let mut checks = vec![10, 15, 20].into_iter();
        query
            .borrow()
            .for_each_mut(|(left,)| assert_eq!(*left, checks.next().unwrap()));
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

        let query = world.query::<(&mut u32,)>();

        let mut checks = vec![10, 15, 20, 11, 16, 21].into_iter();
        query
            .borrow()
            .for_each_mut(|(left,)| assert_eq!(*left, checks.next().unwrap()));
        assert!(checks.next().is_none());
    }

    #[test]
    fn query_param_in_func() {
        let mut world = World::new();
        spawn!(&mut world, 10_u32, 12_u64);
        let query = world.query::<(&u32, &u64)>();

        fn func(query: Query<(&u32, &u64)>) {
            let mut ran = false;
            query.borrow().for_each_mut(|(left, right)| {
                assert!(*left == 10);
                assert!(*right == 12);
                ran = true;
            });
            assert!(ran);
        }

        func(query);
    }

    #[test]
    fn entity_query() {
        let mut world = World::new();

        spawn!(&mut world, 1_u32, 12_u64);

        let query = world.query::<(Entities, &u32, &u64)>();

        let mut checks = vec![(EcsId::new(0, 0), 1, 12)].into_iter();
        query.borrow().for_each_mut(|(entity, left, right)| {
            assert!(checks.next().unwrap() == (entity, *left, *right));
        });
        assert!(checks.next().is_none());
    }
}
