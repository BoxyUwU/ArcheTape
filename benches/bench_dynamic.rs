use arche_tape::world::ComponentMeta;
use arche_tape::EcsId;
use arche_tape::FetchType;
use arche_tape::World;

pub mod frag_iter_20_padding_20 {
    use super::*;
    pub struct Data(f32);
    pub struct Benchmark(World, EcsId);

    impl Benchmark {
        pub fn new() -> Benchmark {
            let mut world = World::new();

            let bloat_cs = { 0..20 }
                .map(|_| unsafe {
                    world
                        .spawn_with_component_meta(ComponentMeta::from_size_align(4, 4))
                        .build()
                })
                .collect::<Box<[EcsId]>>();

            let frag_cs = { 0..26 }
                .map(|_| unsafe {
                    world
                        .spawn_with_component_meta(ComponentMeta::from_size_align(4, 4))
                        .build()
                })
                .collect::<Box<[EcsId]>>();

            let data_id = unsafe {
                world
                    .spawn_with_component_meta(ComponentMeta::from_generic::<Data>())
                    .build()
            };

            for frag_id in &*frag_cs {
                for _ in 0..20 {
                    let mut bloat_d = 0.0_f32;
                    let mut entity_builder = world.spawn();

                    unsafe {
                        for &bloat_id in &*bloat_cs {
                            entity_builder = entity_builder.with_dynamic_with_data(
                                &mut bloat_d as *mut f32 as *mut u8,
                                bloat_id,
                            );
                        }

                        let mut data = Data(1.0);
                        entity_builder
                            .with_dynamic_with_data(&mut bloat_d as *mut f32 as *mut u8, *frag_id)
                            .with_dynamic_with_data(&mut data as *mut Data as *mut u8, data_id)
                            .build();
                    }
                }
            }

            Benchmark(world, data_id)
        }

        pub fn run(&mut self) {
            self.0
                .query_dynamic([FetchType::Mut(self.1)])
                .iter()
                .for_each(|[ptr]| unsafe { &mut *{ ptr as *mut Data } }.0 *= 2.);
        }
    }
}

pub mod frag_iter_20 {
    use super::*;
    pub struct Data(f32);
    pub struct Benchmark(World, EcsId);

    impl Benchmark {
        pub fn new() -> Benchmark {
            let mut world = World::new();

            let frag_cs = { 0..26 }
                .map(|_| unsafe {
                    world
                        .spawn_with_component_meta(ComponentMeta::from_size_align(4, 4))
                        .build()
                })
                .collect::<Box<[EcsId]>>();

            let data_id = unsafe {
                world
                    .spawn_with_component_meta(ComponentMeta::from_generic::<Data>())
                    .build()
            };

            for frag_id in &*frag_cs {
                for _ in 0..20 {
                    let entity_builder = world.spawn();

                    let mut frag_data = 0.0_f32;
                    unsafe {
                        let mut data = Data(1.0);
                        entity_builder
                            .with_dynamic_with_data(&mut frag_data as *mut f32 as *mut u8, *frag_id)
                            .with_dynamic_with_data(&mut data as *mut Data as *mut u8, data_id)
                            .build();
                    }
                }
            }

            Benchmark(world, data_id)
        }

        pub fn run(&mut self) {
            self.0
                .query_dynamic([FetchType::Mut(self.1)])
                .iter()
                .for_each(|[ptr]| unsafe { &mut *{ ptr as *mut Data } }.0 *= 2.);
        }
    }
}

pub mod frag_iter_200 {
    use super::*;
    pub struct Data(f32);
    pub struct Benchmark(World, EcsId);

    impl Benchmark {
        pub fn new() -> Benchmark {
            let mut world = World::new();

            let frag_cs = { 0..26 }
                .map(|_| unsafe {
                    world
                        .spawn_with_component_meta(ComponentMeta::from_size_align(4, 4))
                        .build()
                })
                .collect::<Box<[EcsId]>>();

            let data_id = unsafe {
                world
                    .spawn_with_component_meta(ComponentMeta::from_generic::<Data>())
                    .build()
            };

            for frag_id in &*frag_cs {
                for _ in 0..200 {
                    let entity_builder = world.spawn();

                    let mut frag_data = 0.0_f32;
                    unsafe {
                        let mut data = Data(1.0);
                        entity_builder
                            .with_dynamic_with_data(&mut frag_data as *mut f32 as *mut u8, *frag_id)
                            .with_dynamic_with_data(&mut data as *mut Data as *mut u8, data_id)
                            .build();
                    }
                }
            }

            Benchmark(world, data_id)
        }

        pub fn run(&mut self) {
            self.0
                .query_dynamic([FetchType::Mut(self.1)])
                .iter()
                .for_each(|[ptr]| unsafe { &mut *{ ptr as *mut Data } }.0 *= 2.);
        }
    }
}

pub mod frag_iter_2000 {
    use super::*;
    pub struct Data(f32);
    pub struct Benchmark(World, EcsId);

    impl Benchmark {
        pub fn new() -> Benchmark {
            let mut world = World::new();

            let frag_cs = { 0..26 }
                .map(|_| unsafe {
                    world
                        .spawn_with_component_meta(ComponentMeta::from_size_align(4, 4))
                        .build()
                })
                .collect::<Box<[EcsId]>>();

            let data_id = unsafe {
                world
                    .spawn_with_component_meta(ComponentMeta::from_generic::<Data>())
                    .build()
            };

            for frag_id in &*frag_cs {
                for _ in 0..2000 {
                    let entity_builder = world.spawn();

                    let mut frag_data = 0.0_f32;
                    unsafe {
                        let mut data = Data(1.0);
                        entity_builder
                            .with_dynamic_with_data(&mut frag_data as *mut f32 as *mut u8, *frag_id)
                            .with_dynamic_with_data(&mut data as *mut Data as *mut u8, data_id)
                            .build();
                    }
                }
            }

            Benchmark(world, data_id)
        }

        pub fn run(&mut self) {
            self.0
                .query_dynamic([FetchType::Mut(self.1)])
                .iter()
                .for_each(|[ptr]| unsafe { &mut *{ ptr as *mut Data } }.0 *= 2.);
        }
    }
}

pub mod simple_iter {
    use super::*;
    use cgmath::*;

    #[derive(Copy, Clone)]
    struct Transform(Matrix4<f32>);
    #[derive(Copy, Clone)]
    struct Position(Vector3<f32>);
    #[derive(Copy, Clone)]
    struct Rotation(Vector3<f32>);
    #[derive(Copy, Clone)]
    struct Velocity(Vector3<f32>);

    pub struct Benchmark(World, EcsId, EcsId);

    impl Benchmark {
        pub fn new() -> Self {
            unsafe {
                let mut world = World::new();

                let transform_id = world
                    .spawn_with_component_meta(ComponentMeta::from_generic::<Transform>())
                    .build();
                let position_id = world
                    .spawn_with_component_meta(ComponentMeta::from_generic::<Position>())
                    .build();
                let rotation_id = world
                    .spawn_with_component_meta(ComponentMeta::from_generic::<Rotation>())
                    .build();
                let velocity_id = world
                    .spawn_with_component_meta(ComponentMeta::from_generic::<Velocity>())
                    .build();

                let mut transform = Transform(Matrix4::from_scale(1.0));
                let mut position = Position(Vector3::unit_x());
                let mut rotation = Rotation(Vector3::unit_x());
                let mut velocity = Velocity(Vector3::unit_x());

                for _ in 0..10_000 {
                    world
                        .spawn()
                        .with_dynamic_with_data(
                            &mut transform as *mut Transform as *mut u8,
                            transform_id,
                        )
                        .with_dynamic_with_data(
                            &mut position as *mut Position as *mut u8,
                            position_id,
                        )
                        .with_dynamic_with_data(
                            &mut rotation as *mut Rotation as *mut u8,
                            rotation_id,
                        )
                        .with_dynamic_with_data(
                            &mut velocity as *mut Velocity as *mut u8,
                            velocity_id,
                        )
                        .build();
                }

                Benchmark(world, position_id, velocity_id)
            }
        }

        pub fn run(&mut self) {
            self.0
                .query_dynamic([FetchType::Mut(self.1), FetchType::Mut(self.2)])
                .iter()
                .for_each(|[pos, vel]| {
                    let (pos, vel) =
                        unsafe { (&mut *(pos as *mut Position), &mut *(vel as *mut Velocity)) };
                    pos.0 += vel.0;
                });
        }
    }
}

pub mod simple_large_iter {
    use super::*;

    pub struct Benchmark(World, Box<[EcsId]>);

    impl Benchmark {
        pub fn new() -> Self {
            let mut world = World::new();

            let ids = (0..8)
                .map(|_| unsafe {
                    world
                        .spawn_with_component_meta(ComponentMeta::from_generic::<f32>())
                        .build()
                })
                .collect::<Box<[_]>>();

            for _ in 0..10_000 {
                let mut data = 1.0_f32;
                unsafe {
                    let mut builder = world.spawn();
                    for n in 0..8 {
                        builder =
                            builder.with_dynamic_with_data(&mut data as *mut f32 as *mut u8, ids[n])
                    }
                    builder.build();
                }
            }
            Benchmark(world, ids)
        }

        pub fn run(&mut self) {
            self.0
                .query_dynamic([
                    FetchType::Mut(self.1[0]),
                    FetchType::Immut(self.1[1]),
                    FetchType::Mut(self.1[2]),
                    FetchType::Immut(self.1[3]),
                    FetchType::Mut(self.1[4]),
                    FetchType::Immut(self.1[5]),
                    FetchType::Mut(self.1[6]),
                    FetchType::Immut(self.1[7]),
                ])
                .iter()
                .map(|[a, b, c, d, e, f, g, h]| {
                    [
                        a as *mut f32,
                        b as *mut f32,
                        c as *mut f32,
                        d as *mut f32,
                        e as *mut f32,
                        f as *mut f32,
                        g as *mut f32,
                        h as *mut f32,
                    ]
                })
                .map(|[a, b, c, d, e, f, g, h]| unsafe {
                    (&mut *a, &*b, &mut *c, &*d, &mut *e, &*f, &mut *g, &*h)
                })
                .for_each(|(a, b, c, d, e, f, g, h)| {
                    *a += *b;
                    *c += *d;
                    *e += *f;
                    *g += *h;
                });
        }
    }
}
