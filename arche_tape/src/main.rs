use arche_tape::spawn;
use arche_tape::world::World;

macro_rules! setup {
    ($world:ident, $($x:ident),*) => {
        $(
            pub struct $x(());
        )*

        $(
            for _ in 0..(10_000 / 26) {
                spawn!(&mut $world,
                    Transform([1.0; 16]),
                    Position([1.0; 3]),
                    Rotation([1.0; 3]),
                    Velocity([1.0; 3]),
                    $x(()),
                );
            }
        )*
    }
}

#[derive(Copy, Clone)]
struct Transform([f32; 16]);
#[derive(Copy, Clone)]
struct Position([f32; 3]);
#[derive(Copy, Clone)]
struct Rotation([f32; 3]);
#[derive(Copy, Clone)]
struct Velocity([f32; 3]);

pub fn main() {
    for _ in 0..1_000_0 {
        let mut world = World::new();
        setup!(
            world, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
        );
    }
}
