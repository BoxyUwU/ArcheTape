use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    any::TypeId,
    mem::ManuallyDrop,
    mem::MaybeUninit,
    ptr::NonNull,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TypeInfo {
    pub id: TypeId,
    pub layout: Layout,
}

impl TypeInfo {
    pub fn new<T: 'static>() -> Self {
        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
        }
    }

    pub fn new_from_raw(id: TypeId, layout: Layout) -> Self {
        Self { id, layout }
    }
}

pub struct UntypedVec {
    type_info: TypeInfo,
    data: NonNull<u8>,
    cap: usize, // In bytes
    len: usize, // In bytes
    drop_fn: Option<fn(*mut MaybeUninit<u8>)>,
}

impl UntypedVec {
    pub fn new<T: 'static>() -> Self {
        let type_info = TypeInfo::new::<T>();

        // Safety: it's safe to call drop_in_place here because we know that this function will only be called with pointers to T that are aligned and nonnull
        // Safety: drop_in_place says to "uphold any safety invariants of T that are related to dropping" but we just dont do this :)
        let drop_fn: fn(*mut MaybeUninit<u8>) -> () = |ptr| unsafe {
            std::ptr::drop_in_place::<T>(ptr as *mut T);
        };

        // Safety: Because of generics we know that type_info is correct,
        unsafe { Self::new_from_raw(type_info, Some(drop_fn)) }
    }

    pub fn new_from_untyped_vec(from: &mut UntypedVec) -> Self {
        // Safe because the passed in untyped vec was either made safely or with unsafe code
        unsafe { Self::new_from_raw(from.type_info, from.drop_fn) }
    }

    /// Safety: drop_fn must take a pointer to a MaybeUninit<u8> and call the `Drop` impl of the type that TypeInfo corresponds to.
    /// If your type doesnt have a Drop trait implementaton then this can just be None.
    pub unsafe fn new_from_raw(
        type_info: TypeInfo,
        drop_fn: Option<fn(*mut MaybeUninit<u8>)>,
    ) -> Self {
        Self {
            type_info,
            data: NonNull::dangling(),
            cap: 0,
            len: 0,
            drop_fn,
        }
    }

    pub fn get_type_info(&self) -> TypeInfo {
        self.type_info
    }

    pub fn len(&self) -> usize {
        if self.type_info.layout.size() == 0 {
            return self.len;
        }

        assert!(self.len % self.type_info.layout.size() == 0);
        self.len / self.type_info.layout.size()
    }

    pub fn raw_len(&self) -> usize {
        self.len
    }

    pub fn realloc(&mut self) {
        if self.type_info.layout.size() == 0 {
            panic!("Attempted to reallocate an UntypedVec who's data is size 0");
        }

        if self.cap == 0 {
            let new_cap = self.type_info.layout.size() * 4;
            let layout = Layout::from_size_align(new_cap, self.type_info.layout.align()).unwrap();

            // Safe because we assert size is not 0
            let ptr: *mut u8 = unsafe { alloc(layout) };
            let ptr = NonNull::new(ptr).unwrap();

            self.cap = new_cap;
            self.data = ptr;
        } else {
            let new_cap = self.cap * 2;
            assert!(new_cap < isize::MAX as usize);
            let old_layout =
                Layout::from_size_align(self.cap, self.type_info.layout.align()).unwrap();

            // Safe because we assert size is not 0
            // Safe because the pointer we pass in is always made from this allocator because
            // the only way to get a cap > 0 is if the other branch has run and allocated memory
            // Safe because new_cap is < isize::MAX
            let ptr = self.data.as_ptr();
            let ptr: *mut u8 = unsafe { realloc(ptr, old_layout, new_cap) };
            let ptr = NonNull::new(ptr).unwrap();

            self.cap = new_cap;
            self.data = ptr;
        }
    }

    /// The data at src must not be used after calling this function
    ///
    /// The data at src should be NonNull, aligned to type_info.layout.align() and should be the size given by type_info.layout.size()
    ///
    /// The data must be a valid instance of the type that type_info.id represents
    ///
    /// Type_info passed in must be the same as the type_info used to create the UntypedVec
    #[allow(unused_unsafe)]
    pub unsafe fn push_raw(&mut self, src: *mut MaybeUninit<u8>, type_info: TypeInfo) {
        debug_assert!(type_info == self.type_info);
        debug_assert!(src.is_null() == false);

        if self.type_info.layout.size() == 0 {
            self.len += 1;
            return;
        }

        // A realloc is guaranteed to make enough room to push to data because the initial allocation is of
        // type_info.layout.size() * 4, which means that a realloc will always allocate more than the required bytes
        if self.len + type_info.layout.size() > self.cap {
            self.realloc();
        }

        // Safe because we are offsetting within allocated memory and cap is < isize::MAX
        let dst: *mut u8 = unsafe { self.data.as_ptr().offset(self.len as isize) };
        let dst = dst as *mut MaybeUninit<u8>;

        unsafe {
            // The pointers are guaranteed to be nonoverlapping as we are writing to uninitialised memory in the vec
            std::ptr::copy_nonoverlapping(src, dst, self.type_info.layout.size());
        }

        self.len += self.type_info.layout.size();
    }

    pub fn push<T: 'static>(&mut self, data: T) {
        let type_info = TypeInfo::new::<T>();
        assert!(type_info == self.type_info);

        let mut data = ManuallyDrop::new(data);
        let ptr = &mut data as *mut ManuallyDrop<T> as *mut MaybeUninit<u8>;
        unsafe {
            // Safe because we assert that the UntypedVec's TypeInfo is the same as T's TypeInfo
            // Safe because T must be aligned and a valid instance because that's how rust works
            // Safe because we put data in a ManuallyDrop the data behind ptr
            self.push_raw(ptr, type_info);
        }
    }

    /// Returns true if a value was popped
    pub fn pop(&mut self) -> bool {
        if self.type_info.layout.size() == 0 && self.len > 0 {
            self.len -= 1;
            let ptr = self.data.as_ptr();
            let ptr = ptr as *mut MaybeUninit<u8>;

            if let Some(drop_fn) = self.drop_fn {
                drop_fn(ptr);
            }
            true
        } else if self.len >= self.type_info.layout.size() {
            self.len -= self.type_info.layout.size();
            let ptr = self.data.as_ptr();
            // Safe because we're offsetting inside of the allocation
            let ptr: *mut u8 = unsafe { ptr.offset(self.len as isize) };
            let ptr = ptr as *mut MaybeUninit<u8>;

            if let Some(drop_fn) = self.drop_fn {
                drop_fn(ptr);
            }
            true
        } else {
            false
        }
    }

    pub fn swap_move_element_to_other_vec(&mut self, other: &mut UntypedVec, element: usize) {
        assert!(self.type_info == other.type_info);
        assert!(self.len > 0);
        assert!(
            self.type_info.layout.size() == 0 || element < self.len / self.type_info.layout.size()
        );

        let data: *mut MaybeUninit<u8> = self.data.as_ptr() as *mut MaybeUninit<u8>;

        if self.type_info.layout.size() == 0 {
            self.len -= 1;
            other.len += 1;
        } else if element == self.len / self.type_info.layout.size() - 1 {
            // Safe because we're offsetting inside the allocation and len is never >= isize::MAX
            let to_move = unsafe { data.offset((element * self.type_info.layout.size()) as isize) };

            unsafe {
                // Safe because we assert that the type_info for self and other are the same.
                // Safe because we reduce the length of this vec by one which is effectively mem::forget
                other.push_raw(to_move, self.type_info);
            }

            self.len -= self.type_info.layout.size();
        } else {
            // Safe because we're offsetting inside the allocation and len is never >= isize::MAX
            let to_move = unsafe { data.offset((element * self.type_info.layout.size()) as isize) };
            let to_swap = unsafe {
                data.offset(self.len as isize)
                    .offset(-(self.type_info.layout.size() as isize))
            };

            unsafe {
                // Safe because moving the last entry in the vec happens in the other branch
                std::ptr::swap_nonoverlapping(to_move, to_swap, self.type_info.layout.size());
            }

            unsafe {
                // Safe because we assert that the type_info for self and other are the same.
                // Safe because we assert that byte_index is aligned to self.type_info.layout.align()
                // Safe because we reduce the length of this vec by one which means we wont touch the data again
                other.push_raw(to_swap, self.type_info);
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
            self.pop();
        } else {
            // Safe because we're offsetting inside the allocation and len is never >= isize::MAX
            let to_move = unsafe { data.offset((element * self.type_info.layout.size()) as isize) };
            let to_swap = unsafe {
                data.offset(self.len as isize)
                    .offset(-(self.type_info.layout.size() as isize))
            };

            unsafe {
                // Safe because moving the last entry in the vec happens in the other branch
                std::ptr::swap_nonoverlapping(to_move, to_swap, self.type_info.layout.size());
            }

            self.pop();
        }
    }

    pub fn as_slice<'a, T: 'static>(&'a self) -> &'a [T] {
        assert!(TypeInfo::new::<T>() == self.type_info);
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

    pub fn as_slice_mut<'a, T: 'static>(&'a mut self) -> &'a mut [T] {
        assert!(TypeInfo::new::<T>() == self.type_info);
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
mod tests {
    use super::*;
    #[test]
    pub fn create() {
        let untyped_vec = UntypedVec::new::<u32>();
        assert!(untyped_vec.cap == 0);
        assert!(untyped_vec.len == 0);
        assert!(untyped_vec.data == NonNull::dangling());
        assert!(untyped_vec.type_info.id == TypeId::of::<u32>());
        assert!(untyped_vec.type_info.layout == Layout::new::<u32>());
    }

    #[test]
    pub fn grow() {
        let mut untyped_vec = UntypedVec::new::<u32>();

        untyped_vec.realloc();
        assert!(untyped_vec.cap == 16);
        assert!(untyped_vec.len == 0);
        assert!(untyped_vec.data != NonNull::dangling());
        assert!(untyped_vec.type_info.id == TypeId::of::<u32>());
        assert!(untyped_vec.type_info.layout == Layout::new::<u32>());

        untyped_vec.realloc();
        assert!(untyped_vec.cap == 32);
        assert!(untyped_vec.len == 0);
        assert!(untyped_vec.data != NonNull::dangling());
        assert!(untyped_vec.type_info.id == TypeId::of::<u32>());
        assert!(untyped_vec.type_info.layout == Layout::new::<u32>());
    }

    #[test]
    pub fn push_raw() {
        let mut untyped_vec = UntypedVec::new::<u32>();

        let data = 10_u32;
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec.push_raw(
                &mut data as *mut ManuallyDrop<u32> as *mut MaybeUninit<u8>,
                TypeInfo::new::<u32>(),
            );
        }

        assert!(untyped_vec.len == 4);
        assert!(untyped_vec.cap == 16);
    }

    #[test]
    pub fn push_raw_realloc() {
        let mut untyped_vec = UntypedVec::new::<u32>();

        for n in 0..4 {
            let data = 10_u32;
            let mut data = ManuallyDrop::new(data);
            unsafe {
                untyped_vec.push_raw(
                    &mut data as *mut ManuallyDrop<u32> as *mut MaybeUninit<u8>,
                    TypeInfo::new::<u32>(),
                );
            }

            assert!(untyped_vec.len == (n + 1) * 4);
            assert!(untyped_vec.cap == 16);
        }

        let data = 10_u32;
        let mut data = ManuallyDrop::new(data);
        unsafe {
            untyped_vec.push_raw(
                &mut data as *mut ManuallyDrop<u32> as *mut MaybeUninit<u8>,
                TypeInfo::new::<u32>(),
            );
        }

        assert!(untyped_vec.len == 20);
        assert!(untyped_vec.cap == 32);

        let slice = untyped_vec.as_slice::<u32>();
        assert!(slice.len() == 5);
        for item in slice {
            assert!(*item == 10);
        }
    }

    #[test]
    pub fn as_slice() {
        let mut untyped_vec = UntypedVec::new::<u32>();

        let data = 10_u32;
        untyped_vec.push(data);

        let slice = untyped_vec.as_slice::<u32>();
        assert!(slice.len() == 1);
        assert!(slice[0] == 10);
    }

    #[test]
    pub fn pop() {
        let mut untyped_vec = UntypedVec::new::<u32>();

        let data = 10_u32;
        untyped_vec.push(data);

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

        let mut untyped_vec = UntypedVec::new::<Wrap>();

        let data = Wrap(10, &mut dropped as *mut bool);
        untyped_vec.push(data);

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

        let mut untyped_vec = UntypedVec::new::<Wrap>();

        let data = Wrap(10, &mut dropped as *mut bool);
        untyped_vec.push(data);

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

        let mut untyped_vec_1 = UntypedVec::new::<Wrap>();
        let data = Wrap(10, &mut dropped as *mut bool);
        untyped_vec_1.push(data);

        let mut untyped_vec_2 = UntypedVec::new::<Wrap>();

        untyped_vec_1.swap_move_element_to_other_vec(&mut untyped_vec_2, 0);

        assert!(dropped == false);
        assert!(untyped_vec_1.len == 0);
        assert!(untyped_vec_2.len == std::mem::size_of::<Wrap>());
        assert!(untyped_vec_2.cap == std::mem::size_of::<Wrap>() * 4);
        assert!(untyped_vec_2.as_slice::<Wrap>()[0].0 == 10);
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

        let mut untyped_vec_1 = UntypedVec::new::<Wrap>();
        let data = Wrap(10, &mut dropped_1 as *mut bool);
        untyped_vec_1.push(data);

        let data = Wrap(12, &mut dropped_2 as *mut bool);
        untyped_vec_1.push(data);

        let mut untyped_vec_2 = UntypedVec::new::<Wrap>();

        untyped_vec_1.swap_move_element_to_other_vec(&mut untyped_vec_2, 1);

        assert!(dropped_1 == false);
        assert!(dropped_2 == false);
        assert!(untyped_vec_1.len == std::mem::size_of::<Wrap>());
        assert!(untyped_vec_1.cap == std::mem::size_of::<Wrap>() * 4);
        assert!(untyped_vec_1.as_slice::<Wrap>()[0].0 == 10);
        assert!(untyped_vec_2.len == std::mem::size_of::<Wrap>());
        assert!(untyped_vec_2.cap == std::mem::size_of::<Wrap>() * 4);
        assert!(untyped_vec_2.as_slice::<Wrap>()[0].0 == 12);
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

        let mut untyped_vec = UntypedVec::new::<Wrap>();
        let data = Wrap(10, &mut dropped as *mut bool);
        untyped_vec.push(data);

        untyped_vec.swap_remove(0);

        assert!(dropped == true);
        assert!(untyped_vec.len == 0);
    }
}
