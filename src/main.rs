use arche_tape::entities::EcsId;
use arche_tape::spawn;
use arche_tape::world::World;

#[derive(Copy, Clone)]
struct A(f32);
#[derive(Copy, Clone)]
struct B(f32);

pub struct Benchmark(World, Box<[EcsId]>);

fn main() {
    let mut world = World::new();
    let mut entities = Vec::with_capacity(10000);

    for _ in 0..10_000 {
        entities.push(spawn!(&mut world, A(1.)));
    }

    for _ in 0..1_000_00 {
        for &entity in entities.iter() {
            world.add_component(entity, B(1.));
        }
        for &entity in entities.iter() {
            world.remove_component::<B>(entity);
        }
    }
}
