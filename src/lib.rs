/*
#![feature(box_syntax)]
#![feature(libstd_sys_internals)]

pub fn main() {
    use world::World;

    let mut world = World::new();

    world.spawn((10_u32, 12_u64));
    world.spawn((15_u32, 14_u64));
    world.spawn((20_u32, 16_u64));

    let query = world.query::<(&mut u32, &u64)>();

    for (left, right) in &mut query.borrow() {
        println!("{}, {}", left, right);
    }
}
*/

pub mod anymap;
pub mod archetype_iter;
pub mod bundle;
pub mod lifetime_anymap;
pub mod world;
