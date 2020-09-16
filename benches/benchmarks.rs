use criterion::*;

pub mod frag_iter {
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

    pub struct Benchmark(World);

    impl Benchmark {
        pub fn new() -> Benchmark {
            let mut world = World::new();
            setup!(
                world, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
            );
            Benchmark(world)
        }

        pub fn run(&mut self) {
            for (data,) in &mut self.0.query::<(&mut Data,)>().borrow() {
                data.0 *= 2.;
            }
        }
    }
}

pub mod simple_iter {
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

    pub struct Benchmark(World);

    impl Benchmark {
        pub fn new() -> Self {
            let mut world = World::new();

            for _ in 0..10_000 {
                world.spawn((
                    Transform(Matrix4::from_scale(1.0)),
                    Position(Vector3::unit_x()),
                    Rotation(Vector3::unit_x()),
                    Velocity(Vector3::unit_x()),
                ))
            }

            Benchmark(world)
        }

        pub fn run(&mut self) {
            let query = self.0.query::<(&mut Position, &Velocity)>();
            for (position, velocity) in &mut query.borrow() {
                position.0 += velocity.0;
            }
        }
    }
}

pub fn ellecs(c: &mut Criterion) {
    let mut group = c.benchmark_group("ellecs");
    group.bench_function("frag_iter", |b| {
        let mut bench = frag_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("simple_iter", |b| {
        let mut bench = simple_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
}

criterion_group!(benchmarks, ellecs,);

criterion_main!(benchmarks);
