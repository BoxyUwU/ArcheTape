use arche_tape::entities::EcsId;
use arche_tape::spawn;
use arche_tape::world::World;

pub struct A(f32);

pub struct Benchmark(World, Box<[EcsId]>);

pub fn main() {
    let mut world = World::new();
    let mut entities = Vec::with_capacity(10_000);

    for _ in 0..10_000 {
        let entity = spawn!(&mut world, A(10.0));
        entities.push(entity);
    }

    for _ in 0..1_000_000 {
        let mut q = world.query::<(&mut A,)>();
        for &entity in entities.iter() {
            let _a = q.get(entity).unwrap().0;
        }
    }
}
