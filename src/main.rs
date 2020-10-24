use cgmath::*;
use ellecs::world::World;

#[derive(Copy, Clone)]
struct Transform(Matrix4<f32>);
#[derive(Copy, Clone)]
struct Position(Vector3<f32>);
#[derive(Copy, Clone)]
struct Rotation(Vector3<f32>);
#[derive(Copy, Clone)]
struct Velocity(Vector3<f32>);

pub fn main() {
    let mut world = World::new();

    for _ in 0..10_000 {
        world
            .spawn()
            .with(Transform(Matrix4::from_scale(1.0)))
            .with(Position(Vector3::unit_x()))
            .with(Rotation(Vector3::unit_x()))
            .with(Velocity(Vector3::unit_x()))
            .build();
    }

    for _ in 0..1_000_000 {
        world
            .query::<(&mut Position, &mut Velocity)>()
            .borrow()
            .for_each(|(pos, vel)| {
                pos.0 += vel.0;
            });
    }
}
