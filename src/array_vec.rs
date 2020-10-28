use std::mem::MaybeUninit;

pub struct ArrayVec<T, const LEN: usize> {
    array: [MaybeUninit<T>; LEN],
    len: usize,
}

impl<T, const LEN: usize> ArrayVec<T, LEN> {
    pub fn new() -> Self {
        assert!(LEN < isize::MAX as usize);

        Self {
            array: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn push(&mut self, data: T) -> Result<(), ()> {
        if self.len < LEN {
            self.array[self.len] = MaybeUninit::new(data);
            self.len += 1;
            return Ok(());
        }

        Err(())
    }

    pub fn push_start(&mut self, data: T) {
        if self.len == LEN {
            let last = self.as_slice_mut().last_mut().unwrap() as *mut T;
            unsafe {
                core::ptr::drop_in_place(last);
            };
            self.len -= 1;
        }

        let src = self.array.as_mut_ptr();
        let dst = unsafe { src.offset(1) };
        unsafe {
            core::ptr::copy(src, dst, self.len);
        }

        let uninit_data = &mut self.array[0];
        *uninit_data = MaybeUninit::new(data);

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            let data = unsafe { self.array[self.len].as_mut_ptr().read() };
            self.len -= 1;
            return Some(data);
        }

        None
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            let data = unsafe { &*self.array[index].as_ptr() };
            return Some(data);
        }

        None
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            let data = unsafe { &mut *self.array[index].as_mut_ptr() };
            return Some(data);
        }

        None
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &[T] {
        let slice = &self.array[0..self.len];
        let slice_ptr = slice as *const [MaybeUninit<T>] as *const [T];
        unsafe { &*slice_ptr }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let slice = &mut self.array[0..self.len];
        let slice_ptr = slice as *mut [MaybeUninit<T>] as *mut [T];
        unsafe { &mut *slice_ptr }
    }
}

impl<T, const LEN: usize> Drop for ArrayVec<T, LEN> {
    fn drop(&mut self) {
        let slice = &mut self.array[0..self.len];
        for data in slice {
            unsafe {
                core::ptr::drop_in_place(data.as_mut_ptr());
            }
        }
    }
}
