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
