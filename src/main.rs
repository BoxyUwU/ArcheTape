use cgmath::*;
use ellecs::spawn;
use ellecs::world::World;

#[derive(Copy, Clone)]
struct Transform(Matrix4<f32>);
#[derive(Copy, Clone)]
struct Position(Vector3<f32>);
#[derive(Copy, Clone)]
struct Rotation(Vector3<f32>);
#[derive(Copy, Clone)]
struct Velocity(Vector3<f32>);

fn main() {
    let mut world = World::new();

    for _ in 0..1_000_000 {
        spawn!(
            &mut world,
            Transform(Matrix4::from_scale(1.0)),
            Position(Vector3::unit_x()),
            Rotation(Vector3::unit_x()),
            Velocity(Vector3::unit_x()),
        );
    }
}
