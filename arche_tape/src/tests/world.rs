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
    for (left, right) in world.query::<(&u32, &u64)>().iter() {
        assert!(*left == 1);
        assert!(*right == 2);
        run_times += 1;
    }
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
    for (left, right) in world.query::<(&u32, &u64)>().iter() {
        assert!(checks.next().unwrap() == (*left, *right));
        run_times += 1;
    }
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
