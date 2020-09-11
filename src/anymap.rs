use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::error::Error;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct AnyMapBorrow<'a, T: 'static> {
    pub guard: RwLockReadGuard<'a, Box<dyn Any>>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: 'static> AnyMapBorrow<'a, T> {
    fn new(guard: RwLockReadGuard<'a, Box<dyn Any>>) -> Self {
        Self {
            guard,
            phantom: PhantomData,
        }
    }
}

impl<'a, T: 'static> Deref for AnyMapBorrow<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.downcast_ref::<T>().unwrap()
    }
}

pub struct AnyMapBorrowMut<'a, T: 'static> {
    pub guard: RwLockWriteGuard<'a, Box<dyn Any>>,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T: 'static> AnyMapBorrowMut<'a, T> {
    fn new(guard: RwLockWriteGuard<'a, Box<dyn Any>>) -> Self {
        Self {
            guard,
            phantom: PhantomData,
        }
    }
}

impl<'a, T: 'static> Deref for AnyMapBorrowMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.downcast_ref::<T>().unwrap()
    }
}

impl<'a, T: 'static> DerefMut for AnyMapBorrowMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.downcast_mut::<T>().unwrap()
    }
}

pub struct AnyMap {
    map: HashMap<TypeId, RwLock<Box<dyn Any + 'static>>>,
}

impl<'a> AnyMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert<'this, T: 'static>(&'this mut self, data: T) {
        let type_id = TypeId::of::<T>();
        self.map.insert(type_id, RwLock::new(Box::new(data)));
    }

    pub fn get<'this, T: 'static>(
        &'this self,
    ) -> Result<AnyMapBorrow<'this, T>, Box<dyn Error + 'this>> {
        let type_id = TypeId::of::<T>();
        let lock = self
            .map
            .get(&type_id)
            .ok_or("Couldn't retrieve data from key")?;
        let read_guard = lock.try_read()?;
        let borrow = AnyMapBorrow::new(read_guard);
        Ok(borrow)
    }

    pub fn get_mut<'this, T: 'static>(
        &'this self,
    ) -> Result<AnyMapBorrowMut<'this, T>, Box<dyn Error + 'this>> {
        let type_id = TypeId::of::<T>();
        let lock = self
            .map
            .get(&type_id)
            .ok_or("Couldn't retrieve data from key")?;
        let write_guard = lock.try_write()?;
        let borrow = AnyMapBorrowMut::new(write_guard);
        Ok(borrow)
    }
}
