use cgmath::*;
use ellecs::spawn;
use ellecs::world::World;
pub struct Benchmark(World);

fn main() {
    pub struct A(f32);

    let mut world = World::new();
    let mut entities = Vec::with_capacity(10_000);

    for _ in 0..10_000 {
        let entity = spawn!(&mut world, A(10.0));
        entities.push(entity);
        world.add_component(entity, "test");
    }

    for &entity in entities.iter() {
        world.get_component_mut::<A>(entity).unwrap();
    }
}
