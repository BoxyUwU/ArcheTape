use arche_tape::world::World;

fn main() {
    let mut world1 = World::new();
    let mut world2 = World::new();

    let e1_w1 = world1.spawn().build();
    dbg!(e1_w1);
    world1.despawn(e1_w1);

    let e1_w2 = world2.spawn().build();
    dbg!(e1_w2);
    world2.despawn(e1_w2);
    let e2_w2 = world2.spawn().build();
    dbg!(e2_w2);

    dbg!(world1.is_alive(e1_w1));
    dbg!(world1.is_alive(e1_w2));
    dbg!(world1.is_alive(e2_w2));

    dbg!(world2.is_alive(e1_w1));
    dbg!(world2.is_alive(e1_w2));
    dbg!(world2.is_alive(e2_w2));
}
