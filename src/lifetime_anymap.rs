use super::anymap::{AnyMap, AnyMapBorrow, AnyMapBorrowMut};
use std::error::Error;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct LifetimeAnyMapBorrow<'a, T: 'static> {
    borrow: AnyMapBorrow<'a, *mut T>,
}

impl<'a, T: 'static> LifetimeAnyMapBorrow<'a, T> {
    pub fn new(borrow: AnyMapBorrow<'a, *mut T>) -> Self {
        Self { borrow }
    }
}

impl<'a, T: 'static> Deref for LifetimeAnyMapBorrow<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = self.borrow.deref();
        unsafe { &**ptr }
    }
}

pub struct LifetimeAnyMapBorrowMut<'a, T: 'static> {
    borrow: AnyMapBorrowMut<'a, *mut T>,
}

impl<'a, T: 'static> LifetimeAnyMapBorrowMut<'a, T> {
    pub fn new(borrow: AnyMapBorrowMut<'a, *mut T>) -> Self {
        Self { borrow }
    }
}

impl<'a, T: 'static> Deref for LifetimeAnyMapBorrowMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = self.borrow.deref();
        unsafe { &**ptr }
    }
}

impl<'a, T: 'static> DerefMut for LifetimeAnyMapBorrowMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = self.borrow.deref_mut();
        unsafe { &mut **ptr }
    }
}

/// Stores non-static borrows on data in a TypeId -> Box<dyn Any> Hashmap
pub struct LifetimeAnyMap<'a> {
    map: AnyMap,
    phantom: PhantomData<&'a mut ()>,
}

impl<'a> LifetimeAnyMap<'a> {
    pub fn new() -> Self {
        Self {
            map: AnyMap::new(),
            phantom: PhantomData,
        }
    }

    pub fn insert<'this, T: 'static>(&'this mut self, data: &'a mut T) {
        let ptr: *mut T = data;
        self.map.insert(ptr);
    }

    pub fn get<'this, T: 'static>(
        &'this self,
    ) -> Result<LifetimeAnyMapBorrow<'this, T>, Box<dyn Error + 'this>> {
        let borrow = self.map.get::<*mut T>()?;
        Ok(LifetimeAnyMapBorrow::new(borrow))
    }

    pub fn get_mut<'this, T: 'static>(
        &'this self,
    ) -> Result<LifetimeAnyMapBorrowMut<'this, T>, Box<dyn Error + 'this>> {
        let borrow = self.map.get_mut::<*mut T>()?;
        Ok(LifetimeAnyMapBorrowMut::new(borrow))
    }
}
