use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct AnyMapBorrow<'a> {
    guard: RwLockReadGuard<'a, Box<dyn Any + 'static>>,
}

impl<'a> AnyMapBorrow<'a> {
    pub fn new(guard: RwLockReadGuard<'a, Box<dyn Any + 'static>>) -> Self {
        Self { guard }
    }
}

impl<'a> Deref for AnyMapBorrow<'a> {
    type Target = Box<dyn Any + 'static>;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

pub struct AnyMapBorrowMut<'a> {
    guard: RwLockWriteGuard<'a, Box<dyn Any + 'static>>,
}

impl<'a> AnyMapBorrowMut<'a> {
    pub fn new(guard: RwLockWriteGuard<'a, Box<dyn Any + 'static>>) -> Self {
        Self { guard }
    }
}

impl<'a> Deref for AnyMapBorrowMut<'a> {
    type Target = Box<dyn Any + 'static>;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<'a> DerefMut for AnyMapBorrowMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

pub struct AnyMap {
    map: HashMap<TypeId, RwLock<Box<dyn Any + 'static>>>,
}

impl AnyMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: TypeId, value: Box<dyn Any + 'static>) {
        self.map.insert(key, RwLock::new(value));
    }

    pub fn get<'a>(
        &'a self,
        type_id: TypeId,
    ) -> Result<AnyMapBorrow<'a>, Box<dyn std::error::Error + 'a>> {
        let entry = self
            .map
            .get(&type_id)
            .ok_or("Couldn't retrieve value from AnyMap")?;
        let read_guard = entry.try_read()?;
        Ok(AnyMapBorrow::new(read_guard))
    }

    pub fn get_mut<'a>(
        &'a self,
        type_id: TypeId,
    ) -> Result<AnyMapBorrowMut<'a>, Box<dyn std::error::Error + 'a>> {
        let entry = self
            .map
            .get(&type_id)
            .ok_or("Couldn't retrieve value from AnyMap")?;
        let read_guard = entry.try_write()?;
        Ok(AnyMapBorrowMut::new(read_guard))
    }
}

pub struct World {}

impl World {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) -> RunWorldContext {
        RunWorldContext {
            _world: self,
            temp_resources: AnyMap::new(),
        }
    }
}

pub struct TempResourceBorrow<'run, T: 'static> {
    guard: AnyMapBorrow<'run>,
    phantom: PhantomData<&'run &'static mut T>,
}

impl<'a, T: 'static> TempResourceBorrow<'a, T> {
    pub fn new(guard: AnyMapBorrow<'a>) -> Self {
        guard.downcast_ref::<&'static mut T>().unwrap();

        Self {
            guard,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Deref for TempResourceBorrow<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &**self.guard.downcast_ref::<&'static mut T>().unwrap()
    }
}

pub struct TempResourceBorrowMut<'run, T: 'static> {
    guard: AnyMapBorrowMut<'run>,
    phantom: PhantomData<&'run mut &'static mut T>,
}

impl<'a, T: 'static> TempResourceBorrowMut<'a, T> {
    pub fn new(mut guard: AnyMapBorrowMut<'a>) -> Self {
        guard.downcast_mut::<&'static mut T>().unwrap();

        Self {
            guard,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Deref for TempResourceBorrowMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &**self.guard.downcast_ref::<&'static mut T>().unwrap()
    }
}

impl<'a, T> DerefMut for TempResourceBorrowMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut **self.guard.downcast_mut::<&'static mut T>().unwrap()
    }
}

pub struct RunWorldContext<'run> {
    _world: &'run mut World,
    temp_resources: AnyMap,
}

impl<'run> RunWorldContext<'run> {
    pub fn insert_temp_resource<'a, 'b: 'run, T: 'static>(&'a mut self, resource: &'b mut T) {
        // SAFETY:
        // Safe as long as we dont ever move the &'static mut out of RunWorldContext since it's bounded by the 'run which the pointed to data must live for
        let resource: &'static mut T = unsafe { std::mem::transmute(resource) };

        let type_id = TypeId::of::<T>();
        self.temp_resources.insert(type_id, Box::new(resource));
    }

    pub fn get_temp_resource<'a, T: 'static>(
        &'a self,
    ) -> Result<TempResourceBorrow<'a, T>, Box<dyn std::error::Error + 'a>> {
        let id = TypeId::of::<T>();

        let resource = self.temp_resources.get(id)?;
        resource
            .downcast_ref::<&'static mut T>()
            .ok_or("Resource was not of correct type for key")?;
        Ok(TempResourceBorrow::new(resource))
    }

    pub fn get_temp_resource_mut<'a, T: 'static>(
        &'a self,
    ) -> Result<TempResourceBorrowMut<'a, T>, Box<dyn std::error::Error + 'a>> {
        let id = TypeId::of::<T>();

        let mut resource = self.temp_resources.get_mut(id)?;
        resource
            .downcast_mut::<&'static mut T>()
            .ok_or("Resource was not of correct type for key")?;
        Ok(TempResourceBorrowMut::new(resource))
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
}
