use criterion::*;

pub mod frag_iter_2000 {
    use ellecs::world::World;

    pub struct Data(f32);

    macro_rules! setup {
        ($world:ident, $($x:ident),*) => {
            $(
                pub struct $x(f32);
            )*

            $(
                for _ in 0..2000 {
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
            self.0
                .query::<(&mut Data,)>()
                .borrow()
                .into_for_each_mut(|(data,)| {
                    data.0 *= 2.0;
                });
        }
    }
}

pub mod frag_iter_20 {
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
            self.0
                .query::<(&mut Data,)>()
                .borrow()
                .into_for_each_mut(|(data,)| {
                    data.0 *= 2.;
                });
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
            self.0
                .query::<(&mut Position, &mut Velocity)>()
                .borrow()
                .into_for_each_mut(|(pos, vel)| {
                    pos.0 += vel.0;
                });
        }
    }
}

pub mod simple_insert {
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

    pub struct Benchmark();

    impl Benchmark {
        pub fn new() -> Self {
            Benchmark()
        }

        pub fn run(&mut self) {
            let mut world = World::new();

            for _ in 0..10_000 {
                world.spawn((
                    Transform(Matrix4::from_scale(1.0)),
                    Position(Vector3::unit_x()),
                    Rotation(Vector3::unit_x()),
                    Velocity(Vector3::unit_x()),
                ))
            }
        }
    }
}

pub mod frag_insert {
    use cgmath::*;
    use ellecs::world::World;

    macro_rules! setup {
        ($world:ident, $($x:ident),*) => {
            $(
                pub struct $x(f32);
            )*

            $(
                for _ in 0..1_000 {
                    $world.spawn((
                        Transform(Matrix4::from_scale(1.0)),
                        Position(Vector3::unit_x()),
                        Rotation(Vector3::unit_x()),
                        Velocity(Vector3::unit_x()),
                        $x(1.),
                    ));
                }
            )*
        }
    }

    #[derive(Copy, Clone)]
    struct Transform(Matrix4<f32>);
    #[derive(Copy, Clone)]
    struct Position(Vector3<f32>);
    #[derive(Copy, Clone)]
    struct Rotation(Vector3<f32>);
    #[derive(Copy, Clone)]
    struct Velocity(Vector3<f32>);

    pub struct Benchmark();

    impl Benchmark {
        pub fn new() -> Self {
            Benchmark()
        }

        pub fn run(&mut self) {
            let mut world = World::new();
            setup!(
                world, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
            );
        }
    }
}

pub mod simple_large_iter {
    use cgmath::*;
    use ellecs::world::World;

    pub struct A(u8);
    pub struct B(u8);
    pub struct C(u8);
    pub struct D(u8);
    pub struct E(u8);
    pub struct F(u8);
    pub struct G(u8);
    pub struct H(u8);

    pub struct Benchmark(World);

    impl Benchmark {
        pub fn new() -> Self {
            let mut world = World::new();
            for _ in 0..100_000 {
                world.spawn((A(1), B(1), C(1), D(1), E(1), F(1), G(1), H(1)));
            }
            Benchmark(world)
        }

        pub fn run(&mut self) {
            let query = self.0.query::<(&A, &B, &C, &D, &E, &F, &G, &H)>();
            query
                .borrow()
                .into_for_each_mut(|(_a, _b, _c, _d, _e, _f, _g, _h)| {});
        }
    }
}

pub fn ellecs(c: &mut Criterion) {
    let mut group = c.benchmark_group("ellecs");
    group.bench_function("simple_large_iter", |b| {
        let mut bench = simple_large_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("simple_insert_10_000", |b| {
        let mut bench = simple_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("frag_insert_1_000_x_26", |b| {
        let mut bench = frag_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("simple_iter", |b| {
        let mut bench = simple_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("frag_iter_20_entity", |b| {
        let mut bench = frag_iter_20::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("frag_iter_2000_entity", |b| {
        let mut bench = frag_iter_2000::Benchmark::new();
        b.iter(move || bench.run());
    });
}

criterion_group!(benchmarks, ellecs);

criterion_main!(benchmarks);
