pub mod anymap;
pub mod lifetime_anymap;

use anymap::{AnyMap, AnyMapBorrow, AnyMapBorrowMut};
use lifetime_anymap::{LifetimeAnyMap, LifetimeAnyMapBorrow, LifetimeAnyMapBorrowMut};

pub struct World {}

impl World {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) -> RunWorldContext {
        RunWorldContext {
            _world: self,
            temp_resources: LifetimeAnyMap::new(),
        }
    }
}

pub struct RunWorldContext<'run> {
    _world: &'run mut World,
    temp_resources: LifetimeAnyMap<'run>,
}

impl<'run> RunWorldContext<'run> {
    pub fn insert_temp_resource<'this, T: 'static>(&'this mut self, resource: &'run mut T) {
        self.temp_resources.insert(resource);
    }

    pub fn get_temp_resource<'a, T: 'static>(
        &'a self,
    ) -> Result<LifetimeAnyMapBorrow<'a, T>, Box<dyn std::error::Error + 'a>> {
        self.temp_resources.get()
    }

    pub fn get_temp_resource_mut<'a, T: 'static>(
        &'a self,
    ) -> Result<LifetimeAnyMapBorrowMut<'a, T>, Box<dyn std::error::Error + 'a>> {
        self.temp_resources.get_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn borrow_temp_resource_mut() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;
        run_ctx.insert_temp_resource(&mut foo);
        let mut foo_borrow = run_ctx.get_temp_resource_mut::<u32>().unwrap();
        *foo_borrow += 1;
    }

    #[test]
    pub fn borrow_temp_resource() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;
        run_ctx.insert_temp_resource(&mut foo);
        let _borrow = run_ctx.get_temp_resource::<u32>().unwrap();
    }

    #[test]
    pub fn borrow_temp_resource_mut_twice() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;

        run_ctx.insert_temp_resource(&mut foo);

        let _borrow_1 = run_ctx.get_temp_resource_mut::<u32>().unwrap();
        let borrow_2 = run_ctx.get_temp_resource_mut::<u32>();

        match &borrow_2 {
            Ok(_) => panic!("Should fail"),
            _ => (),
        }
    }

    #[test]
    pub fn borrow_temp_resource_twice() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;

        run_ctx.insert_temp_resource(&mut foo);

        let _1 = run_ctx.get_temp_resource::<u32>().unwrap();
        let _2 = run_ctx.get_temp_resource::<u32>().unwrap();
    }

    #[test]
    pub fn borrow_temp_resource_shared_and_mut() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;

        run_ctx.insert_temp_resource(&mut foo);

        let _borrow_1 = run_ctx.get_temp_resource::<u32>().unwrap();
        let borrow_2 = run_ctx.get_temp_resource_mut::<u32>();

        match &borrow_2 {
            Ok(_) => panic!("Should fail"),
            _ => (),
        }
    }

    #[test]
    pub fn multi_borrow() {
        let mut world = World::new();

        let mut run_ctx = world.run();
        let mut foo = 10_u32;

        run_ctx.insert_temp_resource(&mut foo);

        {
            let _borrow = run_ctx.get_temp_resource_mut::<u32>().unwrap();
        }

        {
            let _borrow = run_ctx.get_temp_resource_mut::<u32>().unwrap();
        }

        {
            let _borrow = run_ctx.get_temp_resource_mut::<u32>().unwrap();
        }

        {
            let _borrow = run_ctx.get_temp_resource_mut::<u32>().unwrap();
        }
    }
}
