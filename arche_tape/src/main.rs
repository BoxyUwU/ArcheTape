use arche_tape::spawn;
use arche_tape::world::World;
fn main() {
    pub struct Data(f32);

    macro_rules! setup {
            ($world:ident, $($x:ident),*) => {
                $(
                    pub struct $x(f32);
                )*

                $(
                    for _ in 0..2000 {
                        spawn!(&mut $world, $x(0.), Data(1.));
                    }
                )*
            };
        }

    let mut world = World::new();
    setup!(
        world, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
    );
    for _ in 0..1_000_00 {
        for (data,) in world.query::<(&mut Data,)>().iter() {
            data.0 *= 2.0;
        }
    }
}
