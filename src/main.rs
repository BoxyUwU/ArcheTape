use ellecs::world::World;

pub struct A(f32);
pub struct B(f32);
pub struct C(f32);
pub struct D(f32);
pub struct E(f32);
pub struct F(f32);
pub struct G(f32);
pub struct H(f32);

fn main() {
    let mut world = World::new();
    for _ in 0..10_000 {
        world.spawn((A(1.), B(1.), C(1.), D(1.), E(1.), F(1.), G(1.), H(1.)));
    }
    let query = world.query::<(&mut A, &B, &mut C, &D, &mut E, &F, &mut G, &H)>();
    query
        .borrow()
        .into_for_each_mut(|(a, b, c, d, e, f, g, h)| {
            a.0 += b.0;
            c.0 += d.0;
            e.0 += f.0;
            g.0 += h.0;
        });
}
