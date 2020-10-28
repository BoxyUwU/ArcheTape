use ellecs::entities::Entity;
use ellecs::spawn;
use ellecs::world::World;

struct P1([u8; 146]);
struct P2([u8; 146]);
struct P3([u8; 146]);
struct P4([u8; 146]);
struct P5([u8; 146]);
struct P6([u8; 146]);
struct P7([u8; 146]);

struct A(f32);
struct B(f32);

fn main() {
    let mut world = World::new();
    let mut entities = Vec::with_capacity(10000);

    for _ in 0..10_000 {
        entities.push(spawn!(
            &mut world,
            P1([1; 146]),
            P2([1; 146]),
            P3([1; 146]),
            P4([1; 146]),
            P5([1; 146]),
            P6([1; 146]),
            P7([1; 146]),
            A(0.0),
        ));
    }

    for _ in 0..100_000 {
        for &entity in entities.iter() {
            world.add_component(entity, B(1.));
        }
        for &entity in entities.iter() {
            world.remove_component::<B>(entity);
        }
    }
}
