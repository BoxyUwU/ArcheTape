#![allow(clippy::bool_comparison)]
#![deny(unsafe_op_in_unsafe_fn)]

use std::{
    alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout},
    mem::MaybeUninit,
    ptr::NonNull,
};

#[derive(Clone, Debug)] // If we ever add a Hash impl we need to do it manually because of the custom Eq/PartialEq impls
pub struct TypeInfo {
    pub layout: Layout,
    pub drop_fn: Option<fn(*mut MaybeUninit<u8>)>,
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.layout == other.layout
    }
}

impl Eq for TypeInfo {}

impl TypeInfo {
    pub fn new(layout: Layout, drop_fn: Option<fn(*mut MaybeUninit<u8>)>) -> TypeInfo {
        Self { layout, drop_fn }
    }

    pub fn dangling(&self) -> NonNull<u8> {
        NonNull::new(self.layout.align() as *mut u8).unwrap()
    }
}

pub struct UntypedVec {
    type_info: TypeInfo,
    data: NonNull<u8>,
    cap: usize, // In bytes
    len: usize, // In bytes, if cap is 0 then a len > 0 implies ZST
}

impl UntypedVec {
    pub fn new_from_untyped_vec(from: &mut UntypedVec) -> Self {
        // Safe because the passed in untyped vec was either made safely or with unsafe code
        unsafe { Self::new_from_raw(from.type_info.clone()) }
    }

    /// # Safety
    ///
    ///    TypeInfo::drop_fn must take a pointer to a MaybeUninit<u8> and call the `Drop` impl of the type that TypeInfo::layout corresponds to.
    ///    If your type doesnt have a Drop trait implementation then this can just be None.
    ///    Make sure that the used EcsId corresponds correctly to the provided TypeInfo
    pub unsafe fn new_from_raw(type_info: TypeInfo) -> Self {
        Self {
            data: type_info.dangling(),
            type_info,
            cap: 0,
            len: 0,
        }
    }

    pub fn get_type_info(&self) -> TypeInfo {
        self.type_info.clone()
    }

    pub fn len(&self) -> usize {
        if self.type_info.layout.size() == 0 {
            return self.len;
        }

        assert!(self.len % self.type_info.layout.size() == 0);
        self.len / self.type_info.layout.size()
    }

    /// Length in bytes
    pub fn raw_len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn realloc(&mut self) {
        if self.type_info.layout.size() == 0 {
            panic!("Attempted to reallocate an UntypedVec who's data is size 0");
        }

        if self.cap == 0 {
            let new_cap = self.type_info.layout.size() * 4;

            let layout = Layout::from_size_align(new_cap, self.type_info.layout.align()).unwrap();
            // Safe because type info size is always non-zero and thus new_cap is always non-zero
            let ptr = unsafe { alloc(layout) };
            self.data = NonNull::new(ptr).unwrap_or_else(|| handle_alloc_error(layout));

            self.cap = new_cap;
        } else {
            let new_cap = self.cap * 2;
            assert!(new_cap < isize::MAX as usize);
            let old_layout =
                Layout::from_size_align(self.cap, self.type_info.layout.align()).unwrap();

            // Safe because the pointer we pass in is always made from this allocator because
            // the only way to get a cap > 0 is if the other branch has run and allocated memory
            // the layout is also safe because cap is always greater than zero here
            // Safe because new_cap is < isize::MAX
            let ptr = unsafe { realloc(self.data.as_ptr(), old_layout, new_cap) };
            self.data = NonNull::new(ptr).unwrap_or_else(|| handle_alloc_error(old_layout));

            self.cap = new_cap;
        }
    }

    /// # Safety
    ///
    ///   The data at src must not be used after calling this function
    ///
    ///   The data at src should be NonNull, aligned to type_info.layout.align() and should be the size given by type_info.layout.size() provided upon creation
    ///
    ///   The data must be a valid instance of the type that type_info.id represents
    #[allow(unused_unsafe)]
    pub unsafe fn push_raw(&mut self, src: *mut MaybeUninit<u8>) {
        assert!(src.is_null() == false);

        if self.type_info.layout.size() == 0 {
            self.len += 1;
            return;
        }

        // A realloc is guaranteed to make enough room to push to data because the initial allocation is of
        // type_info.layout.size() * 4, which means that a realloc will always allocate more than the required bytes
        if self.len + self.type_info.layout.size() > self.cap {
            self.realloc();
        }

        // Safe because we are offsetting within allocated memory and cap is < isize::MAX
        let dst: *mut u8 = unsafe { self.data.as_ptr().add(self.len) };
        let dst = dst as *mut MaybeUninit<u8>;

        unsafe {
            // The pointers are guaranteed to be nonoverlapping as we are writing to uninitialised memory in the vec
            std::ptr::copy_nonoverlapping(src, dst, self.type_info.layout.size());
        }

        self.len += self.type_info.layout.size();
    }

    pub fn get_raw(&self, element: usize) -> Option<*const u8> {
        if self.len == 0 {
            return None;
        }

        if self.type_info.layout.size() == 0 {
            if element < self.len {
                Some(self.data.as_ptr())
            } else {
                None
            }
        } else {
            assert!(self.len % self.type_info.layout.size() == 0);
            if element < { self.len / self.type_info.layout.size() } {
                unsafe {
                    Some(
                        self.data
                            .as_ptr()
                            .add(element * self.type_info.layout.size()),
                    )
                }
            } else {
                None
            }
        }
    }

    pub fn get_mut_raw(&mut self, element: usize) -> Option<*mut u8> {
        if self.len == 0 {
            return None;
        }

        if self.type_info.layout.size() == 0 {
            if element < self.len {
                Some(self.data.as_ptr())
            } else {
                None
            }
        } else {
            assert!(self.len % self.type_info.layout.size() == 0);
            if element < { self.len / self.type_info.layout.size() } {
                unsafe {
                    Some(
                        self.data
                            .as_ptr()
                            .add(element * self.type_info.layout.size()),
                    )
                }
            } else {
                None
            }
        }
    }

    /// Returns true if a value was popped
    pub fn pop(&mut self) -> bool {
        if self.type_info.layout.size() == 0 && self.len > 0 {
            self.len -= 1;
            let ptr = self.data.as_ptr();
            let ptr = ptr as *mut MaybeUninit<u8>;

            if let Some(drop_fn) = self.type_info.drop_fn {
                drop_fn(ptr);
            }
            true
        } else if self.len >= self.type_info.layout.size() {
            self.len -= self.type_info.layout.size();
            let ptr = self.data.as_ptr();
            // Safe because we're offsetting inside of the allocation
            let ptr: *mut u8 = unsafe { ptr.add(self.len) };
            let ptr = ptr as *mut MaybeUninit<u8>;

            if let Some(drop_fn) = self.type_info.drop_fn {
                drop_fn(ptr);
            }
            true
        } else {
            false
        }
    }

    /// # Safety
    ///
    ///  The other UntypedVec must be of the same type
    pub unsafe fn swap_move_element_to_other_vec(
        &mut self,
        other: &mut UntypedVec,
        element: usize,
    ) {
        assert!(self.type_info == other.type_info);
        assert!(self.len > 0);
        assert!(
            self.type_info.layout.size() == 0 || element < self.len / self.type_info.layout.size()
        );

        let data: *mut MaybeUninit<u8> = self.data.as_ptr() as *mut MaybeUninit<u8>;

        if self.type_info.layout.size() == 0 {
            self.len -= 1;
            other.len += 1;
        } else if element == (self.len / self.type_info.layout.size()) - 1 {
            unsafe {
                // Safe because we're offsetting inside the allocation and len is never >= isize::MAX
                let to_move = data.add(element * self.type_info.layout.size());
                // Safe because we assert that the type_info for self and other are the same.
                // Safe because we reduce the length of this vec by one which is effectively mem::forget
                other.push_raw(to_move);
            }

            self.len -= self.type_info.layout.size();
        } else {
            unsafe {
                // Safe because we're offsetting inside the allocation and len is never >= isize::MAX
                let to_move = data.add(element * self.type_info.layout.size());
                let to_swap = data
                    .add(self.len)
                    .offset(-(self.type_info.layout.size() as isize));

                // Safe because moving the last entry in the vec happens in the other branch
                std::ptr::swap_nonoverlapping(to_move, to_swap, self.type_info.layout.size());
                // Safe because we assert that the type_info for self and other are the same.
                // Safe because we assert that byte_index is aligned to self.type_info.layout.align()
                // Safe because we reduce the length of this vec by one which means we wont touch the data again
                other.push_raw(to_swap);
            }

            self.len -= self.type_info.layout.size();
        }
    }

    pub fn swap_remove(&mut self, element: usize) {
        assert!(self.len > 0);

        assert!(
            self.type_info.layout.size() == 0 || element < self.len / self.type_info.layout.size()
        );

        let data = self.data.as_ptr() as *mut MaybeUninit<u8>;

        if self.type_info.layout.size() == 0
            || element == self.len / self.type_info.layout.size() - 1
        {
            assert!(self.len > element);
        } else {
            // Safe because we're offsetting inside the allocation and len is never >= isize::MAX
            let to_move = unsafe { data.add(element * self.type_info.layout.size()) };
            let to_swap = unsafe { data.add(self.len).sub(self.type_info.layout.size()) };

            unsafe {
                // Safe because moving the last entry in the vec happens in the other branch
                std::ptr::swap_nonoverlapping(to_move, to_swap, self.type_info.layout.size());
            }
        }
        self.pop();
    }

    /// # Safety
    ///
    ///   The generic used must be the same as the type used for push_raw and must correspond to the data for the EcsId in TypeInfo
    #[allow(unused_unsafe)]
    pub unsafe fn as_slice<'a, T: 'static>(&'a self) -> &'a [T] {
        assert!(self.type_info.layout == core::alloc::Layout::new::<T>());
        assert!(self.len % self.type_info.layout.size() == 0);

        let slice_len = if self.type_info.layout.size() == 0 {
            self.len
        } else {
            self.len / self.type_info.layout.size()
        };

        unsafe {
            // Safe because we've really failed our job as an untyped vec if the data isnt aligned to T and size of T
            // The size of len * mem::size_of::<T>() cannot be > isize::MAX as we limit the capacity of our vec to less than isize::MAX
            std::slice::from_raw_parts(self.data.as_ptr() as *const T, slice_len)
        }
    }

    /// # Safety
    ///
    ///    The generic used must be the same as the type used for push_raw and must correspond to the data for the EcsId in TypeInfo
    #[allow(unused_unsafe)]
    pub unsafe fn as_slice_mut<'a, T: 'static>(&'a mut self) -> &'a mut [T] {
        assert!(self.type_info.layout == core::alloc::Layout::new::<T>());
        assert!(self.len % self.type_info.layout.size() == 0);

        let slice_len = if self.type_info.layout.size() == 0 {
            self.len
        } else {
            self.len / self.type_info.layout.size()
        };

        unsafe {
            // Safe because we've really failed our job as an untyped vec if the data isnt aligned to T and size of T
            // The size of len * mem::size_of::<T>() cannot be > isize::MAX as we limit the capacity of our vec to less than isize::MAX
            std::slice::from_raw_parts_mut(self.data.as_ptr() as *mut T, slice_len)
        }
    }

    /// # Safety
    ///
    /// Must not mutate through this ptr or use it after it has been invalidated
    pub unsafe fn as_immut_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// # Safety
    ///
    /// Must not use this ptr after it has been invalidated
    pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_ptr()
    }
}

impl Drop for UntypedVec {
    fn drop(&mut self) {
        if self.cap > 0 {
            while self.pop() {}

            let layout = Layout::from_size_align(self.cap, self.type_info.layout.align()).unwrap();
            let ptr = self.data.as_ptr();

            // Safe because we allocate this memory ourself so it must be from this allocator.
            // Safe because when we allocate memory we set cap to the size we used in the layout and
            // the align we use for the allocation is always self.type_info.layout.align()
            unsafe { dealloc(ptr, layout) }
        }
    }
}

#[cfg(test)]
mod untyped_vec {
    use super::*;
    use core::mem::ManuallyDrop;

    #[cfg(test)]
    pub fn untyped_vec_new<T: 'static>() -> UntypedVec {
        unsafe {
            UntypedVec::new_from_raw(TypeInfo::new(
                Layout::new::<T>(),
                Some(|ptr| core::ptr::drop_in_place::<T>(ptr as *mut T)),
            ))
        }
    }

    #[test]
    pub fn create() {
        //let untyped_vec = UntypedVec::new::<u32>();
        let untyped_vec = untyped_vec_new::<u32>();
        assert!(untyped_vec.cap == 0);
        assert!(untyped_vec.len == 0);
        assert!(untyped_vec.data == untyped_vec.type_info.dangling());
        assert!(untyped_vec.type_info.layout == Layout::new::<u32>());
    }

    #[test]
    pub fn grow() {
        let mut untyped_vec = untyped_vec_new::<u32>();

        untyped_vec.realloc();
        assert!(untyped_vec.cap == 16);
        assert!(untyped_vec.len == 0);
        assert!(untyped_vec.data != untyped_vec.type_info.dangling());
        assert!(untyped_vec.type_info.layout == Layout::new::<u32>());

        untyped_vec.realloc();
        assert!(untyped_vec.cap == 32);
        assert!(untyped_vec.len == 0);
        assert!(untyped_vec.data != untyped_vec.type_info.dangling());
        assert!(untyped_vec.type_info.layout == Layout::new::<u32>());
    }

    #[test]
    pub fn push_raw() {
        let mut untyped_vec = untyped_vec_new::<u32>();

        let data = 10_u32;
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
        }

        assert!(untyped_vec.len == 4);
        assert!(untyped_vec.cap == 16);
    }

    #[test]
    pub fn push_raw_realloc() {
        let mut untyped_vec = untyped_vec_new::<u32>();

        for n in 0..4 {
            let data = 10_u32;
            let mut data = ManuallyDrop::new(data);
            unsafe {
                untyped_vec.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
            }

            assert!(untyped_vec.len == (n + 1) * 4);
            assert!(untyped_vec.cap == 16);
        }

        let data = 10_u32;
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
        }

        assert!(untyped_vec.len == 20);
        assert!(untyped_vec.cap == 32);

        let slice = unsafe { untyped_vec.as_slice::<u32>() };
        assert!(slice.len() == 5);
        for item in slice {
            assert!(*item == 10);
        }
    }

    #[test]
    pub fn as_slice() {
        let mut untyped_vec = untyped_vec_new::<u32>();

        let data = 10_u32;
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
        }

        let slice = unsafe { untyped_vec.as_slice::<u32>() };
        assert!(slice.len() == 1);
        assert!(slice[0] == 10);
    }

    #[test]
    pub fn pop() {
        let mut untyped_vec = untyped_vec_new::<u32>();

        let data = 10_u32;
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
        }

        assert!(untyped_vec.pop());
        assert!(untyped_vec.len == 0);

        assert!(!untyped_vec.pop());
        assert!(untyped_vec.len == 0);
    }

    #[test]
    pub fn pop_drop_impl() {
        let mut dropped = false;
        pub struct Wrap(u32, *mut bool);
        impl Drop for Wrap {
            fn drop(&mut self) {
                unsafe { *self.1 = true };
            }
        }

        let mut untyped_vec = untyped_vec_new::<Wrap>();

        let data = Wrap(10, &mut dropped as *mut bool);
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
        }

        assert!(untyped_vec.pop());
        assert!(dropped);
        assert!(untyped_vec.len == 0);
    }

    #[test]
    pub fn drop_impl() {
        let mut dropped = false;
        pub struct Wrap(u32, *mut bool);
        impl Drop for Wrap {
            fn drop(&mut self) {
                unsafe { *self.1 = true };
            }
        }

        let mut untyped_vec = untyped_vec_new::<Wrap>();

        let data = Wrap(10, &mut dropped as *mut bool);
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
        }

        drop(untyped_vec);
        assert!(dropped);
    }

    #[test]
    pub fn move_element_to_other_vec() {
        let mut dropped = false;
        pub struct Wrap(u32, *mut bool);
        impl Drop for Wrap {
            fn drop(&mut self) {
                unsafe { *self.1 = true };
            }
        }

        let mut untyped_vec_1 = untyped_vec_new::<Wrap>();
        let data = Wrap(10, &mut dropped as *mut bool);
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec_1.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
        }

        let mut untyped_vec_2 = untyped_vec_new::<Wrap>();

        // TODO: proper workaround
        #[cfg(miri)]
        {
            untyped_vec_2.type_info.drop_fn = untyped_vec_1.type_info.drop_fn.clone();
        }

        unsafe {
            untyped_vec_1.swap_move_element_to_other_vec(&mut untyped_vec_2, 0);
        }

        assert!(dropped == false);
        assert!(untyped_vec_1.len == 0);
        assert!(untyped_vec_2.len == std::mem::size_of::<Wrap>());
        assert!(untyped_vec_2.cap == std::mem::size_of::<Wrap>() * 4);
        assert!(unsafe { untyped_vec_2.as_slice::<Wrap>()[0].0 } == 10);
    }

    #[test]
    pub fn move_element_to_other_vec_2() {
        let mut dropped_1 = false;
        let mut dropped_2 = false;
        pub struct Wrap(u32, *mut bool);
        impl Drop for Wrap {
            fn drop(&mut self) {
                unsafe { *self.1 = true };
            }
        }

        let mut untyped_vec_1 = untyped_vec_new::<Wrap>();
        let data = Wrap(10, &mut dropped_1 as *mut bool);
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec_1.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
        }

        let data = Wrap(12, &mut dropped_2 as *mut bool);
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec_1.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
        }

        let mut untyped_vec_2 = untyped_vec_new::<Wrap>();

        // TODO: proper workaround
        #[cfg(miri)]
        {
            untyped_vec_2.type_info.drop_fn = untyped_vec_1.type_info.drop_fn.clone();
        }

        unsafe {
            untyped_vec_1.swap_move_element_to_other_vec(&mut untyped_vec_2, 1);
        }

        assert!(dropped_1 == false);
        assert!(dropped_2 == false);
        assert!(untyped_vec_1.len == std::mem::size_of::<Wrap>());
        assert!(untyped_vec_1.cap == std::mem::size_of::<Wrap>() * 4);
        assert!(unsafe { untyped_vec_1.as_slice::<Wrap>()[0].0 } == 10);
        assert!(untyped_vec_2.len == std::mem::size_of::<Wrap>());
        assert!(untyped_vec_2.cap == std::mem::size_of::<Wrap>() * 4);
        assert!(unsafe { untyped_vec_2.as_slice::<Wrap>()[0].0 } == 12);
    }

    #[test]
    pub fn remove() {
        let mut dropped = false;
        pub struct Wrap(u32, *mut bool);
        impl Drop for Wrap {
            fn drop(&mut self) {
                unsafe { *self.1 = true };
            }
        }

        let mut untyped_vec = untyped_vec_new::<Wrap>();
        let data = Wrap(10, &mut dropped as *mut bool);
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec.push_raw(&mut data as *mut _ as *mut MaybeUninit<u8>);
        }

        untyped_vec.swap_remove(0);

        assert!(dropped == true);
        assert!(untyped_vec.len == 0);
    }
}
