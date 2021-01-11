use arche_tape::world::ComponentMeta;
use arche_tape::EcsId;
use arche_tape::FetchType;
use arche_tape::World;

struct Data(f32);

pub fn main() {
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
                    entity_builder = entity_builder
                        .with_dynamic_with_data(&mut bloat_d as *mut f32 as *mut u8, bloat_id);
                }

                let mut data = Data(1.0);
                entity_builder
                    .with_dynamic_with_data(&mut bloat_d as *mut f32 as *mut u8, *frag_id)
                    .with_dynamic_with_data(&mut data as *mut Data as *mut u8, data_id)
                    .with(Data(2.0))
                    .build();
            }
        }
    }

    #[inline(never)]
    fn bar(world: &mut World) {
        world
            .query::<(&mut Data,)>()
            .borrow()
            .for_each(|(data,)| data.0 *= 2.);
    }
    #[inline(never)]
    fn foo(world: &mut World, data_id: EcsId) {
        #[inline(never)]
        fn thingies(data: &mut Data) {
            dbg!("{}", data.0);
        }

        world
            .query_dynamic([FetchType::Mut(data_id)])
            .iter()
            .for_each(|[ptr]| {
                thingies(unsafe { &mut *{ ptr as *mut Data } });
            })
    }
    foo(&mut world, data_id);
    bar(&mut world);
}
