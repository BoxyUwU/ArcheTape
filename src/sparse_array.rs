pub struct SparseArray<T, const PAGE_SIZE: usize> {
    sparse: Vec<Option<Box<[Option<T>; PAGE_SIZE]>>>,
}

impl<T, const PAGE_SIZE: usize> SparseArray<T, PAGE_SIZE> {
    pub fn new() -> Self {
        Self { sparse: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut sparse = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            sparse.push(Some(Box::new([None; PAGE_SIZE])));
        }
        Self { sparse }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        match &self.sparse[index / PAGE_SIZE] {
            Some(page) => page[index % PAGE_SIZE].as_ref(),
            None => return None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match &mut self.sparse[index / PAGE_SIZE] {
            Some(page) => page[index % PAGE_SIZE].as_mut(),
            None => return None,
        }
    }

    pub fn insert(&mut self, index: usize, data: T) -> Option<T> {
        if self.sparse.len() <= index / PAGE_SIZE {
            self.sparse.resize_with(index / PAGE_SIZE + 1, || None);
        }

        let page = self.sparse.get_mut(index / PAGE_SIZE).unwrap();
        if page.is_none() {
            *page = Some(Box::new([None; PAGE_SIZE]));
        }

        let page = page.as_mut().unwrap();
        let entry = page.get_mut(index % PAGE_SIZE).unwrap();

        std::mem::replace(entry, Some(data))
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        let page = self.sparse.get_mut(index / PAGE_SIZE)?.as_mut()?;
        let entry = &mut page[index % PAGE_SIZE];
        std::mem::replace(entry, None)
    }
}
