use criterion::*;
use ellecs::spawn;

pub mod frag_iter_20_padding_20 {
    use super::spawn;
    use ellecs::world::World;

    pub struct Data(f32);

    macro_rules! setup {
        ($world:ident, (bloat: ($($y:ident,)*)), ($($x:ident),*)) => {
            $(
                pub struct $x(f32);
            )*

            $(
                pub struct $y(f32);
            )*

            $(
                for _ in 0..20 {
                    spawn_entity(&mut $world, $x);
                }
            )*

            fn spawn_entity<T: 'static>(world: &mut World, data: T) {
                spawn!(&mut world, data, $($y(2.),)* Data(1.));
            }
        };
    }

    pub struct Benchmark(World);

    impl Benchmark {
        pub fn new() -> Benchmark {
            let mut world = World::new();
            setup!(
                world,
                (bloat:
                    (
                        Bloat1,
                        Bloat2,
                        Bloat3,
                        Bloat4,
                        Bloat5,
                        Bloat6,
                        Bloat7,
                        Bloat8,
                        Bloat9,
                        Bloat10,
                        Bloat11,
                        Bloat12,
                        Bloat13,
                        Bloat14,
                        Bloat15,
                        Bloat16,
                        Bloat17,
                        Bloat18,
                        Bloat19,
                        Bloat20,
                    )),
                (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)
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

pub mod frag_iter_2000 {
    use ellecs::spawn;
    use ellecs::world::World;

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

pub mod frag_iter_200 {
    use ellecs::spawn;
    use ellecs::world::World;

    pub struct Data(f32);

    macro_rules! setup {
        ($world:ident, $($x:ident),*) => {
            $(
                pub struct $x(f32);
            )*

            $(
                for _ in 0..200 {
                    spawn!(&mut $world, $x(0.), Data(1.));
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

pub mod frag_iter_20 {
    use ellecs::spawn;
    use ellecs::world::World;

    pub struct Data(f32);

    macro_rules! setup {
        ($world:ident, $($x:ident),*) => {
            $(
                pub struct $x(f32);
            )*

            $(
                for _ in 0..20 {
                    spawn!(&mut $world, $x(0.), Data(1.));
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

    pub struct Benchmark(World);

    impl Benchmark {
        pub fn new() -> Self {
            let mut world = World::new();

            for _ in 0..10_000 {
                spawn!(
                    &mut world,
                    Transform(Matrix4::from_scale(1.0)),
                    Position(Vector3::unit_x()),
                    Rotation(Vector3::unit_x()),
                    Velocity(Vector3::unit_x()),
                );
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

    pub struct Benchmark();

    impl Benchmark {
        pub fn new() -> Self {
            Benchmark()
        }

        pub fn run(&mut self) {
            let mut world = World::new();

            for _ in 0..10_000 {
                spawn!(
                    &mut world,
                    Transform(Matrix4::from_scale(1.0)),
                    Position(Vector3::unit_x()),
                    Rotation(Vector3::unit_x()),
                    Velocity(Vector3::unit_x()),
                );
            }
        }
    }
}

pub mod frag_insert {
    use cgmath::*;
    use ellecs::spawn;
    use ellecs::world::World;

    macro_rules! setup {
        ($world:ident, $($x:ident),*) => {
            $(
                pub struct $x(f32);
            )*

            $(
                for _ in 0..1_000 {
                    spawn!(&mut $world,
                        Transform(Matrix4::from_scale(1.0)),
                        Position(Vector3::unit_x()),
                        Rotation(Vector3::unit_x()),
                        Velocity(Vector3::unit_x()),
                        $x(1.),
                    );
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
    use ellecs::spawn;
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
                spawn!(
                    &mut world,
                    A(1.),
                    B(1.),
                    C(1.),
                    D(1.),
                    E(1.),
                    F(1.),
                    G(1.),
                    H(1.),
                );
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

pub mod add_remove {
    use ellecs::entities::Entity;
    use ellecs::spawn;
    use ellecs::world::World;

    #[derive(Copy, Clone)]
    struct A(f32);
    #[derive(Copy, Clone)]
    struct B(f32);

    pub struct Benchmark(World, Box<[Entity]>);

    impl Benchmark {
        pub fn new() -> Self {
            let mut world = World::new();
            let mut entities = Vec::with_capacity(10000);

            for _ in 0..10_000 {
                entities.push(spawn!(&mut world, A(1.)));
            }

            Benchmark(world, entities.into_boxed_slice())
        }

        pub fn run(&mut self) {
            for &entity in self.1.iter() {
                self.0.add_component(entity, B(1.));
            }
            for &entity in self.1.iter() {
                self.0.remove_component::<B>(entity);
            }
        }
    }
}

pub mod padded_add_remove {
    use ellecs::entities::Entity;
    use ellecs::spawn;
    use ellecs::world::World;

    struct Padding([u8; 1024]);
    struct A(f32);
    struct B(f32);

    pub struct Benchmark(World, Box<[Entity]>);

    impl Benchmark {
        pub fn new() -> Self {
            let mut world = World::new();
            let mut entities = Vec::with_capacity(10000);

            for _ in 0..10_000 {
                entities.push(spawn!(&mut world, Padding([0; 1024]), A(1.)));
            }

            Benchmark(world, entities.into_boxed_slice())
        }

        pub fn run(&mut self) {
            for &entity in self.1.iter() {
                self.0.add_component(entity, B(1.));
            }
            for &entity in self.1.iter() {
                self.0.remove_component::<B>(entity);
            }
        }
    }
}

pub mod wide_add_remove {
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

    pub struct Benchmark(World, Box<[Entity]>);

    impl Benchmark {
        pub fn new() -> Self {
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

            Benchmark(world, entities.into_boxed_slice())
        }

        pub fn run(&mut self) {
            for &entity in self.1.iter() {
                self.0.add_component(entity, B(1.));
            }
            for &entity in self.1.iter() {
                self.0.remove_component::<B>(entity);
            }
        }
    }
}

pub mod get {
    use ellecs::entities::Entity;
    use ellecs::spawn;
    use ellecs::world::World;

    pub struct A(f32);

    pub struct Benchmark(World, Box<[Entity]>);

    impl Benchmark {
        pub fn new() -> Self {
            let mut world = World::new();
            let mut entities = Vec::with_capacity(10_000);

            for _ in 0..10_000 {
                let entity = spawn!(&mut world, A(10.0));
                entities.push(entity);
            }

            Benchmark(world, entities.into_boxed_slice())
        }

        pub fn run(&mut self) {
            for &entity in self.1.iter() {
                let a = self.0.get_component_mut::<A>(entity);
                criterion::black_box(a);
            }
        }
    }
}

pub mod padded_get {
    use ellecs::entities::Entity;
    use ellecs::spawn;
    use ellecs::world::World;

    pub struct Data(f32);

    macro_rules! create_entities {
        ($world:ident; $entities:ident; $($dummy:ident),*) => {
            $(pub struct $dummy(f32);)*

            for _ in 0..10_000 {
                let entity = spawn!(&mut $world, $($dummy(1.0)),*);
                $entities.push(entity);

                $world.add_component(entity, Data(10.0));
            }
        };
    }

    pub struct Benchmark(World, Box<[Entity]>);

    impl Benchmark {
        pub fn new() -> Self {
            let mut world = World::new();
            let mut entities = Vec::with_capacity(10_000);
            create_entities!(world; entities; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

            Benchmark(world, entities.into_boxed_slice())
        }

        pub fn run(&mut self) {
            for &entity in self.1.iter() {
                let data: &mut Data = self.0.get_component_mut(entity).unwrap();
                criterion::black_box(data);
            }
        }
    }
}

pub fn ellecs(c: &mut Criterion) {
    let mut group = c.benchmark_group("ellecs");
    group.bench_function("add_remove_10_000", |b| {
        let mut bench = add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("padded_add_remove_10_000", |b| {
        let mut bench = padded_add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("wide_remove_10_000", |b| {
        let mut bench = wide_add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("padded_get", |b| {
        let mut bench = padded_get::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("get", |b| {
        let mut bench = get::Benchmark::new();
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
    group.bench_function("frag_iter_20_padding_20", |b| {
        let mut bench = frag_iter_20_padding_20::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("frag_iter_200_entity", |b| {
        let mut bench = frag_iter_200::Benchmark::new();
        b.iter(move || bench.run());
    });
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
}

criterion_group!(benchmarks, ellecs);

criterion_main!(benchmarks);
