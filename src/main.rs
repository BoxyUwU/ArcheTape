use arche_tape::world::World;

fn main() {
    let mut world = World::new();

    let e1 = world.spawn().with(10_u32).build();
    let e2 = world.spawn().build();

    world.add_component_dynamic(e1, e2);

    assert!(world.get_component_mut_dynamic(e1, e2).is_some());
}
