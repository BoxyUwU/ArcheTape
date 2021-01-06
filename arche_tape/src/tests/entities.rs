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
#[should_panic(expected = "could not get generation for Generation 0xFFFFFFFF, Index 0xFFFFFFFF")]
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
