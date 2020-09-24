use ellecs::world::World;

pub struct Data(f32);

macro_rules! setup {
        ($world:ident, $($x:ident),*) => {
            $(
                pub struct $x(f32);
            )*

            $(
                for _ in 0..20 {
                    $world.spawn(($x(0.), Data(1.)));
                }
            )*
        };
    }

pub fn main() {
    let mut world = World::new();
    setup!(world, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

    for _ in 0..1_000_000 {
        world.query::<(&mut Data,)>().borrow().for_each(|(data,)| {
            data.0 *= 2.;
        });
    }
}
