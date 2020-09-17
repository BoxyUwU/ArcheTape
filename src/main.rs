use ellecs::world::World;

fn main() {
    let mut world = World::new();
    world.spawn((10_u32, 12_u64, true));

    let query = world.query::<(&u32, &u64, &bool)>();

    query.borrow().into_for_each_mut(|(l, m, r)| {
        println!("{}, {}, {}", l, m, r);
    });
}
