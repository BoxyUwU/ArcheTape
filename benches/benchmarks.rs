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
            self.0.query::<(&mut Data,)>().borrow().for_each(|(data,)| {
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
            self.0.query::<(&mut Data,)>().borrow().for_each(|(data,)| {
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
                ));
            }

            Benchmark(world)
        }

        pub fn run(&mut self) {
            self.0
                .query::<(&mut Position, &mut Velocity)>()
                .borrow()
                .for_each(|(pos, vel)| {
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
                ));
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
    use ellecs::world::World;

    pub struct A(f32);
    pub struct B(f32);
    pub struct C(f32);
    pub struct D(f32);
    pub struct E(f32);
    pub struct F(f32);
    pub struct G(f32);
    pub struct H(f32);

    pub struct Benchmark(World);

    impl Benchmark {
        pub fn new() -> Self {
            let mut world = World::new();
            for _ in 0..10_000 {
                world.spawn((A(1.), B(1.), C(1.), D(1.), E(1.), F(1.), G(1.), H(1.)));
            }
            Benchmark(world)
        }

        pub fn run(&mut self) {
            let query = self
                .0
                .query::<(&mut A, &B, &mut C, &D, &mut E, &F, &mut G, &H)>();
            query.borrow().for_each(|(a, b, c, d, e, f, g, h)| {
                a.0 += b.0;
                c.0 += d.0;
                e.0 += f.0;
                g.0 += h.0;
            });
        }
    }
}

pub fn ellecs(c: &mut Criterion) {
    let mut group = c.benchmark_group("ellecs");
    group.bench_function("frag_iter_2000_entity", |b| {
        let mut bench = frag_iter_2000::Benchmark::new();
        b.iter(move || bench.run());
    });
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
}

criterion_group!(benchmarks, ellecs);

criterion_main!(benchmarks);
