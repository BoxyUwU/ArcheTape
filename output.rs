#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
pub mod anymap {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::convert::TryInto;
    use std::error::Error;
    use std::hash::{BuildHasher, Hasher};
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};
    use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
    pub struct TypeIdHasher(u64);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::default::Default for TypeIdHasher {
        #[inline]
        fn default() -> TypeIdHasher {
            TypeIdHasher(::core::default::Default::default())
        }
    }
    impl Hasher for TypeIdHasher {
        fn write(&mut self, bytes: &[u8]) {
            self.0 = u64::from_ne_bytes(bytes.try_into().unwrap());
        }
        fn finish(&self) -> u64 {
            self.0
        }
    }
    pub struct TypeIdHasherBuilder();
    impl BuildHasher for TypeIdHasherBuilder {
        type Hasher = TypeIdHasher;
        fn build_hasher(&self) -> Self::Hasher {
            TypeIdHasher::default()
        }
    }
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
        map: HashMap<TypeId, RwLock<Box<dyn Any + 'static>>, TypeIdHasherBuilder>,
    }
    impl<'a> AnyMap {
        pub fn new() -> Self {
            Self {
                map: HashMap::with_hasher(TypeIdHasherBuilder()),
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
        pub fn get_mut_with_self<'this, T: 'static>(
            &'this mut self,
        ) -> Result<&mut Box<dyn Any>, Box<dyn Error + 'this>> {
            let type_id = TypeId::of::<T>();
            let rw_lock = self.map.get_mut(&type_id).unwrap();
            Ok(rw_lock.get_mut()?)
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
}
pub mod archetype_iter {
    use super::world::{Archetype, World};
    use std::any::{Any, TypeId};
    use std::iter::Peekable;
    use std::marker::PhantomData;
    use std::slice::{Iter, IterMut};
    use std::sync::{RwLockReadGuard, RwLockWriteGuard};
    pub enum RwLockEitherGuard<'a> {
        WriteGuard(RwLockWriteGuard<'a, Box<dyn Any>>),
        ReadGuard(RwLockReadGuard<'a, Box<dyn Any>>),
    }
    pub struct Query<'a, T: QueryInfos + 'a> {
        world: &'a World,
        phantom: PhantomData<T>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<'a, T: ::core::marker::Copy + QueryInfos + 'a> ::core::marker::Copy for Query<'a, T> {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<'a, T: ::core::clone::Clone + QueryInfos + 'a> ::core::clone::Clone for Query<'a, T> {
        #[inline]
        fn clone(&self) -> Query<'a, T> {
            match *self {
                Query {
                    world: ref __self_0_0,
                    phantom: ref __self_0_1,
                } => Query {
                    world: ::core::clone::Clone::clone(&(*__self_0_0)),
                    phantom: ::core::clone::Clone::clone(&(*__self_0_1)),
                },
            }
        }
    }
    impl<'a, T: QueryInfos + 'a> Query<'a, T> {
        pub fn new(world: &'a World) -> Self {
            Self {
                world,
                phantom: PhantomData,
            }
        }
        pub fn borrow(&self) -> QueryBorrow<'_, '_, T> {
            let archetypes = self.world.query_archetypes::<T>();
            let mut guards = Vec::with_capacity(16);
            for archetype in archetypes.map(|idx| self.world.archetypes.get(idx).unwrap()) {
                guards.extend(T::borrow_guards(archetype));
            }
            QueryBorrow {
                lock_guards: guards,
                phantom: PhantomData,
                phantom2: PhantomData,
            }
        }
    }
    pub struct QueryBorrow<'b, 'guard, T: QueryInfos + 'b> {
        lock_guards: Vec<RwLockEitherGuard<'guard>>,
        phantom: PhantomData<T>,
        phantom2: PhantomData<&'b ()>,
    }
    pub trait QueryInfos {
        fn arity() -> usize;
        fn type_ids() -> Vec<TypeId>;
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>>;
    }
    impl<
            'b,
            A: Borrow<'b>,
            B: Borrow<'b>,
            C: Borrow<'b>,
            D: Borrow<'b>,
            E: Borrow<'b>,
            F: Borrow<'b>,
            G: Borrow<'b>,
            H: Borrow<'b>,
            I: Borrow<'b>,
            J: Borrow<'b>,
        > QueryInfos for (A, B, C, D, E, F, G, H, I, J)
    {
        #[allow(unused, non_snake_case)]
        fn arity() -> usize {
            let mut count = 0;
            let A = ();
            count += 1;
            let B = ();
            count += 1;
            let C = ();
            count += 1;
            let D = ();
            count += 1;
            let E = ();
            count += 1;
            let F = ();
            count += 1;
            let G = ();
            count += 1;
            let H = ();
            count += 1;
            let I = ();
            count += 1;
            let J = ();
            count += 1;
            count
        }
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A::Of>(),
                TypeId::of::<B::Of>(),
                TypeId::of::<C::Of>(),
                TypeId::of::<D::Of>(),
                TypeId::of::<E::Of>(),
                TypeId::of::<F::Of>(),
                TypeId::of::<G::Of>(),
                TypeId::of::<H::Of>(),
                TypeId::of::<I::Of>(),
                TypeId::of::<J::Of>(),
            ])
        }
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
            <[_]>::into_vec(box [
                A::guards_from_archetype(archetype),
                B::guards_from_archetype(archetype),
                C::guards_from_archetype(archetype),
                D::guards_from_archetype(archetype),
                E::guards_from_archetype(archetype),
                F::guards_from_archetype(archetype),
                G::guards_from_archetype(archetype),
                H::guards_from_archetype(archetype),
                I::guards_from_archetype(archetype),
                J::guards_from_archetype(archetype),
            ])
        }
    }
    impl<
            'g: 'b,
            'b,
            A: Borrow<'b>,
            B: Borrow<'b>,
            C: Borrow<'b>,
            D: Borrow<'b>,
            E: Borrow<'b>,
            F: Borrow<'b>,
            G: Borrow<'b>,
            H: Borrow<'b>,
            I: Borrow<'b>,
            J: Borrow<'b>,
        > QueryBorrow<'b, 'g, (A, B, C, D, E, F, G, H, I, J)>
    {
        pub fn into_for_each_mut<Func: FnMut((A, B, C, D, E, F, G, H, I, J))>(
            &'b mut self,
            mut func: Func,
        ) {
            let arity = <(A, B, C, D, E, F, G, H, I, J) as QueryInfos>::arity();
            if true {
                if !(self.lock_guards.len() % arity == 0) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: self.lock_guards.len() % arity == 0",
                        )
                    }
                };
            };
            let mut iterators = Vec::with_capacity(self.lock_guards.len());
            for chunk in self.lock_guards.chunks_mut(arity) {
                let iter = <(
                    A::Iter,
                    B::Iter,
                    C::Iter,
                    D::Iter,
                    E::Iter,
                    F::Iter,
                    G::Iter,
                    H::Iter,
                    I::Iter,
                    J::Iter,
                ) as Iters<(A, B, C, D, E, F, G, H, I, J)>>::iter_from_guards(
                    chunk
                );
                let iter: ItersIterator<'_, (A, B, C, D, E, F, G, H, I, J), _> =
                    ItersIterator::new(iter);
                iterators.push(iter);
            }
            for iter in iterators {
                for item in iter {
                    func(item);
                }
            }
        }
    }
    impl<
            'a,
            A: Borrow<'a>,
            B: Borrow<'a>,
            C: Borrow<'a>,
            D: Borrow<'a>,
            E: Borrow<'a>,
            F: Borrow<'a>,
            G: Borrow<'a>,
            H: Borrow<'a>,
            I: Borrow<'a>,
            J: Borrow<'a>,
        > Iters<'a, (A, B, C, D, E, F, G, H, I, J)>
        for (
            A::Iter,
            B::Iter,
            C::Iter,
            D::Iter,
            E::Iter,
            F::Iter,
            G::Iter,
            H::Iter,
            I::Iter,
            J::Iter,
        )
    {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
            let mut iter = locks.iter_mut();
            let mut length = None;
            (
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <A as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <B as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <C as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <D as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <E as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <F as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <G as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <H as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <I as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <J as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
            )
        }
        #[allow(non_snake_case)]
        #[inline(always)]
        fn next(&mut self) -> Option<(A, B, C, D, E, F, G, H, I, J)> {
            if !self.0.is_next_some() {
                return None;
            }
            let (A, B, C, D, E, F, G, H, I, J) = self;
            Some((
                unsafe { <A as Borrow<'a>>::borrow_from_iter_unchecked(A) },
                unsafe { <B as Borrow<'a>>::borrow_from_iter_unchecked(B) },
                unsafe { <C as Borrow<'a>>::borrow_from_iter_unchecked(C) },
                unsafe { <D as Borrow<'a>>::borrow_from_iter_unchecked(D) },
                unsafe { <E as Borrow<'a>>::borrow_from_iter_unchecked(E) },
                unsafe { <F as Borrow<'a>>::borrow_from_iter_unchecked(F) },
                unsafe { <G as Borrow<'a>>::borrow_from_iter_unchecked(G) },
                unsafe { <H as Borrow<'a>>::borrow_from_iter_unchecked(H) },
                unsafe { <I as Borrow<'a>>::borrow_from_iter_unchecked(I) },
                unsafe { <J as Borrow<'a>>::borrow_from_iter_unchecked(J) },
            ))
        }
        fn new_empty() -> Self {
            (
                <A as Borrow<'a>>::iter_empty(),
                <B as Borrow<'a>>::iter_empty(),
                <C as Borrow<'a>>::iter_empty(),
                <D as Borrow<'a>>::iter_empty(),
                <E as Borrow<'a>>::iter_empty(),
                <F as Borrow<'a>>::iter_empty(),
                <G as Borrow<'a>>::iter_empty(),
                <H as Borrow<'a>>::iter_empty(),
                <I as Borrow<'a>>::iter_empty(),
                <J as Borrow<'a>>::iter_empty(),
            )
        }
    }
    impl<
            'b,
            A: Borrow<'b>,
            B: Borrow<'b>,
            C: Borrow<'b>,
            D: Borrow<'b>,
            E: Borrow<'b>,
            F: Borrow<'b>,
            G: Borrow<'b>,
            H: Borrow<'b>,
            I: Borrow<'b>,
        > QueryInfos for (A, B, C, D, E, F, G, H, I)
    {
        #[allow(unused, non_snake_case)]
        fn arity() -> usize {
            let mut count = 0;
            let A = ();
            count += 1;
            let B = ();
            count += 1;
            let C = ();
            count += 1;
            let D = ();
            count += 1;
            let E = ();
            count += 1;
            let F = ();
            count += 1;
            let G = ();
            count += 1;
            let H = ();
            count += 1;
            let I = ();
            count += 1;
            count
        }
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A::Of>(),
                TypeId::of::<B::Of>(),
                TypeId::of::<C::Of>(),
                TypeId::of::<D::Of>(),
                TypeId::of::<E::Of>(),
                TypeId::of::<F::Of>(),
                TypeId::of::<G::Of>(),
                TypeId::of::<H::Of>(),
                TypeId::of::<I::Of>(),
            ])
        }
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
            <[_]>::into_vec(box [
                A::guards_from_archetype(archetype),
                B::guards_from_archetype(archetype),
                C::guards_from_archetype(archetype),
                D::guards_from_archetype(archetype),
                E::guards_from_archetype(archetype),
                F::guards_from_archetype(archetype),
                G::guards_from_archetype(archetype),
                H::guards_from_archetype(archetype),
                I::guards_from_archetype(archetype),
            ])
        }
    }
    impl<
            'g: 'b,
            'b,
            A: Borrow<'b>,
            B: Borrow<'b>,
            C: Borrow<'b>,
            D: Borrow<'b>,
            E: Borrow<'b>,
            F: Borrow<'b>,
            G: Borrow<'b>,
            H: Borrow<'b>,
            I: Borrow<'b>,
        > QueryBorrow<'b, 'g, (A, B, C, D, E, F, G, H, I)>
    {
        pub fn into_for_each_mut<Func: FnMut((A, B, C, D, E, F, G, H, I))>(
            &'b mut self,
            mut func: Func,
        ) {
            let arity = <(A, B, C, D, E, F, G, H, I) as QueryInfos>::arity();
            if true {
                if !(self.lock_guards.len() % arity == 0) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: self.lock_guards.len() % arity == 0",
                        )
                    }
                };
            };
            let mut iterators = Vec::with_capacity(self.lock_guards.len());
            for chunk in self.lock_guards.chunks_mut(arity) {
                let iter = <(
                    A::Iter,
                    B::Iter,
                    C::Iter,
                    D::Iter,
                    E::Iter,
                    F::Iter,
                    G::Iter,
                    H::Iter,
                    I::Iter,
                ) as Iters<(A, B, C, D, E, F, G, H, I)>>::iter_from_guards(
                    chunk
                );
                let iter: ItersIterator<'_, (A, B, C, D, E, F, G, H, I), _> =
                    ItersIterator::new(iter);
                iterators.push(iter);
            }
            for iter in iterators {
                for item in iter {
                    func(item);
                }
            }
        }
    }
    impl<
            'a,
            A: Borrow<'a>,
            B: Borrow<'a>,
            C: Borrow<'a>,
            D: Borrow<'a>,
            E: Borrow<'a>,
            F: Borrow<'a>,
            G: Borrow<'a>,
            H: Borrow<'a>,
            I: Borrow<'a>,
        > Iters<'a, (A, B, C, D, E, F, G, H, I)>
        for (
            A::Iter,
            B::Iter,
            C::Iter,
            D::Iter,
            E::Iter,
            F::Iter,
            G::Iter,
            H::Iter,
            I::Iter,
        )
    {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
            let mut iter = locks.iter_mut();
            let mut length = None;
            (
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <A as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <B as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <C as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <D as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <E as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <F as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <G as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <H as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <I as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
            )
        }
        #[allow(non_snake_case)]
        #[inline(always)]
        fn next(&mut self) -> Option<(A, B, C, D, E, F, G, H, I)> {
            if !self.0.is_next_some() {
                return None;
            }
            let (A, B, C, D, E, F, G, H, I) = self;
            Some((
                unsafe { <A as Borrow<'a>>::borrow_from_iter_unchecked(A) },
                unsafe { <B as Borrow<'a>>::borrow_from_iter_unchecked(B) },
                unsafe { <C as Borrow<'a>>::borrow_from_iter_unchecked(C) },
                unsafe { <D as Borrow<'a>>::borrow_from_iter_unchecked(D) },
                unsafe { <E as Borrow<'a>>::borrow_from_iter_unchecked(E) },
                unsafe { <F as Borrow<'a>>::borrow_from_iter_unchecked(F) },
                unsafe { <G as Borrow<'a>>::borrow_from_iter_unchecked(G) },
                unsafe { <H as Borrow<'a>>::borrow_from_iter_unchecked(H) },
                unsafe { <I as Borrow<'a>>::borrow_from_iter_unchecked(I) },
            ))
        }
        fn new_empty() -> Self {
            (
                <A as Borrow<'a>>::iter_empty(),
                <B as Borrow<'a>>::iter_empty(),
                <C as Borrow<'a>>::iter_empty(),
                <D as Borrow<'a>>::iter_empty(),
                <E as Borrow<'a>>::iter_empty(),
                <F as Borrow<'a>>::iter_empty(),
                <G as Borrow<'a>>::iter_empty(),
                <H as Borrow<'a>>::iter_empty(),
                <I as Borrow<'a>>::iter_empty(),
            )
        }
    }
    impl<
            'b,
            A: Borrow<'b>,
            B: Borrow<'b>,
            C: Borrow<'b>,
            D: Borrow<'b>,
            E: Borrow<'b>,
            F: Borrow<'b>,
            G: Borrow<'b>,
            H: Borrow<'b>,
        > QueryInfos for (A, B, C, D, E, F, G, H)
    {
        #[allow(unused, non_snake_case)]
        fn arity() -> usize {
            let mut count = 0;
            let A = ();
            count += 1;
            let B = ();
            count += 1;
            let C = ();
            count += 1;
            let D = ();
            count += 1;
            let E = ();
            count += 1;
            let F = ();
            count += 1;
            let G = ();
            count += 1;
            let H = ();
            count += 1;
            count
        }
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A::Of>(),
                TypeId::of::<B::Of>(),
                TypeId::of::<C::Of>(),
                TypeId::of::<D::Of>(),
                TypeId::of::<E::Of>(),
                TypeId::of::<F::Of>(),
                TypeId::of::<G::Of>(),
                TypeId::of::<H::Of>(),
            ])
        }
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
            <[_]>::into_vec(box [
                A::guards_from_archetype(archetype),
                B::guards_from_archetype(archetype),
                C::guards_from_archetype(archetype),
                D::guards_from_archetype(archetype),
                E::guards_from_archetype(archetype),
                F::guards_from_archetype(archetype),
                G::guards_from_archetype(archetype),
                H::guards_from_archetype(archetype),
            ])
        }
    }
    impl<
            'g: 'b,
            'b,
            A: Borrow<'b>,
            B: Borrow<'b>,
            C: Borrow<'b>,
            D: Borrow<'b>,
            E: Borrow<'b>,
            F: Borrow<'b>,
            G: Borrow<'b>,
            H: Borrow<'b>,
        > QueryBorrow<'b, 'g, (A, B, C, D, E, F, G, H)>
    {
        pub fn into_for_each_mut<Func: FnMut((A, B, C, D, E, F, G, H))>(
            &'b mut self,
            mut func: Func,
        ) {
            let arity = <(A, B, C, D, E, F, G, H) as QueryInfos>::arity();
            if true {
                if !(self.lock_guards.len() % arity == 0) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: self.lock_guards.len() % arity == 0",
                        )
                    }
                };
            };
            let mut iterators = Vec::with_capacity(self.lock_guards.len());
            for chunk in self.lock_guards.chunks_mut(arity) {
                let iter = <(
                    A::Iter,
                    B::Iter,
                    C::Iter,
                    D::Iter,
                    E::Iter,
                    F::Iter,
                    G::Iter,
                    H::Iter,
                ) as Iters<(A, B, C, D, E, F, G, H)>>::iter_from_guards(
                    chunk
                );
                let iter: ItersIterator<'_, (A, B, C, D, E, F, G, H), _> = ItersIterator::new(iter);
                iterators.push(iter);
            }
            for iter in iterators {
                for item in iter {
                    func(item);
                }
            }
        }
    }
    impl<
            'a,
            A: Borrow<'a>,
            B: Borrow<'a>,
            C: Borrow<'a>,
            D: Borrow<'a>,
            E: Borrow<'a>,
            F: Borrow<'a>,
            G: Borrow<'a>,
            H: Borrow<'a>,
        > Iters<'a, (A, B, C, D, E, F, G, H)>
        for (
            A::Iter,
            B::Iter,
            C::Iter,
            D::Iter,
            E::Iter,
            F::Iter,
            G::Iter,
            H::Iter,
        )
    {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
            let mut iter = locks.iter_mut();
            let mut length = None;
            (
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <A as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <B as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <C as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <D as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <E as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <F as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <G as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <H as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
            )
        }
        #[allow(non_snake_case)]
        #[inline(always)]
        fn next(&mut self) -> Option<(A, B, C, D, E, F, G, H)> {
            if !self.0.is_next_some() {
                return None;
            }
            let (A, B, C, D, E, F, G, H) = self;
            Some((
                unsafe { <A as Borrow<'a>>::borrow_from_iter_unchecked(A) },
                unsafe { <B as Borrow<'a>>::borrow_from_iter_unchecked(B) },
                unsafe { <C as Borrow<'a>>::borrow_from_iter_unchecked(C) },
                unsafe { <D as Borrow<'a>>::borrow_from_iter_unchecked(D) },
                unsafe { <E as Borrow<'a>>::borrow_from_iter_unchecked(E) },
                unsafe { <F as Borrow<'a>>::borrow_from_iter_unchecked(F) },
                unsafe { <G as Borrow<'a>>::borrow_from_iter_unchecked(G) },
                unsafe { <H as Borrow<'a>>::borrow_from_iter_unchecked(H) },
            ))
        }
        fn new_empty() -> Self {
            (
                <A as Borrow<'a>>::iter_empty(),
                <B as Borrow<'a>>::iter_empty(),
                <C as Borrow<'a>>::iter_empty(),
                <D as Borrow<'a>>::iter_empty(),
                <E as Borrow<'a>>::iter_empty(),
                <F as Borrow<'a>>::iter_empty(),
                <G as Borrow<'a>>::iter_empty(),
                <H as Borrow<'a>>::iter_empty(),
            )
        }
    }
    impl<
            'b,
            A: Borrow<'b>,
            B: Borrow<'b>,
            C: Borrow<'b>,
            D: Borrow<'b>,
            E: Borrow<'b>,
            F: Borrow<'b>,
            G: Borrow<'b>,
        > QueryInfos for (A, B, C, D, E, F, G)
    {
        #[allow(unused, non_snake_case)]
        fn arity() -> usize {
            let mut count = 0;
            let A = ();
            count += 1;
            let B = ();
            count += 1;
            let C = ();
            count += 1;
            let D = ();
            count += 1;
            let E = ();
            count += 1;
            let F = ();
            count += 1;
            let G = ();
            count += 1;
            count
        }
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A::Of>(),
                TypeId::of::<B::Of>(),
                TypeId::of::<C::Of>(),
                TypeId::of::<D::Of>(),
                TypeId::of::<E::Of>(),
                TypeId::of::<F::Of>(),
                TypeId::of::<G::Of>(),
            ])
        }
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
            <[_]>::into_vec(box [
                A::guards_from_archetype(archetype),
                B::guards_from_archetype(archetype),
                C::guards_from_archetype(archetype),
                D::guards_from_archetype(archetype),
                E::guards_from_archetype(archetype),
                F::guards_from_archetype(archetype),
                G::guards_from_archetype(archetype),
            ])
        }
    }
    impl<
            'g: 'b,
            'b,
            A: Borrow<'b>,
            B: Borrow<'b>,
            C: Borrow<'b>,
            D: Borrow<'b>,
            E: Borrow<'b>,
            F: Borrow<'b>,
            G: Borrow<'b>,
        > QueryBorrow<'b, 'g, (A, B, C, D, E, F, G)>
    {
        pub fn into_for_each_mut<Func: FnMut((A, B, C, D, E, F, G))>(&'b mut self, mut func: Func) {
            let arity = <(A, B, C, D, E, F, G) as QueryInfos>::arity();
            if true {
                if !(self.lock_guards.len() % arity == 0) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: self.lock_guards.len() % arity == 0",
                        )
                    }
                };
            };
            let mut iterators = Vec::with_capacity(self.lock_guards.len());
            for chunk in self.lock_guards.chunks_mut(arity) {
                let iter =
                    <(
                        A::Iter,
                        B::Iter,
                        C::Iter,
                        D::Iter,
                        E::Iter,
                        F::Iter,
                        G::Iter,
                    ) as Iters<(A, B, C, D, E, F, G)>>::iter_from_guards(chunk);
                let iter: ItersIterator<'_, (A, B, C, D, E, F, G), _> = ItersIterator::new(iter);
                iterators.push(iter);
            }
            for iter in iterators {
                for item in iter {
                    func(item);
                }
            }
        }
    }
    impl<
            'a,
            A: Borrow<'a>,
            B: Borrow<'a>,
            C: Borrow<'a>,
            D: Borrow<'a>,
            E: Borrow<'a>,
            F: Borrow<'a>,
            G: Borrow<'a>,
        > Iters<'a, (A, B, C, D, E, F, G)>
        for (
            A::Iter,
            B::Iter,
            C::Iter,
            D::Iter,
            E::Iter,
            F::Iter,
            G::Iter,
        )
    {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
            let mut iter = locks.iter_mut();
            let mut length = None;
            (
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <A as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <B as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <C as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <D as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <E as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <F as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <G as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
            )
        }
        #[allow(non_snake_case)]
        #[inline(always)]
        fn next(&mut self) -> Option<(A, B, C, D, E, F, G)> {
            if !self.0.is_next_some() {
                return None;
            }
            let (A, B, C, D, E, F, G) = self;
            Some((
                unsafe { <A as Borrow<'a>>::borrow_from_iter_unchecked(A) },
                unsafe { <B as Borrow<'a>>::borrow_from_iter_unchecked(B) },
                unsafe { <C as Borrow<'a>>::borrow_from_iter_unchecked(C) },
                unsafe { <D as Borrow<'a>>::borrow_from_iter_unchecked(D) },
                unsafe { <E as Borrow<'a>>::borrow_from_iter_unchecked(E) },
                unsafe { <F as Borrow<'a>>::borrow_from_iter_unchecked(F) },
                unsafe { <G as Borrow<'a>>::borrow_from_iter_unchecked(G) },
            ))
        }
        fn new_empty() -> Self {
            (
                <A as Borrow<'a>>::iter_empty(),
                <B as Borrow<'a>>::iter_empty(),
                <C as Borrow<'a>>::iter_empty(),
                <D as Borrow<'a>>::iter_empty(),
                <E as Borrow<'a>>::iter_empty(),
                <F as Borrow<'a>>::iter_empty(),
                <G as Borrow<'a>>::iter_empty(),
            )
        }
    }
    impl<
            'b,
            A: Borrow<'b>,
            B: Borrow<'b>,
            C: Borrow<'b>,
            D: Borrow<'b>,
            E: Borrow<'b>,
            F: Borrow<'b>,
        > QueryInfos for (A, B, C, D, E, F)
    {
        #[allow(unused, non_snake_case)]
        fn arity() -> usize {
            let mut count = 0;
            let A = ();
            count += 1;
            let B = ();
            count += 1;
            let C = ();
            count += 1;
            let D = ();
            count += 1;
            let E = ();
            count += 1;
            let F = ();
            count += 1;
            count
        }
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A::Of>(),
                TypeId::of::<B::Of>(),
                TypeId::of::<C::Of>(),
                TypeId::of::<D::Of>(),
                TypeId::of::<E::Of>(),
                TypeId::of::<F::Of>(),
            ])
        }
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
            <[_]>::into_vec(box [
                A::guards_from_archetype(archetype),
                B::guards_from_archetype(archetype),
                C::guards_from_archetype(archetype),
                D::guards_from_archetype(archetype),
                E::guards_from_archetype(archetype),
                F::guards_from_archetype(archetype),
            ])
        }
    }
    impl<
            'g: 'b,
            'b,
            A: Borrow<'b>,
            B: Borrow<'b>,
            C: Borrow<'b>,
            D: Borrow<'b>,
            E: Borrow<'b>,
            F: Borrow<'b>,
        > QueryBorrow<'b, 'g, (A, B, C, D, E, F)>
    {
        pub fn into_for_each_mut<Func: FnMut((A, B, C, D, E, F))>(&'b mut self, mut func: Func) {
            let arity = <(A, B, C, D, E, F) as QueryInfos>::arity();
            if true {
                if !(self.lock_guards.len() % arity == 0) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: self.lock_guards.len() % arity == 0",
                        )
                    }
                };
            };
            let mut iterators = Vec::with_capacity(self.lock_guards.len());
            for chunk in self.lock_guards.chunks_mut(arity) {
                let iter = <(A::Iter, B::Iter, C::Iter, D::Iter, E::Iter, F::Iter) as Iters<(
                    A,
                    B,
                    C,
                    D,
                    E,
                    F,
                )>>::iter_from_guards(chunk);
                let iter: ItersIterator<'_, (A, B, C, D, E, F), _> = ItersIterator::new(iter);
                iterators.push(iter);
            }
            for iter in iterators {
                for item in iter {
                    func(item);
                }
            }
        }
    }
    impl<
            'a,
            A: Borrow<'a>,
            B: Borrow<'a>,
            C: Borrow<'a>,
            D: Borrow<'a>,
            E: Borrow<'a>,
            F: Borrow<'a>,
        > Iters<'a, (A, B, C, D, E, F)> for (A::Iter, B::Iter, C::Iter, D::Iter, E::Iter, F::Iter)
    {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
            let mut iter = locks.iter_mut();
            let mut length = None;
            (
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <A as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <B as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <C as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <D as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <E as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <F as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
            )
        }
        #[allow(non_snake_case)]
        #[inline(always)]
        fn next(&mut self) -> Option<(A, B, C, D, E, F)> {
            if !self.0.is_next_some() {
                return None;
            }
            let (A, B, C, D, E, F) = self;
            Some((
                unsafe { <A as Borrow<'a>>::borrow_from_iter_unchecked(A) },
                unsafe { <B as Borrow<'a>>::borrow_from_iter_unchecked(B) },
                unsafe { <C as Borrow<'a>>::borrow_from_iter_unchecked(C) },
                unsafe { <D as Borrow<'a>>::borrow_from_iter_unchecked(D) },
                unsafe { <E as Borrow<'a>>::borrow_from_iter_unchecked(E) },
                unsafe { <F as Borrow<'a>>::borrow_from_iter_unchecked(F) },
            ))
        }
        fn new_empty() -> Self {
            (
                <A as Borrow<'a>>::iter_empty(),
                <B as Borrow<'a>>::iter_empty(),
                <C as Borrow<'a>>::iter_empty(),
                <D as Borrow<'a>>::iter_empty(),
                <E as Borrow<'a>>::iter_empty(),
                <F as Borrow<'a>>::iter_empty(),
            )
        }
    }
    impl<'b, A: Borrow<'b>, B: Borrow<'b>, C: Borrow<'b>, D: Borrow<'b>, E: Borrow<'b>> QueryInfos
        for (A, B, C, D, E)
    {
        #[allow(unused, non_snake_case)]
        fn arity() -> usize {
            let mut count = 0;
            let A = ();
            count += 1;
            let B = ();
            count += 1;
            let C = ();
            count += 1;
            let D = ();
            count += 1;
            let E = ();
            count += 1;
            count
        }
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A::Of>(),
                TypeId::of::<B::Of>(),
                TypeId::of::<C::Of>(),
                TypeId::of::<D::Of>(),
                TypeId::of::<E::Of>(),
            ])
        }
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
            <[_]>::into_vec(box [
                A::guards_from_archetype(archetype),
                B::guards_from_archetype(archetype),
                C::guards_from_archetype(archetype),
                D::guards_from_archetype(archetype),
                E::guards_from_archetype(archetype),
            ])
        }
    }
    impl<'g: 'b, 'b, A: Borrow<'b>, B: Borrow<'b>, C: Borrow<'b>, D: Borrow<'b>, E: Borrow<'b>>
        QueryBorrow<'b, 'g, (A, B, C, D, E)>
    {
        pub fn into_for_each_mut<Func: FnMut((A, B, C, D, E))>(&'b mut self, mut func: Func) {
            let arity = <(A, B, C, D, E) as QueryInfos>::arity();
            if true {
                if !(self.lock_guards.len() % arity == 0) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: self.lock_guards.len() % arity == 0",
                        )
                    }
                };
            };
            let mut iterators = Vec::with_capacity(self.lock_guards.len());
            for chunk in self.lock_guards.chunks_mut(arity) {
                let iter = <(A::Iter, B::Iter, C::Iter, D::Iter, E::Iter) as Iters<(
                    A,
                    B,
                    C,
                    D,
                    E,
                )>>::iter_from_guards(chunk);
                let iter: ItersIterator<'_, (A, B, C, D, E), _> = ItersIterator::new(iter);
                iterators.push(iter);
            }
            for iter in iterators {
                for item in iter {
                    func(item);
                }
            }
        }
    }
    impl<'a, A: Borrow<'a>, B: Borrow<'a>, C: Borrow<'a>, D: Borrow<'a>, E: Borrow<'a>>
        Iters<'a, (A, B, C, D, E)> for (A::Iter, B::Iter, C::Iter, D::Iter, E::Iter)
    {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
            let mut iter = locks.iter_mut();
            let mut length = None;
            (
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <A as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <B as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <C as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <D as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <E as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
            )
        }
        #[allow(non_snake_case)]
        #[inline(always)]
        fn next(&mut self) -> Option<(A, B, C, D, E)> {
            if !self.0.is_next_some() {
                return None;
            }
            let (A, B, C, D, E) = self;
            Some((
                unsafe { <A as Borrow<'a>>::borrow_from_iter_unchecked(A) },
                unsafe { <B as Borrow<'a>>::borrow_from_iter_unchecked(B) },
                unsafe { <C as Borrow<'a>>::borrow_from_iter_unchecked(C) },
                unsafe { <D as Borrow<'a>>::borrow_from_iter_unchecked(D) },
                unsafe { <E as Borrow<'a>>::borrow_from_iter_unchecked(E) },
            ))
        }
        fn new_empty() -> Self {
            (
                <A as Borrow<'a>>::iter_empty(),
                <B as Borrow<'a>>::iter_empty(),
                <C as Borrow<'a>>::iter_empty(),
                <D as Borrow<'a>>::iter_empty(),
                <E as Borrow<'a>>::iter_empty(),
            )
        }
    }
    impl<'b, A: Borrow<'b>, B: Borrow<'b>, C: Borrow<'b>, D: Borrow<'b>> QueryInfos for (A, B, C, D) {
        #[allow(unused, non_snake_case)]
        fn arity() -> usize {
            let mut count = 0;
            let A = ();
            count += 1;
            let B = ();
            count += 1;
            let C = ();
            count += 1;
            let D = ();
            count += 1;
            count
        }
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A::Of>(),
                TypeId::of::<B::Of>(),
                TypeId::of::<C::Of>(),
                TypeId::of::<D::Of>(),
            ])
        }
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
            <[_]>::into_vec(box [
                A::guards_from_archetype(archetype),
                B::guards_from_archetype(archetype),
                C::guards_from_archetype(archetype),
                D::guards_from_archetype(archetype),
            ])
        }
    }
    impl<'g: 'b, 'b, A: Borrow<'b>, B: Borrow<'b>, C: Borrow<'b>, D: Borrow<'b>>
        QueryBorrow<'b, 'g, (A, B, C, D)>
    {
        pub fn into_for_each_mut<Func: FnMut((A, B, C, D))>(&'b mut self, mut func: Func) {
            let arity = <(A, B, C, D) as QueryInfos>::arity();
            if true {
                if !(self.lock_guards.len() % arity == 0) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: self.lock_guards.len() % arity == 0",
                        )
                    }
                };
            };
            let mut iterators = Vec::with_capacity(self.lock_guards.len());
            for chunk in self.lock_guards.chunks_mut(arity) {
                let iter =
                    <(A::Iter, B::Iter, C::Iter, D::Iter) as Iters<(A, B, C, D)>>::iter_from_guards(
                        chunk,
                    );
                let iter: ItersIterator<'_, (A, B, C, D), _> = ItersIterator::new(iter);
                iterators.push(iter);
            }
            for iter in iterators {
                for item in iter {
                    func(item);
                }
            }
        }
    }
    impl<'a, A: Borrow<'a>, B: Borrow<'a>, C: Borrow<'a>, D: Borrow<'a>> Iters<'a, (A, B, C, D)>
        for (A::Iter, B::Iter, C::Iter, D::Iter)
    {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
            let mut iter = locks.iter_mut();
            let mut length = None;
            (
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <A as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <B as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <C as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <D as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
            )
        }
        #[allow(non_snake_case)]
        #[inline(always)]
        fn next(&mut self) -> Option<(A, B, C, D)> {
            if !self.0.is_next_some() {
                return None;
            }
            let (A, B, C, D) = self;
            Some((
                unsafe { <A as Borrow<'a>>::borrow_from_iter_unchecked(A) },
                unsafe { <B as Borrow<'a>>::borrow_from_iter_unchecked(B) },
                unsafe { <C as Borrow<'a>>::borrow_from_iter_unchecked(C) },
                unsafe { <D as Borrow<'a>>::borrow_from_iter_unchecked(D) },
            ))
        }
        fn new_empty() -> Self {
            (
                <A as Borrow<'a>>::iter_empty(),
                <B as Borrow<'a>>::iter_empty(),
                <C as Borrow<'a>>::iter_empty(),
                <D as Borrow<'a>>::iter_empty(),
            )
        }
    }
    impl<'b, A: Borrow<'b>, B: Borrow<'b>, C: Borrow<'b>> QueryInfos for (A, B, C) {
        #[allow(unused, non_snake_case)]
        fn arity() -> usize {
            let mut count = 0;
            let A = ();
            count += 1;
            let B = ();
            count += 1;
            let C = ();
            count += 1;
            count
        }
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A::Of>(),
                TypeId::of::<B::Of>(),
                TypeId::of::<C::Of>(),
            ])
        }
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
            <[_]>::into_vec(box [
                A::guards_from_archetype(archetype),
                B::guards_from_archetype(archetype),
                C::guards_from_archetype(archetype),
            ])
        }
    }
    impl<'g: 'b, 'b, A: Borrow<'b>, B: Borrow<'b>, C: Borrow<'b>> QueryBorrow<'b, 'g, (A, B, C)> {
        pub fn into_for_each_mut<Func: FnMut((A, B, C))>(&'b mut self, mut func: Func) {
            let arity = <(A, B, C) as QueryInfos>::arity();
            if true {
                if !(self.lock_guards.len() % arity == 0) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: self.lock_guards.len() % arity == 0",
                        )
                    }
                };
            };
            let mut iterators = Vec::with_capacity(self.lock_guards.len());
            for chunk in self.lock_guards.chunks_mut(arity) {
                let iter =
                    <(A::Iter, B::Iter, C::Iter) as Iters<(A, B, C)>>::iter_from_guards(chunk);
                let iter: ItersIterator<'_, (A, B, C), _> = ItersIterator::new(iter);
                iterators.push(iter);
            }
            for iter in iterators {
                for item in iter {
                    func(item);
                }
            }
        }
    }
    impl<'a, A: Borrow<'a>, B: Borrow<'a>, C: Borrow<'a>> Iters<'a, (A, B, C)>
        for (A::Iter, B::Iter, C::Iter)
    {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
            let mut iter = locks.iter_mut();
            let mut length = None;
            (
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <A as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <B as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <C as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
            )
        }
        #[allow(non_snake_case)]
        #[inline(always)]
        fn next(&mut self) -> Option<(A, B, C)> {
            if !self.0.is_next_some() {
                return None;
            }
            let (A, B, C) = self;
            Some((
                unsafe { <A as Borrow<'a>>::borrow_from_iter_unchecked(A) },
                unsafe { <B as Borrow<'a>>::borrow_from_iter_unchecked(B) },
                unsafe { <C as Borrow<'a>>::borrow_from_iter_unchecked(C) },
            ))
        }
        fn new_empty() -> Self {
            (
                <A as Borrow<'a>>::iter_empty(),
                <B as Borrow<'a>>::iter_empty(),
                <C as Borrow<'a>>::iter_empty(),
            )
        }
    }
    impl<'b, A: Borrow<'b>, B: Borrow<'b>> QueryInfos for (A, B) {
        #[allow(unused, non_snake_case)]
        fn arity() -> usize {
            let mut count = 0;
            let A = ();
            count += 1;
            let B = ();
            count += 1;
            count
        }
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [TypeId::of::<A::Of>(), TypeId::of::<B::Of>()])
        }
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
            <[_]>::into_vec(box [
                A::guards_from_archetype(archetype),
                B::guards_from_archetype(archetype),
            ])
        }
    }
    impl<'g: 'b, 'b, A: Borrow<'b>, B: Borrow<'b>> QueryBorrow<'b, 'g, (A, B)> {
        pub fn into_for_each_mut<Func: FnMut((A, B))>(&'b mut self, mut func: Func) {
            let arity = <(A, B) as QueryInfos>::arity();
            if true {
                if !(self.lock_guards.len() % arity == 0) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: self.lock_guards.len() % arity == 0",
                        )
                    }
                };
            };
            let mut iterators = Vec::with_capacity(self.lock_guards.len());
            for chunk in self.lock_guards.chunks_mut(arity) {
                let iter = <(A::Iter, B::Iter) as Iters<(A, B)>>::iter_from_guards(chunk);
                let iter: ItersIterator<'_, (A, B), _> = ItersIterator::new(iter);
                iterators.push(iter);
            }
            for iter in iterators {
                for item in iter {
                    func(item);
                }
            }
        }
    }
    impl<'a, A: Borrow<'a>, B: Borrow<'a>> Iters<'a, (A, B)> for (A::Iter, B::Iter) {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
            let mut iter = locks.iter_mut();
            let mut length = None;
            (
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <A as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
                {
                    let guard = iter.next().unwrap();
                    let (len, iter) = <B as Borrow<'a>>::iter_from_guard(guard);
                    if length.is_none() {
                        length = Some(len);
                    }
                    {
                        match (&length.unwrap(), &len) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    {
                                        ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ))
                                    }
                                }
                            }
                        }
                    };
                    iter
                },
            )
        }
        #[allow(non_snake_case)]
        #[inline(always)]
        fn next(&mut self) -> Option<(A, B)> {
            if !self.0.is_next_some() {
                return None;
            }
            let (A, B) = self;
            Some((
                unsafe { <A as Borrow<'a>>::borrow_from_iter_unchecked(A) },
                unsafe { <B as Borrow<'a>>::borrow_from_iter_unchecked(B) },
            ))
        }
        fn new_empty() -> Self {
            (
                <A as Borrow<'a>>::iter_empty(),
                <B as Borrow<'a>>::iter_empty(),
            )
        }
    }
    impl<'b, A: Borrow<'b>> QueryInfos for (A,) {
        #[allow(unused, non_snake_case)]
        fn arity() -> usize {
            let mut count = 0;
            let A = ();
            count += 1;
            count
        }
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [TypeId::of::<A::Of>()])
        }
        fn borrow_guards<'guard>(archetype: &'guard Archetype) -> Vec<RwLockEitherGuard<'guard>> {
            <[_]>::into_vec(box [A::guards_from_archetype(archetype)])
        }
    }
    impl<'g: 'b, 'b, A: Borrow<'b>> QueryBorrow<'b, 'g, (A,)> {
        pub fn into_for_each_mut<Func: FnMut((A,))>(&'b mut self, mut func: Func) {
            let arity = <(A,) as QueryInfos>::arity();
            if true {
                if !(self.lock_guards.len() % arity == 0) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: self.lock_guards.len() % arity == 0",
                        )
                    }
                };
            };
            let mut iterators = Vec::with_capacity(self.lock_guards.len());
            for chunk in self.lock_guards.chunks_mut(arity) {
                let iter = <(A::Iter,) as Iters<(A,)>>::iter_from_guards(chunk);
                let iter: ItersIterator<'_, (A,), _> = ItersIterator::new(iter);
                iterators.push(iter);
            }
            for iter in iterators {
                for item in iter {
                    func(item);
                }
            }
        }
    }
    impl<'a, A: Borrow<'a>> Iters<'a, (A,)> for (A::Iter,) {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self {
            let mut iter = locks.iter_mut();
            let mut length = None;
            ({
                let guard = iter.next().unwrap();
                let (len, iter) = <A as Borrow<'a>>::iter_from_guard(guard);
                if length.is_none() {
                    length = Some(len);
                }
                {
                    match (&length.unwrap(), &len) {
                        (left_val, right_val) => {
                            if !(*left_val == *right_val) {
                                {
                                    ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                                        &[
                                            "assertion failed: `(left == right)`\n  left: `",
                                            "`,\n right: `",
                                            "`",
                                        ],
                                        &match (&&*left_val, &&*right_val) {
                                            (arg0, arg1) => [
                                                ::core::fmt::ArgumentV1::new(
                                                    arg0,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                                ::core::fmt::ArgumentV1::new(
                                                    arg1,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                            ],
                                        },
                                    ))
                                }
                            }
                        }
                    }
                };
                iter
            },)
        }
        #[allow(non_snake_case)]
        #[inline(always)]
        fn next(&mut self) -> Option<(A,)> {
            if !self.0.is_next_some() {
                return None;
            }
            let (A,) = self;
            Some((unsafe { <A as Borrow<'a>>::borrow_from_iter_unchecked(A) },))
        }
        fn new_empty() -> Self {
            (<A as Borrow<'a>>::iter_empty(),)
        }
    }
    pub struct ItersIterator<'a, U: QueryInfos, T: Iters<'a, U>> {
        iter: T,
        phantom: PhantomData<&'a U>,
    }
    impl<'a, U: QueryInfos, T: Iters<'a, U>> Iterator for ItersIterator<'a, U, T> {
        type Item = U;
        #[inline(always)]
        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next()
        }
    }
    impl<'a, U: QueryInfos, T: Iters<'a, U>> ItersIterator<'a, U, T> {
        pub fn new(iter: T) -> Self {
            Self {
                iter,
                phantom: PhantomData,
            }
        }
    }
    pub trait Iters<'a, T: QueryInfos> {
        fn iter_from_guards<'guard: 'a>(locks: &'a mut [RwLockEitherGuard<'guard>]) -> Self;
        fn next(&mut self) -> Option<T>;
        fn new_empty() -> Self;
    }
    pub trait BorrowIterator: Iterator {
        fn is_next_some(&mut self) -> bool;
    }
    impl<T: Iterator> BorrowIterator for Peekable<T> {
        fn is_next_some(&mut self) -> bool {
            self.peek().is_some()
        }
    }
    pub unsafe trait Borrow<'b>: Sized {
        type Of: 'static;
        type Iter: BorrowIterator;
        fn iter_from_guard<'guard: 'b>(
            guard: &'b mut RwLockEitherGuard<'guard>,
        ) -> (usize, Self::Iter);
        fn borrow_from_iter<'a>(iter: &'a mut Self::Iter) -> Option<Self>;
        unsafe fn borrow_from_iter_unchecked<'a>(iter: &'a mut Self::Iter) -> Self;
        fn guards_from_archetype<'guard>(archetype: &'guard Archetype)
            -> RwLockEitherGuard<'guard>;
        fn iter_empty<'a>() -> Self::Iter;
    }
    unsafe impl<'b, T: 'static> Borrow<'b> for &'b T {
        type Of = T;
        type Iter = Peekable<Iter<'b, T>>;
        fn iter_from_guard<'guard: 'b>(
            guard: &'b mut RwLockEitherGuard<'guard>,
        ) -> (usize, Self::Iter) {
            match guard {
                RwLockEitherGuard::ReadGuard(guard) => {
                    let vec = guard.downcast_ref::<Vec<T>>().unwrap();
                    (vec.len(), vec.iter().peekable())
                }
                _ => ::std::rt::begin_panic("internal error: entered unreachable code"),
            }
        }
        #[inline(always)]
        fn borrow_from_iter<'a>(iter: &'a mut Self::Iter) -> Option<Self> {
            iter.next()
        }
        #[inline(always)]
        #[allow(unused_unsafe)]
        unsafe fn borrow_from_iter_unchecked<'a>(iter: &'a mut Self::Iter) -> Self {
            match iter.next() {
                Some(item) => return item,
                None => unsafe { std::hint::unreachable_unchecked() },
            }
        }
        fn guards_from_archetype<'guard>(
            archetype: &'guard Archetype,
        ) -> RwLockEitherGuard<'guard> {
            RwLockEitherGuard::ReadGuard(archetype.data.get::<Vec<T>>().unwrap().guard)
        }
        fn iter_empty<'a>() -> Self::Iter {
            [].iter().peekable()
        }
    }
    unsafe impl<'b, T: 'static> Borrow<'b> for &'b mut T {
        type Of = T;
        type Iter = Peekable<IterMut<'b, T>>;
        fn iter_from_guard<'guard: 'b>(
            guard: &'b mut RwLockEitherGuard<'guard>,
        ) -> (usize, Self::Iter) {
            match guard {
                RwLockEitherGuard::WriteGuard(guard) => {
                    let vec = guard.downcast_mut::<Vec<T>>().unwrap();
                    (vec.len(), vec.iter_mut().peekable())
                }
                _ => ::std::rt::begin_panic("internal error: entered unreachable code"),
            }
        }
        #[inline(always)]
        fn borrow_from_iter<'a>(iter: &'a mut Self::Iter) -> Option<Self> {
            iter.next()
        }
        #[inline(always)]
        #[allow(unused_unsafe)]
        unsafe fn borrow_from_iter_unchecked<'a>(iter: &'a mut Self::Iter) -> Self {
            match iter.next() {
                Some(item) => item,
                None => unsafe { std::hint::unreachable_unchecked() },
            }
        }
        fn guards_from_archetype<'guard>(
            archetype: &'guard Archetype,
        ) -> RwLockEitherGuard<'guard> {
            RwLockEitherGuard::WriteGuard(archetype.data.get_mut::<Vec<T>>().unwrap().guard)
        }
        fn iter_empty<'a>() -> Self::Iter {
            [].iter_mut().peekable()
        }
    }
}
pub mod bundle {
    use super::anymap::AnyMap;
    use super::world::Archetype;
    use std::any::TypeId;
    use std::error::Error;
    pub trait Bundle {
        fn type_ids() -> Vec<TypeId>;
        fn new_archetype() -> Archetype;
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>>;
    }
    #[allow(non_snake_case)]
    impl<
            A: 'static,
            B: 'static,
            C: 'static,
            D: 'static,
            E: 'static,
            F: 'static,
            G: 'static,
            H: 'static,
            I: 'static,
            J: 'static,
        > Bundle for (A, B, C, D, E, F, G, H, I, J)
    {
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A>(),
                TypeId::of::<B>(),
                TypeId::of::<C>(),
                TypeId::of::<D>(),
                TypeId::of::<E>(),
                TypeId::of::<F>(),
                TypeId::of::<G>(),
                TypeId::of::<H>(),
                TypeId::of::<I>(),
                TypeId::of::<J>(),
            ])
        }
        fn new_archetype() -> Archetype {
            let type_ids = Self::type_ids();
            let mut data = AnyMap::new();
            let item = Vec::<A>::new();
            data.insert(item);
            let item = Vec::<B>::new();
            data.insert(item);
            let item = Vec::<C>::new();
            data.insert(item);
            let item = Vec::<D>::new();
            data.insert(item);
            let item = Vec::<E>::new();
            data.insert(item);
            let item = Vec::<F>::new();
            data.insert(item);
            let item = Vec::<G>::new();
            data.insert(item);
            let item = Vec::<H>::new();
            data.insert(item);
            let item = Vec::<I>::new();
            data.insert(item);
            let item = Vec::<J>::new();
            data.insert(item);
            Archetype { data, type_ids }
        }
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
            if true {
                if !(Self::type_ids() == archetype.type_ids) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: Self::type_ids() == archetype.type_ids",
                        )
                    }
                };
            };
            let (A, B, C, D, E, F, G, H, I, J) = self;
            archetype
                .data
                .get_mut_with_self::<Vec<A>>()
                .unwrap()
                .downcast_mut::<Vec<A>>()
                .unwrap()
                .push(A);
            archetype
                .data
                .get_mut_with_self::<Vec<B>>()
                .unwrap()
                .downcast_mut::<Vec<B>>()
                .unwrap()
                .push(B);
            archetype
                .data
                .get_mut_with_self::<Vec<C>>()
                .unwrap()
                .downcast_mut::<Vec<C>>()
                .unwrap()
                .push(C);
            archetype
                .data
                .get_mut_with_self::<Vec<D>>()
                .unwrap()
                .downcast_mut::<Vec<D>>()
                .unwrap()
                .push(D);
            archetype
                .data
                .get_mut_with_self::<Vec<E>>()
                .unwrap()
                .downcast_mut::<Vec<E>>()
                .unwrap()
                .push(E);
            archetype
                .data
                .get_mut_with_self::<Vec<F>>()
                .unwrap()
                .downcast_mut::<Vec<F>>()
                .unwrap()
                .push(F);
            archetype
                .data
                .get_mut_with_self::<Vec<G>>()
                .unwrap()
                .downcast_mut::<Vec<G>>()
                .unwrap()
                .push(G);
            archetype
                .data
                .get_mut_with_self::<Vec<H>>()
                .unwrap()
                .downcast_mut::<Vec<H>>()
                .unwrap()
                .push(H);
            archetype
                .data
                .get_mut_with_self::<Vec<I>>()
                .unwrap()
                .downcast_mut::<Vec<I>>()
                .unwrap()
                .push(I);
            archetype
                .data
                .get_mut_with_self::<Vec<J>>()
                .unwrap()
                .downcast_mut::<Vec<J>>()
                .unwrap()
                .push(J);
            Ok(())
        }
    }
    #[allow(non_snake_case)]
    impl<
            A: 'static,
            B: 'static,
            C: 'static,
            D: 'static,
            E: 'static,
            F: 'static,
            G: 'static,
            H: 'static,
            I: 'static,
        > Bundle for (A, B, C, D, E, F, G, H, I)
    {
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A>(),
                TypeId::of::<B>(),
                TypeId::of::<C>(),
                TypeId::of::<D>(),
                TypeId::of::<E>(),
                TypeId::of::<F>(),
                TypeId::of::<G>(),
                TypeId::of::<H>(),
                TypeId::of::<I>(),
            ])
        }
        fn new_archetype() -> Archetype {
            let type_ids = Self::type_ids();
            let mut data = AnyMap::new();
            let item = Vec::<A>::new();
            data.insert(item);
            let item = Vec::<B>::new();
            data.insert(item);
            let item = Vec::<C>::new();
            data.insert(item);
            let item = Vec::<D>::new();
            data.insert(item);
            let item = Vec::<E>::new();
            data.insert(item);
            let item = Vec::<F>::new();
            data.insert(item);
            let item = Vec::<G>::new();
            data.insert(item);
            let item = Vec::<H>::new();
            data.insert(item);
            let item = Vec::<I>::new();
            data.insert(item);
            Archetype { data, type_ids }
        }
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
            if true {
                if !(Self::type_ids() == archetype.type_ids) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: Self::type_ids() == archetype.type_ids",
                        )
                    }
                };
            };
            let (A, B, C, D, E, F, G, H, I) = self;
            archetype
                .data
                .get_mut_with_self::<Vec<A>>()
                .unwrap()
                .downcast_mut::<Vec<A>>()
                .unwrap()
                .push(A);
            archetype
                .data
                .get_mut_with_self::<Vec<B>>()
                .unwrap()
                .downcast_mut::<Vec<B>>()
                .unwrap()
                .push(B);
            archetype
                .data
                .get_mut_with_self::<Vec<C>>()
                .unwrap()
                .downcast_mut::<Vec<C>>()
                .unwrap()
                .push(C);
            archetype
                .data
                .get_mut_with_self::<Vec<D>>()
                .unwrap()
                .downcast_mut::<Vec<D>>()
                .unwrap()
                .push(D);
            archetype
                .data
                .get_mut_with_self::<Vec<E>>()
                .unwrap()
                .downcast_mut::<Vec<E>>()
                .unwrap()
                .push(E);
            archetype
                .data
                .get_mut_with_self::<Vec<F>>()
                .unwrap()
                .downcast_mut::<Vec<F>>()
                .unwrap()
                .push(F);
            archetype
                .data
                .get_mut_with_self::<Vec<G>>()
                .unwrap()
                .downcast_mut::<Vec<G>>()
                .unwrap()
                .push(G);
            archetype
                .data
                .get_mut_with_self::<Vec<H>>()
                .unwrap()
                .downcast_mut::<Vec<H>>()
                .unwrap()
                .push(H);
            archetype
                .data
                .get_mut_with_self::<Vec<I>>()
                .unwrap()
                .downcast_mut::<Vec<I>>()
                .unwrap()
                .push(I);
            Ok(())
        }
    }
    #[allow(non_snake_case)]
    impl<
            A: 'static,
            B: 'static,
            C: 'static,
            D: 'static,
            E: 'static,
            F: 'static,
            G: 'static,
            H: 'static,
        > Bundle for (A, B, C, D, E, F, G, H)
    {
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A>(),
                TypeId::of::<B>(),
                TypeId::of::<C>(),
                TypeId::of::<D>(),
                TypeId::of::<E>(),
                TypeId::of::<F>(),
                TypeId::of::<G>(),
                TypeId::of::<H>(),
            ])
        }
        fn new_archetype() -> Archetype {
            let type_ids = Self::type_ids();
            let mut data = AnyMap::new();
            let item = Vec::<A>::new();
            data.insert(item);
            let item = Vec::<B>::new();
            data.insert(item);
            let item = Vec::<C>::new();
            data.insert(item);
            let item = Vec::<D>::new();
            data.insert(item);
            let item = Vec::<E>::new();
            data.insert(item);
            let item = Vec::<F>::new();
            data.insert(item);
            let item = Vec::<G>::new();
            data.insert(item);
            let item = Vec::<H>::new();
            data.insert(item);
            Archetype { data, type_ids }
        }
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
            if true {
                if !(Self::type_ids() == archetype.type_ids) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: Self::type_ids() == archetype.type_ids",
                        )
                    }
                };
            };
            let (A, B, C, D, E, F, G, H) = self;
            archetype
                .data
                .get_mut_with_self::<Vec<A>>()
                .unwrap()
                .downcast_mut::<Vec<A>>()
                .unwrap()
                .push(A);
            archetype
                .data
                .get_mut_with_self::<Vec<B>>()
                .unwrap()
                .downcast_mut::<Vec<B>>()
                .unwrap()
                .push(B);
            archetype
                .data
                .get_mut_with_self::<Vec<C>>()
                .unwrap()
                .downcast_mut::<Vec<C>>()
                .unwrap()
                .push(C);
            archetype
                .data
                .get_mut_with_self::<Vec<D>>()
                .unwrap()
                .downcast_mut::<Vec<D>>()
                .unwrap()
                .push(D);
            archetype
                .data
                .get_mut_with_self::<Vec<E>>()
                .unwrap()
                .downcast_mut::<Vec<E>>()
                .unwrap()
                .push(E);
            archetype
                .data
                .get_mut_with_self::<Vec<F>>()
                .unwrap()
                .downcast_mut::<Vec<F>>()
                .unwrap()
                .push(F);
            archetype
                .data
                .get_mut_with_self::<Vec<G>>()
                .unwrap()
                .downcast_mut::<Vec<G>>()
                .unwrap()
                .push(G);
            archetype
                .data
                .get_mut_with_self::<Vec<H>>()
                .unwrap()
                .downcast_mut::<Vec<H>>()
                .unwrap()
                .push(H);
            Ok(())
        }
    }
    #[allow(non_snake_case)]
    impl<A: 'static, B: 'static, C: 'static, D: 'static, E: 'static, F: 'static, G: 'static> Bundle
        for (A, B, C, D, E, F, G)
    {
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A>(),
                TypeId::of::<B>(),
                TypeId::of::<C>(),
                TypeId::of::<D>(),
                TypeId::of::<E>(),
                TypeId::of::<F>(),
                TypeId::of::<G>(),
            ])
        }
        fn new_archetype() -> Archetype {
            let type_ids = Self::type_ids();
            let mut data = AnyMap::new();
            let item = Vec::<A>::new();
            data.insert(item);
            let item = Vec::<B>::new();
            data.insert(item);
            let item = Vec::<C>::new();
            data.insert(item);
            let item = Vec::<D>::new();
            data.insert(item);
            let item = Vec::<E>::new();
            data.insert(item);
            let item = Vec::<F>::new();
            data.insert(item);
            let item = Vec::<G>::new();
            data.insert(item);
            Archetype { data, type_ids }
        }
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
            if true {
                if !(Self::type_ids() == archetype.type_ids) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: Self::type_ids() == archetype.type_ids",
                        )
                    }
                };
            };
            let (A, B, C, D, E, F, G) = self;
            archetype
                .data
                .get_mut_with_self::<Vec<A>>()
                .unwrap()
                .downcast_mut::<Vec<A>>()
                .unwrap()
                .push(A);
            archetype
                .data
                .get_mut_with_self::<Vec<B>>()
                .unwrap()
                .downcast_mut::<Vec<B>>()
                .unwrap()
                .push(B);
            archetype
                .data
                .get_mut_with_self::<Vec<C>>()
                .unwrap()
                .downcast_mut::<Vec<C>>()
                .unwrap()
                .push(C);
            archetype
                .data
                .get_mut_with_self::<Vec<D>>()
                .unwrap()
                .downcast_mut::<Vec<D>>()
                .unwrap()
                .push(D);
            archetype
                .data
                .get_mut_with_self::<Vec<E>>()
                .unwrap()
                .downcast_mut::<Vec<E>>()
                .unwrap()
                .push(E);
            archetype
                .data
                .get_mut_with_self::<Vec<F>>()
                .unwrap()
                .downcast_mut::<Vec<F>>()
                .unwrap()
                .push(F);
            archetype
                .data
                .get_mut_with_self::<Vec<G>>()
                .unwrap()
                .downcast_mut::<Vec<G>>()
                .unwrap()
                .push(G);
            Ok(())
        }
    }
    #[allow(non_snake_case)]
    impl<A: 'static, B: 'static, C: 'static, D: 'static, E: 'static, F: 'static> Bundle
        for (A, B, C, D, E, F)
    {
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A>(),
                TypeId::of::<B>(),
                TypeId::of::<C>(),
                TypeId::of::<D>(),
                TypeId::of::<E>(),
                TypeId::of::<F>(),
            ])
        }
        fn new_archetype() -> Archetype {
            let type_ids = Self::type_ids();
            let mut data = AnyMap::new();
            let item = Vec::<A>::new();
            data.insert(item);
            let item = Vec::<B>::new();
            data.insert(item);
            let item = Vec::<C>::new();
            data.insert(item);
            let item = Vec::<D>::new();
            data.insert(item);
            let item = Vec::<E>::new();
            data.insert(item);
            let item = Vec::<F>::new();
            data.insert(item);
            Archetype { data, type_ids }
        }
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
            if true {
                if !(Self::type_ids() == archetype.type_ids) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: Self::type_ids() == archetype.type_ids",
                        )
                    }
                };
            };
            let (A, B, C, D, E, F) = self;
            archetype
                .data
                .get_mut_with_self::<Vec<A>>()
                .unwrap()
                .downcast_mut::<Vec<A>>()
                .unwrap()
                .push(A);
            archetype
                .data
                .get_mut_with_self::<Vec<B>>()
                .unwrap()
                .downcast_mut::<Vec<B>>()
                .unwrap()
                .push(B);
            archetype
                .data
                .get_mut_with_self::<Vec<C>>()
                .unwrap()
                .downcast_mut::<Vec<C>>()
                .unwrap()
                .push(C);
            archetype
                .data
                .get_mut_with_self::<Vec<D>>()
                .unwrap()
                .downcast_mut::<Vec<D>>()
                .unwrap()
                .push(D);
            archetype
                .data
                .get_mut_with_self::<Vec<E>>()
                .unwrap()
                .downcast_mut::<Vec<E>>()
                .unwrap()
                .push(E);
            archetype
                .data
                .get_mut_with_self::<Vec<F>>()
                .unwrap()
                .downcast_mut::<Vec<F>>()
                .unwrap()
                .push(F);
            Ok(())
        }
    }
    #[allow(non_snake_case)]
    impl<A: 'static, B: 'static, C: 'static, D: 'static, E: 'static> Bundle for (A, B, C, D, E) {
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A>(),
                TypeId::of::<B>(),
                TypeId::of::<C>(),
                TypeId::of::<D>(),
                TypeId::of::<E>(),
            ])
        }
        fn new_archetype() -> Archetype {
            let type_ids = Self::type_ids();
            let mut data = AnyMap::new();
            let item = Vec::<A>::new();
            data.insert(item);
            let item = Vec::<B>::new();
            data.insert(item);
            let item = Vec::<C>::new();
            data.insert(item);
            let item = Vec::<D>::new();
            data.insert(item);
            let item = Vec::<E>::new();
            data.insert(item);
            Archetype { data, type_ids }
        }
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
            if true {
                if !(Self::type_ids() == archetype.type_ids) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: Self::type_ids() == archetype.type_ids",
                        )
                    }
                };
            };
            let (A, B, C, D, E) = self;
            archetype
                .data
                .get_mut_with_self::<Vec<A>>()
                .unwrap()
                .downcast_mut::<Vec<A>>()
                .unwrap()
                .push(A);
            archetype
                .data
                .get_mut_with_self::<Vec<B>>()
                .unwrap()
                .downcast_mut::<Vec<B>>()
                .unwrap()
                .push(B);
            archetype
                .data
                .get_mut_with_self::<Vec<C>>()
                .unwrap()
                .downcast_mut::<Vec<C>>()
                .unwrap()
                .push(C);
            archetype
                .data
                .get_mut_with_self::<Vec<D>>()
                .unwrap()
                .downcast_mut::<Vec<D>>()
                .unwrap()
                .push(D);
            archetype
                .data
                .get_mut_with_self::<Vec<E>>()
                .unwrap()
                .downcast_mut::<Vec<E>>()
                .unwrap()
                .push(E);
            Ok(())
        }
    }
    #[allow(non_snake_case)]
    impl<A: 'static, B: 'static, C: 'static, D: 'static> Bundle for (A, B, C, D) {
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [
                TypeId::of::<A>(),
                TypeId::of::<B>(),
                TypeId::of::<C>(),
                TypeId::of::<D>(),
            ])
        }
        fn new_archetype() -> Archetype {
            let type_ids = Self::type_ids();
            let mut data = AnyMap::new();
            let item = Vec::<A>::new();
            data.insert(item);
            let item = Vec::<B>::new();
            data.insert(item);
            let item = Vec::<C>::new();
            data.insert(item);
            let item = Vec::<D>::new();
            data.insert(item);
            Archetype { data, type_ids }
        }
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
            if true {
                if !(Self::type_ids() == archetype.type_ids) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: Self::type_ids() == archetype.type_ids",
                        )
                    }
                };
            };
            let (A, B, C, D) = self;
            archetype
                .data
                .get_mut_with_self::<Vec<A>>()
                .unwrap()
                .downcast_mut::<Vec<A>>()
                .unwrap()
                .push(A);
            archetype
                .data
                .get_mut_with_self::<Vec<B>>()
                .unwrap()
                .downcast_mut::<Vec<B>>()
                .unwrap()
                .push(B);
            archetype
                .data
                .get_mut_with_self::<Vec<C>>()
                .unwrap()
                .downcast_mut::<Vec<C>>()
                .unwrap()
                .push(C);
            archetype
                .data
                .get_mut_with_self::<Vec<D>>()
                .unwrap()
                .downcast_mut::<Vec<D>>()
                .unwrap()
                .push(D);
            Ok(())
        }
    }
    #[allow(non_snake_case)]
    impl<A: 'static, B: 'static, C: 'static> Bundle for (A, B, C) {
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [TypeId::of::<A>(), TypeId::of::<B>(), TypeId::of::<C>()])
        }
        fn new_archetype() -> Archetype {
            let type_ids = Self::type_ids();
            let mut data = AnyMap::new();
            let item = Vec::<A>::new();
            data.insert(item);
            let item = Vec::<B>::new();
            data.insert(item);
            let item = Vec::<C>::new();
            data.insert(item);
            Archetype { data, type_ids }
        }
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
            if true {
                if !(Self::type_ids() == archetype.type_ids) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: Self::type_ids() == archetype.type_ids",
                        )
                    }
                };
            };
            let (A, B, C) = self;
            archetype
                .data
                .get_mut_with_self::<Vec<A>>()
                .unwrap()
                .downcast_mut::<Vec<A>>()
                .unwrap()
                .push(A);
            archetype
                .data
                .get_mut_with_self::<Vec<B>>()
                .unwrap()
                .downcast_mut::<Vec<B>>()
                .unwrap()
                .push(B);
            archetype
                .data
                .get_mut_with_self::<Vec<C>>()
                .unwrap()
                .downcast_mut::<Vec<C>>()
                .unwrap()
                .push(C);
            Ok(())
        }
    }
    #[allow(non_snake_case)]
    impl<A: 'static, B: 'static> Bundle for (A, B) {
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [TypeId::of::<A>(), TypeId::of::<B>()])
        }
        fn new_archetype() -> Archetype {
            let type_ids = Self::type_ids();
            let mut data = AnyMap::new();
            let item = Vec::<A>::new();
            data.insert(item);
            let item = Vec::<B>::new();
            data.insert(item);
            Archetype { data, type_ids }
        }
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
            if true {
                if !(Self::type_ids() == archetype.type_ids) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: Self::type_ids() == archetype.type_ids",
                        )
                    }
                };
            };
            let (A, B) = self;
            archetype
                .data
                .get_mut_with_self::<Vec<A>>()
                .unwrap()
                .downcast_mut::<Vec<A>>()
                .unwrap()
                .push(A);
            archetype
                .data
                .get_mut_with_self::<Vec<B>>()
                .unwrap()
                .downcast_mut::<Vec<B>>()
                .unwrap()
                .push(B);
            Ok(())
        }
    }
    #[allow(non_snake_case)]
    impl<A: 'static> Bundle for (A,) {
        fn type_ids() -> Vec<TypeId> {
            <[_]>::into_vec(box [TypeId::of::<A>()])
        }
        fn new_archetype() -> Archetype {
            let type_ids = Self::type_ids();
            let mut data = AnyMap::new();
            let item = Vec::<A>::new();
            data.insert(item);
            Archetype { data, type_ids }
        }
        fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
            if true {
                if !(Self::type_ids() == archetype.type_ids) {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: Self::type_ids() == archetype.type_ids",
                        )
                    }
                };
            };
            let (A,) = self;
            archetype
                .data
                .get_mut_with_self::<Vec<A>>()
                .unwrap()
                .downcast_mut::<Vec<A>>()
                .unwrap()
                .push(A);
            Ok(())
        }
    }
}
pub mod lifetime_anymap {
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
}
pub mod world {
    use super::anymap::{AnyMap, AnyMapBorrow, AnyMapBorrowMut};
    use super::archetype_iter::{Query, QueryInfos};
    use super::bundle::Bundle;
    use super::lifetime_anymap::{LifetimeAnyMap, LifetimeAnyMapBorrow, LifetimeAnyMapBorrowMut};
    use std::any::TypeId;
    use std::error::Error;
    pub struct Archetype {
        pub type_ids: Vec<TypeId>,
        pub data: AnyMap,
    }
    impl Archetype {
        pub fn new<T: Bundle>() -> Archetype {
            T::new_archetype()
        }
        pub fn add<T: Bundle>(&mut self, components: T) -> Result<(), Box<dyn Error>> {
            components.add_to_archetype(self)
        }
    }
    pub struct World {
        pub archetypes: Vec<Archetype>,
        owned_resources: AnyMap,
        cache: Vec<(Vec<TypeId>, usize)>,
    }
    impl World {
        pub fn new() -> Self {
            Self {
                archetypes: Vec::new(),
                owned_resources: AnyMap::new(),
                cache: Vec::with_capacity(8),
            }
        }
        pub fn query<T: QueryInfos>(&self) -> Query<T> {
            Query::<T>::new(self)
        }
        pub fn find_archetype<T: Bundle>(&mut self, type_ids: &[TypeId]) -> Option<usize> {
            if true {
                if !(T::type_ids() == type_ids) {
                    {
                        ::std::rt::begin_panic("assertion failed: T::type_ids() == type_ids")
                    }
                };
            };
            for (cached_type_id, archetype) in self.cache.iter() {
                if *cached_type_id == type_ids {
                    return Some(*archetype);
                }
            }
            let position = self
                .archetypes
                .iter()
                .position(|archetype| archetype.type_ids == type_ids);
            if let Some(position) = position {
                if self.cache.len() > 8 {
                    self.cache.pop();
                }
                self.cache.insert(0, (Vec::from(type_ids), position));
            }
            position
        }
        pub fn find_archetype_or_insert<T: Bundle>(&mut self, type_ids: &[TypeId]) -> usize {
            if true {
                if !(T::type_ids() == type_ids) {
                    {
                        ::std::rt::begin_panic("assertion failed: T::type_ids() == type_ids")
                    }
                };
            };
            self.find_archetype::<T>(type_ids).unwrap_or_else(|| {
                self.cache.clear();
                self.archetypes.push(T::new_archetype());
                self.archetypes.len() - 1
            })
        }
        pub fn query_archetypes<T: QueryInfos>(&self) -> impl Iterator<Item = usize> + '_ {
            self.archetypes
                .iter()
                .enumerate()
                .filter(|(_, archetype)| {
                    T::type_ids()
                        .iter()
                        .all(|id| archetype.type_ids.contains(id))
                })
                .map(|(n, _)| n)
        }
        pub fn spawn<T: Bundle>(&mut self, bundle: T) {
            let type_ids = T::type_ids();
            let archetype_idx = self.find_archetype_or_insert::<T>(&type_ids);
            self.archetypes
                .get_mut(archetype_idx)
                .unwrap()
                .add(bundle)
                .unwrap();
        }
        pub fn run(&mut self) -> RunWorldContext {
            RunWorldContext {
                world: self,
                temp_resources: LifetimeAnyMap::new(),
            }
        }
    }
    pub struct RunWorldContext<'run> {
        world: &'run mut World,
        temp_resources: LifetimeAnyMap<'run>,
    }
    impl<'run> RunWorldContext<'run> {
        pub fn insert_owned_resource<T: 'static>(&mut self, data: T) {
            self.world.owned_resources.insert(data);
        }
        pub fn get_owned_resource<'a, T: 'static>(
            &'a self,
        ) -> Result<AnyMapBorrow<'a, T>, Box<dyn Error + 'a>> {
            self.world.owned_resources.get()
        }
        pub fn get_owned_resource_mut<'a, T: 'static>(
            &'a self,
        ) -> Result<AnyMapBorrowMut<'a, T>, Box<dyn Error + 'a>> {
            self.world.owned_resources.get_mut()
        }
        pub fn insert_temp_resource<'a, T: 'static>(&'a mut self, resource: &'run mut T) {
            self.temp_resources.insert(resource);
        }
        pub fn get_temp_resource<'a, T: 'static>(
            &'a self,
        ) -> Result<LifetimeAnyMapBorrow<'a, T>, Box<dyn Error + 'a>> {
            self.temp_resources.get()
        }
        pub fn get_temp_resource_mut<'a, T: 'static>(
            &'a self,
        ) -> Result<LifetimeAnyMapBorrowMut<'a, T>, Box<dyn Error + 'a>> {
            self.temp_resources.get_mut()
        }
    }
}
