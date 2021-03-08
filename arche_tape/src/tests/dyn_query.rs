use crate::World;
use crate::{dyn_query::DynQuery, world::ComponentMeta};
use crate::{EcsId, FetchType};

#[test]
fn for_each_mut() {
    unsafe {
        let mut world = World::new();

        let u32_id = world
            .spawn_with_component_meta(ComponentMeta::from_generic::<u32>())
            .build();
        let u64_id = world
            .spawn_with_component_meta(ComponentMeta::from_generic::<u64>())
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 10_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 12_u64 } as *mut u64 as *mut u8, u64_id)
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 15_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 14_u64 } as *mut u64 as *mut u8, u64_id)
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 20_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 16_u64 } as *mut u64 as *mut u8, u64_id)
            .build();

        let mut query = world.query_dynamic([FetchType::Mut(u32_id), FetchType::Immut(u64_id)]);
        let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();

        for (left, right) in query
            .iter()
            .map(|[a, b]| (&mut *{ a as *mut u32 }, &*{ b as *mut u64 }))
        {
            assert_eq!(checks.next().unwrap(), (*left, *right));
        }
        assert_eq!(checks.next(), None);
    }
}

#[test]
fn for_each_subset_iterator() {
    unsafe {
        let mut world = World::new();

        let u32_id = world
            .spawn_with_component_meta(ComponentMeta::from_generic::<u32>())
            .build();
        let u64_id = world
            .spawn_with_component_meta(ComponentMeta::from_generic::<u64>())
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 10_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 12_u64 } as *mut u64 as *mut u8, u64_id)
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 15_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 14_u64 } as *mut u64 as *mut u8, u64_id)
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 20_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 16_u64 } as *mut u64 as *mut u8, u64_id)
            .build();

        let mut query = world.query_dynamic([FetchType::Mut(u32_id)]);
        let mut checks = vec![10, 15, 20].into_iter();

        for data in query.iter().map(|[a]| &mut *{ a as *mut u32 }) {
            assert_eq!(checks.next().unwrap(), *data);
        }
        assert_eq!(checks.next(), None);
    }
}

#[test]
fn for_each_multi_archetype_iterator() {
    unsafe {
        let mut world = World::new();

        let u32_id = world
            .spawn_with_component_meta(ComponentMeta::from_generic::<u32>())
            .build();
        let u64_id = world
            .spawn_with_component_meta(ComponentMeta::from_generic::<u64>())
            .build();
        let u128_id = world
            .spawn_with_component_meta(ComponentMeta::from_generic::<u128>())
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 10_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 12_u64 } as *mut u64 as *mut u8, u64_id)
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 15_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 14_u64 } as *mut u64 as *mut u8, u64_id)
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 20_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 16_u64 } as *mut u64 as *mut u8, u64_id)
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 11_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 12_u64 } as *mut u64 as *mut u8, u64_id)
            .with_dynamic_with_data({ &mut 99_u128 } as *mut u128 as *mut u8, u128_id)
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 16_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 14_u64 } as *mut u64 as *mut u8, u64_id)
            .with_dynamic_with_data({ &mut 99_u128 } as *mut u128 as *mut u8, u128_id)
            .build();

        world
            .spawn()
            .with_dynamic_with_data({ &mut 21_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 16_u64 } as *mut u64 as *mut u8, u64_id)
            .with_dynamic_with_data({ &mut 99_u128 } as *mut u128 as *mut u8, u128_id)
            .build();

        let mut query = world.query_dynamic([FetchType::Mut(u32_id)]);
        let mut checks = vec![10, 15, 20, 11, 16, 21].into_iter();

        for data in query.iter().map(|[ptr]| &mut *{ ptr as *mut u32 }) {
            assert_eq!(checks.next().unwrap(), *data);
        }
        assert!(checks.next().is_none());
    }
}

#[test]
fn query_param_in_func() {
    let mut world = World::new();
    let comp_id = world.spawn().build();
    world.spawn().with_dynamic(comp_id).build();
    let query = world.query_dynamic([FetchType::Immut(comp_id)]);
    fn func<const N: usize>(mut query: DynQuery<N>) {
        let mut ran = false;
        for _ in query.iter() {
            ran = true;
        }
        assert!(ran);
    }
    func(query);
}

#[test]
fn entity_query() {
    unsafe {
        let mut world = World::new();

        let u32_id = world
            .spawn_with_component_meta(ComponentMeta::from_generic::<u32>())
            .build();
        let u64_id = world
            .spawn_with_component_meta(ComponentMeta::from_generic::<u64>())
            .build();

        let entity = world
            .spawn()
            .with_dynamic_with_data({ &mut 1_u32 } as *mut u32 as *mut u8, u32_id)
            .with_dynamic_with_data({ &mut 12_u64 } as *mut u64 as *mut u8, u64_id)
            .build();

        let mut query = world.query_dynamic([
            FetchType::EcsId,
            FetchType::Immut(u32_id),
            FetchType::Immut(u64_id),
        ]);

        let mut checks = vec![(entity, 1, 12)].into_iter();
        for (entity, data1, data2) in query.iter().map(|[ecs_id, u32_data, u64_data]| {
            (&*{ ecs_id as *mut EcsId }, &*{ u32_data as *mut u32 }, &*{
                u64_data as *mut u64
            })
        }) {
            let (entity_check, data1_check, data2_check) = checks.next().unwrap();
            assert_eq!(*entity, entity_check);
            assert_eq!(*data1, data1_check);
            assert_eq!(*data2, data2_check);
        }
        assert!(checks.next().is_none());
    }
}

#[test]
fn non_present_component_query() {
    let mut other_world = World::new();
    let c_id = other_world.spawn().build();
    let world = World::new();
    world
        .query_dynamic([FetchType::Mut(c_id)])
        .iter()
        .for_each(|_| panic!());
    world
        .query_dynamic([FetchType::Immut(c_id)])
        .iter()
        .for_each(|_| panic!());
}
