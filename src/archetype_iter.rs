use super::entities::{EcsId, Entities};
use super::untyped_vec::UntypedVec;
use super::world::{Archetype, World};
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::slice::{Iter, IterMut};
use std::sync::RwLock;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

#[derive(Copy, Clone)]
pub struct Query<'a, T: QueryInfos + 'static> {
    world: &'a World,
    phantom: PhantomData<T>,
}

impl<'a, T: QueryInfos + 'static> Query<'a, T> {
    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            phantom: PhantomData,
        }
    }
}

pub struct QueryBorrow<
    'b,
    'guard: 'b,
    T: QueryInfos + 'static,
    U: StorageBorrows<'b, T>,
    Locks: 'guard,
> {
    storage_borrows: Vec<U>,
    _locks: Locks,
    phantom: PhantomData<&'b T>,
    phantom2: PhantomData<&'guard ()>,
}

pub trait QueryInfos: 'static {
    fn comp_ids(
        type_id_to_ecs_id: &HashMap<TypeId, EcsId, crate::utils::TypeIdHasherBuilder>,
    ) -> Vec<Option<EcsId>>;

    fn type_ids() -> Vec<Option<TypeId>>;
}

macro_rules! impl_query_infos {
    ($($x:ident)*) => {
        impl<'b, 'guard: 'b, $($x: Borrow<'b, 'guard>,)*> QueryInfos for ($($x,)*) {
            fn comp_ids(type_id_to_ecs_id: &HashMap<TypeId, EcsId, crate::utils::TypeIdHasherBuilder>) -> Vec<Option<EcsId>> {
                vec![$(
                    {
                        if let Some(id) = $x::type_id() {
                            Some(type_id_to_ecs_id[&id])
                        } else {
                            None
                        }
                    },
                )*]
            }

            fn type_ids() -> Vec<Option<TypeId>> {
                vec![
                    $(
                        $x::type_id(),
                    )*
                ]
            }
        }

        impl<'a, 'guard: 'a, $($x: Borrow<'a, 'guard>,)*> Query<'a, ($($x,)*)> {
            pub fn borrow<'this: 'guard>(&'this self) -> QueryBorrow<'a, 'guard, ($($x,)*), ($($x::StorageBorrow,)*), ($($x::Lock,)*)> {
                let archetypes = self.world.query_archetypes::<($($x,)*)>();
                let mut borrows: Vec<($($x::StorageBorrow,)*)> = Vec::with_capacity(16);

                let ecs_ids = [$($x::get_ecs_id(&self.world.type_id_to_ecs_id),)*];

                let locks = Self::acquire_locks(&self.world.lock_lookup, &self.world.locks, &ecs_ids);
                for archetype in archetypes.map(|idx| self.world.archetypes.get(idx).unwrap()) {
                    <Query<($($x,)*)>>::borrow_storages(&mut borrows, archetype, &ecs_ids);
                }

                QueryBorrow {
                    storage_borrows: borrows,
                    _locks: locks,
                    phantom: PhantomData,
                    phantom2: PhantomData,
                }
            }

            pub fn acquire_locks(lock_lookup: &HashMap<EcsId, usize, crate::utils::TypeIdHasherBuilder>, locks: &'guard [RwLock<()>], ecs_ids: &[Option<EcsId>]) -> ($($x::Lock,)*) {
                let mut n = 0;
                ($({
                    n += 1;
                    $x::acquire_lock(ecs_ids[n - 1], &lock_lookup, &locks)
                },)*)
            }

            pub fn borrow_storages(all_guards: &mut Vec<($($x::StorageBorrow,)*)>, archetype: &'guard Archetype, ecs_ids: &[Option<EcsId>]) {
                let mut n = 0;
                all_guards.push(
                    ($({
                        n += 1;
                        <$x as Borrow<'a, 'guard>>::borrow_storage(archetype, ecs_ids[n - 1])
                    },)*)
                );
            }
        }

        impl<'b, 'guard: 'b, $($x: Borrow<'b, 'guard>,)*> QueryBorrow<'b, 'guard, ($($x,)*), ($(<$x as Borrow<'b, 'guard>>::StorageBorrow,)*), ($($x::Lock,)*)> {
            pub fn for_each_mut<Func: FnMut(($($x::Returns,)*))>(&'b mut self, mut func: Func) {
                for guards in self.storage_borrows.iter_mut() {
                    let iter = <($(
                        $x::Iter,
                    )*) as Iters<($($x,)*)>>::iter_from_guards(guards);
                    let iter: ItersIterator<'b, 'guard, ($($x,)*), _> = ItersIterator::new(iter);

                    for item in iter {
                        func(item);
                    }
                }
            }

            pub fn for_each<Func: Fn(($($x::Returns,)*))>(&'b mut self, func: Func) {
                for guards in self.storage_borrows.iter_mut() {
                    let iter = <($(
                        $x::Iter,
                    )*) as Iters<($($x,)*)>>::iter_from_guards(guards);
                    let iter: ItersIterator<'b, 'guard, ($($x,)*), _> = ItersIterator::new(iter);

                    for item in iter {
                        func(item);
                    }
                }
            }
        }

        impl<'a, 'guard: 'a, $($x: Borrow<'a, 'guard>,)*> Iters<'a, 'guard, ($($x,)*)> for ($(<$x as Borrow<'a, 'guard>>::Iter,)*) {
            type Returns = ($($x::Returns,)*);
            type StorageBorrows = ($($x::StorageBorrow,)*);

            #[allow(non_snake_case)]
            fn iter_from_guards(locks: &'a mut Self::StorageBorrows) -> Self {
                let ($(
                    $x,
                )*) = locks;
                let mut length = None;

                ($(
                    {
                        let (len, iter) = $x::iter_from_guard($x);

                        // SAFETY, it's important that all iterators are the same length so that we can get_unchecked if the first iterator return Some()
                        if length.is_none() {
                            length = Some(len);
                        }
                        assert_eq!(length.unwrap(), len);

                        iter
                    },
                )*)
            }

            #[allow(non_snake_case)]
            #[inline(always)]
            fn next(&mut self) -> Option<Self::Returns> {
                if self.0.is_empty() {
                    return None;
                } else {
                    let ($($x,)*) = self;

                    // SAFETY: is_next_some returned true which means that out iterator is Some(_).
                    // Because the length of all the iterators in Iters must be the same this means all the other iterators must return Some(_) too.
                    // See Iters::iter_from_guards
                    return Some(($(
                        unsafe { <$x as Borrow<'a, 'guard>>::borrow_from_iter_unchecked($x) },
                    )*));
                }
            }

            fn new_empty() -> Self {
                ($(
                    <$x as Borrow<'a, 'guard>>::iter_empty(),
                )*)
            }
        }

        impl<'a, 'guard: 'a, $($x: Borrow<'a, 'guard>,)*> StorageBorrows<'a, ($($x,)*)> for ($($x::StorageBorrow,)*) {}
    };
}

/*impl_query_infos!(A B C D E F G H I J K L M N O P Q R S T U V W X Y Z);
impl_query_infos!(A B C D E F G H I J K L M N O P Q R S T U V W X Y);
impl_query_infos!(A B C D E F G H I J K L M N O P Q R S T U V W X);
impl_query_infos!(A B C D E F G H I J K L M N O P Q R S T U V W);
impl_query_infos!(A B C D E F G H I J K L M N O P Q R S T U V);
impl_query_infos!(A B C D E F G H I J K L M N O P Q R S T U);
impl_query_infos!(A B C D E F G H I J K L M N O P Q R S T);
impl_query_infos!(A B C D E F G H I J K L M N O P Q R S);
impl_query_infos!(A B C D E F G H I J K L M N O P Q R);
impl_query_infos!(A B C D E F G H I J K L M N O P Q);
impl_query_infos!(A B C D E F G H I J K L M N O P);
impl_query_infos!(A B C D E F G H I J K L M N O);
impl_query_infos!(A B C D E F G H I J K L M N);
impl_query_infos!(A B C D E F G H I J K L M);
impl_query_infos!(A B C D E F G H I J K L);
impl_query_infos!(A B C D E F G H I J K);*/
impl_query_infos!(A B C D E F G H I J);
impl_query_infos!(A B C D E F G H I);
impl_query_infos!(A B C D E F G H);
impl_query_infos!(A B C D E F G);
impl_query_infos!(A B C D E F);
impl_query_infos!(A B C D E);
impl_query_infos!(A B C D);
impl_query_infos!(A B C);
impl_query_infos!(A B);
impl_query_infos!(A);

pub struct ItersIterator<'a, 'guard: 'a, U: QueryInfos + 'static, T: Iters<'a, 'guard, U>> {
    iter: T,
    phantom: PhantomData<&'a U>,
    phantom2: PhantomData<&'guard T::StorageBorrows>,
}

impl<'a, 'guard: 'a, U: QueryInfos + 'static, T: Iters<'a, 'guard, U>> Iterator
    for ItersIterator<'a, 'guard, U, T>
{
    type Item = T::Returns;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, 'guard: 'a, U: QueryInfos + 'static, T: Iters<'a, 'guard, U>>
    ItersIterator<'a, 'guard, U, T>
{
    pub fn new(iter: T) -> Self {
        Self {
            iter,
            phantom: PhantomData,
            phantom2: PhantomData,
        }
    }
}

pub trait Iters<'a, 'guard: 'a, T: QueryInfos + 'static> {
    type Returns;
    type StorageBorrows: 'guard;

    fn iter_from_guards(locks: &'a mut Self::StorageBorrows) -> Self;
    fn next(&mut self) -> Option<Self::Returns>;
    fn new_empty() -> Self;
}

pub trait StorageBorrows<'guard, T: QueryInfos + 'static> {}

/// SAFETY: The length returned from iter_from_guards **must** be accurate as we rely on being able to call get_unchecked() if one iterator returns Some(_)
///
/// SAFETY: type_id function should return the same type_id used in the lookup inside get_ecs_id
///
/// SAFETY: <Borrow::Iter as ExactSizeIterator>::len must correctly give the amount of remaining elements
pub unsafe trait Borrow<'b, 'guard: 'b>: Sized + 'static {
    type Of: 'static;
    type Returns: 'b;
    type Iter: ExactSizeIterator + 'b;
    type StorageBorrow: 'guard;
    type Lock: 'guard;

    /// Used to find matching archetypes for the query
    fn get_ecs_id(
        type_id_to_ecs_id: &HashMap<TypeId, EcsId, crate::utils::TypeIdHasherBuilder>,
    ) -> Option<EcsId>;

    fn iter_from_guard(guard: &'b mut Self::StorageBorrow) -> (usize, Self::Iter);

    fn borrow_from_iter(iter: &mut Self::Iter) -> Option<Self::Returns>;

    unsafe fn borrow_from_iter_unchecked(iter: &mut Self::Iter) -> Self::Returns;

    fn borrow_storage(archetype: &'guard Archetype, comp_id: Option<EcsId>) -> Self::StorageBorrow;

    fn acquire_lock(
        comp_id: Option<EcsId>,
        lock_lookup: &HashMap<EcsId, usize, crate::utils::TypeIdHasherBuilder>,
        locks: &'guard [RwLock<()>],
    ) -> Self::Lock;

    fn iter_empty<'a>() -> Self::Iter;

    /// Used to create EcsId's needed for this query.
    fn type_id() -> Option<TypeId>;
}

unsafe impl<'b, 'guard: 'b, T: 'static> Borrow<'b, 'guard> for &'static T {
    type Of = T;
    type Returns = &'b T;
    type Iter = Iter<'b, T>;
    type StorageBorrow = &'guard UntypedVec;
    type Lock = RwLockReadGuard<'guard, ()>;

    fn iter_from_guard(borrow: &'b mut Self::StorageBorrow) -> (usize, Self::Iter) {
        // Safe because we lookup the UntypedVec via a EcsId gotten from the TypeId->EcsId hashmap
        let slice = unsafe { borrow.as_slice::<T>() };
        (slice.len(), slice.iter())
    }

    #[inline(always)]
    fn borrow_from_iter(iter: &mut Self::Iter) -> Option<Self::Returns> {
        iter.next()
    }

    #[inline(always)]
    #[allow(unused_unsafe)]
    unsafe fn borrow_from_iter_unchecked(iter: &mut Self::Iter) -> Self::Returns {
        match iter.next() {
            Some(item) => return item,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    fn get_ecs_id(
        type_id_to_ecs_id: &HashMap<TypeId, EcsId, crate::utils::TypeIdHasherBuilder>,
    ) -> Option<EcsId> {
        Some(type_id_to_ecs_id[&TypeId::of::<T>()])
    }

    fn borrow_storage(archetype: &'guard Archetype, comp_id: Option<EcsId>) -> Self::StorageBorrow {
        // TODO this has really bad performance when there's lots of components in an archetype
        // We really should use the .lookup[type_id] for those cases but that really badly affects
        // perf in cases where there *arent* many components in the archetype... EVENTUALLY we should
        // just cache the indices we need for a query rendering this all moot
        for (n, id) in archetype.comp_ids.iter().enumerate() {
            if id == &comp_id.unwrap() {
                return unsafe { &*archetype.component_storages[n].get() };
            }
        }
        panic!("Guard not found")
    }

    fn acquire_lock(
        comp_id: Option<EcsId>,
        lock_lookup: &HashMap<EcsId, usize, crate::utils::TypeIdHasherBuilder>,
        locks: &'guard [RwLock<()>],
    ) -> Self::Lock {
        let lock_idx = lock_lookup[&comp_id.unwrap()];
        locks.get(lock_idx).unwrap().read().unwrap()
    }

    fn iter_empty<'a>() -> Self::Iter {
        [].iter()
    }

    fn type_id() -> Option<TypeId> {
        Some(TypeId::of::<Self::Of>())
    }
}
unsafe impl<'b, 'guard: 'b, T: 'static> Borrow<'b, 'guard> for &'static mut T {
    type Of = T;
    type Returns = &'b mut T;
    type Iter = IterMut<'b, T>;
    type StorageBorrow = &'guard mut UntypedVec;
    type Lock = RwLockWriteGuard<'guard, ()>;

    fn iter_from_guard(borrow: &'b mut Self::StorageBorrow) -> (usize, Self::Iter) {
        // Safe because we lookup the UntypedVec via a EcsId gotten from the TypeId->EcsId hashmap
        let slice = unsafe { borrow.as_slice_mut::<T>() };
        (slice.len(), slice.iter_mut())
    }

    #[inline(always)]
    fn borrow_from_iter(iter: &mut Self::Iter) -> Option<Self::Returns> {
        iter.next()
    }

    #[inline(always)]
    #[allow(unused_unsafe)]
    unsafe fn borrow_from_iter_unchecked(iter: &mut Self::Iter) -> Self::Returns {
        match iter.next() {
            Some(item) => item,
            None => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    fn get_ecs_id(
        type_id_to_ecs_id: &HashMap<TypeId, EcsId, crate::utils::TypeIdHasherBuilder>,
    ) -> Option<EcsId> {
        Some(type_id_to_ecs_id[&TypeId::of::<T>()])
    }

    fn borrow_storage(archetype: &'guard Archetype, comp_id: Option<EcsId>) -> Self::StorageBorrow {
        // TODO this has really bad performance when there's lots of components in an archetype
        // We really should use the .lookup[type_id] for those cases but that really badly affects
        // perf in cases where there *arent* many components in the archetype... EVENTUALLY we should
        // just cache the indices we need for a query rendering this all moot
        for (n, id) in archetype.comp_ids.iter().enumerate() {
            if id == &comp_id.unwrap() {
                return unsafe { &mut *archetype.component_storages[n].get() };
            }
        }
        panic!("Guard not found")
    }

    fn acquire_lock(
        comp_id: Option<EcsId>,
        lock_lookup: &HashMap<EcsId, usize, crate::utils::TypeIdHasherBuilder>,
        locks: &'guard [RwLock<()>],
    ) -> Self::Lock {
        let lock_idx = lock_lookup[&comp_id.unwrap()];
        locks.get(lock_idx).unwrap().write().unwrap()
    }

    fn iter_empty<'a>() -> Self::Iter {
        [].iter_mut()
    }

    fn type_id() -> Option<TypeId> {
        Some(TypeId::of::<Self::Of>())
    }
}

unsafe impl<'b, 'guard: 'b> Borrow<'b, 'guard> for Entities {
    type Of = EcsId;
    type Returns = EcsId;
    type Iter = std::iter::Copied<Iter<'b, EcsId>>;
    type StorageBorrow = &'guard Vec<EcsId>;
    type Lock = ();

    fn iter_from_guard(guard: &'b mut Self::StorageBorrow) -> (usize, Self::Iter) {
        (guard.len(), guard.iter().copied())
    }

    #[inline(always)]
    fn borrow_from_iter(iter: &mut Self::Iter) -> Option<Self::Returns> {
        iter.next()
    }

    #[inline(always)]
    #[allow(unused_unsafe)]
    unsafe fn borrow_from_iter_unchecked(iter: &mut Self::Iter) -> Self::Returns {
        match iter.next() {
            Some(item) => item,
            None => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    fn get_ecs_id(_: &HashMap<TypeId, EcsId, crate::utils::TypeIdHasherBuilder>) -> Option<EcsId> {
        None
    }

    fn borrow_storage(archetype: &'guard Archetype, comp_id: Option<EcsId>) -> Self::StorageBorrow {
        assert!(comp_id.is_none());
        &archetype.entities
    }

    fn acquire_lock(
        _: Option<EcsId>,
        _: &HashMap<EcsId, usize, crate::utils::TypeIdHasherBuilder>,
        _: &'guard [RwLock<()>],
    ) -> Self::Lock {
        ()
    }

    fn iter_empty<'a>() -> Self::Iter {
        [].iter().copied()
    }

    fn type_id() -> Option<TypeId> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spawn;

    #[test]
    fn for_each_mut() {
        let mut world = World::new();

        spawn!(&mut world, 10_u32, 12_u64);
        spawn!(&mut world, 15_u32, 14_u64);
        spawn!(&mut world, 20_u32, 16_u64);

        let query = world.query::<(&mut u32, &u64)>();
        let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
        query.borrow().for_each_mut(|(left, right)| {
            assert_eq!(checks.next().unwrap(), (*left, *right));
        });
        assert_eq!(checks.next(), None);
    }

    #[test]
    fn for_each_iterator() {
        let mut world = World::new();

        spawn!(&mut world, 10_u32, 12_u64);
        spawn!(&mut world, 15_u32, 14_u64);
        spawn!(&mut world, 20_u32, 16_u64);

        let query = world.query::<(&mut u32, &u64)>();

        let mut checks = vec![(10, 12), (15, 14), (20, 16)].into_iter();
        query
            .borrow()
            .for_each_mut(|(left, right)| assert_eq!((*left, *right), checks.next().unwrap()));
        assert!(checks.next().is_none());
    }

    #[test]
    fn for_each_subset_iterator() {
        let mut world = World::new();

        spawn!(&mut world, 10_u32, 12_u64);
        spawn!(&mut world, 15_u32, 14_u64);
        spawn!(&mut world, 20_u32, 16_u64);

        let query = world.query::<(&mut u32,)>();

        let mut checks = vec![10, 15, 20].into_iter();
        query
            .borrow()
            .for_each_mut(|(left,)| assert_eq!(*left, checks.next().unwrap()));
        assert!(checks.next().is_none());
    }

    #[test]
    fn for_each_multi_archetype_iterator() {
        let mut world = World::new();

        spawn!(&mut world, 10_u32, 12_u64);
        spawn!(&mut world, 15_u32, 14_u64);
        spawn!(&mut world, 20_u32, 16_u64);

        spawn!(&mut world, 11_u32, 12_u64, 99_u128);
        spawn!(&mut world, 16_u32, 14_u64, 99_u128);
        spawn!(&mut world, 21_u32, 16_u64, 99_u128);

        let query = world.query::<(&mut u32,)>();

        let mut checks = vec![10, 15, 20, 11, 16, 21].into_iter();
        query
            .borrow()
            .for_each_mut(|(left,)| assert_eq!(*left, checks.next().unwrap()));
        assert!(checks.next().is_none());
    }

    #[test]
    fn query_param_in_func() {
        let mut world = World::new();
        spawn!(&mut world, 10_u32, 12_u64);
        let query = world.query::<(&u32, &u64)>();

        fn func(query: Query<(&u32, &u64)>) {
            let mut ran = false;
            query.borrow().for_each_mut(|(left, right)| {
                assert!(*left == 10);
                assert!(*right == 12);
                ran = true;
            });
            assert!(ran);
        }

        func(query);
    }

    #[test]
    fn entity_query() {
        let mut world = World::new();

        spawn!(&mut world, 1_u32, 12_u64);

        let query = world.query::<(Entities, &u32, &u64)>();

        let mut checks = vec![(EcsId::new(0, 0), 1, 12)].into_iter();
        query.borrow().for_each_mut(|(entity, left, right)| {
            assert!(checks.next().unwrap() == (entity, *left, *right));
        });
        assert!(checks.next().is_none());
    }
}
