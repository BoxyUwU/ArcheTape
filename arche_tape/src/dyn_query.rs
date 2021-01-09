use crate::EcsId;
use std::marker::PhantomData;

trait QueryInfo {
    fn name() -> &'static str;
}

impl QueryInfo for () {
    fn name() -> &'static str {
        "()"
    }
}
pub struct Immut;
impl QueryInfo for Immut {
    fn name() -> &'static str {
        "immut"
    }
}
pub struct Excl;
impl QueryInfo for Excl {
    fn name() -> &'static str {
        "mut"
    }
}

struct BuilderFns {
    immut: fn(*mut u8, EcsId) -> (*mut u8, BuilderFns, fn(*mut u8)),
    excl: fn(*mut u8, EcsId) -> (*mut u8, BuilderFns, fn(*mut u8)),
}

struct DynQueryBuilder {
    data: *mut u8,
    fns: BuilderFns,
    contents: fn(*mut u8),
}

impl DynQueryBuilder {
    fn new() -> Self {
        let (data, fns, contents) = RealQueryBuilder::new();
        Self {
            data,
            fns,
            contents,
        }
    }

    fn contents(&self) {
        (self.contents)(self.data);
    }

    fn with(&mut self, id: EcsId) {
        let (data, fns, contents) = (self.fns.immut)(self.data, id);
        *self = Self {
            data,
            fns,
            contents,
        }
    }

    fn with_mut(&mut self, id: EcsId) {
        let (data, fns, contents) = (self.fns.excl)(self.data, id);
        *self = Self {
            data,
            fns,
            contents,
        }
    }
}

struct RealQueryBuilder<T, const IDS: usize> {
    phantom: PhantomData<T>,
    ids: [EcsId; IDS],
}

macro_rules! impl_query_builder {
    ($($T:ident)* $N1:literal $N2:literal) => {
        impl<$($T: QueryInfo,)*> RealQueryBuilder<($($T,)*), $N1> {
            fn contents(this: *mut u8) {
                //let this: Box<Self> = unsafe { Box::from_raw(this as _) };
                //let mut n = 0;
                //$({
                //    n += 1;
                //    dbg!(this.ids[n - 1], $T::name());
                //})*
            }

            fn immut(this: *mut u8, id: EcsId) -> (*mut u8, BuilderFns, fn(*mut u8)) {
                let this: Self = *unsafe { Box::from_raw(this as _) };
                let new_ids = push_id(&this.ids, id);

                let builder_fns = BuilderFns {
                    immut: RealQueryBuilder::<(Immut, $($T,)*), $N2>::immut,
                    excl: RealQueryBuilder::<(Excl, $($T,)*), $N2>::excl,
                };

                (Box::into_raw(Box::new(
                    RealQueryBuilder::<(Immut, $($T,)*), $N2> {
                        phantom: PhantomData,
                        ids: new_ids,
                    }
                )) as *mut u8, builder_fns, RealQueryBuilder::<(Immut, $($T,)*), $N2>::contents)
            }

            fn excl(this: *mut u8, id: EcsId) -> (*mut u8, BuilderFns, fn(*mut u8)) {
                let this: Self = *unsafe { Box::from_raw(this as _) };
                let new_ids = push_id(&this.ids, id);

                let builder_fns = BuilderFns {
                    immut: RealQueryBuilder::<(Immut, $($T,)*), $N2>::immut,
                    excl: RealQueryBuilder::<(Excl, $($T,)*), $N2>::excl,
                };

                (Box::into_raw(Box::new(
                    RealQueryBuilder::<(Excl, $($T,)*), $N2> {
                        phantom: PhantomData,
                        ids: new_ids,
                    }
                )) as *mut u8, builder_fns, RealQueryBuilder::<(Excl, $($T,)*), $N2>::contents)
            }
        }
    };
}

#[inline(never)]
fn push_id<const N: usize>(old_ids: &[EcsId], id: EcsId) -> [EcsId; N] {
    let mut vec = Vec::with_capacity(old_ids.len());
    vec.copy_from_slice(&old_ids);
    vec.push(id);

    use std::convert::TryFrom;
    <_>::try_from(&*vec).unwrap()
}

macro_rules! impl_final_query_builder {
    ($($T:ident)* $N1:literal $N2:literal) => {
        impl<$($T: QueryInfo,)*> RealQueryBuilder<($($T,)*), $N1> {
            #[allow(unused)]
            fn immut(_: *mut u8, _: EcsId) -> (*mut u8, BuilderFns, fn(*mut u8)) {
                panic!("Querying for this many EcsId's is not supported")
            }

            #[allow(unused)]
            fn excl(_: *mut u8, _: EcsId) -> (*mut u8, BuilderFns, fn(*mut u8)) {
                panic!("Querying for this many EcsId's is not supported")
            }

            #[allow(unused)]
            fn contents(_: *mut u8) {
                panic!("Querying for this many EcsId's is not supported")
            }
        }
    };
}

impl RealQueryBuilder<(), 0> {
    fn new() -> (*mut u8, BuilderFns, fn(*mut u8)) {
        let data = Box::into_raw(Box::new(Self {
            phantom: PhantomData,
            ids: [],
        })) as *mut u8;
        let fns = BuilderFns {
            immut: Self::immut,
            excl: Self::excl,
        };

        (data, fns, Self::contents)
    }
}

impl_final_query_builder!(A B C D E F G H I J K 11 12);
impl_query_builder!(A B C D E F G H I J 10 11);
impl_query_builder!(A B C D E F G H I 9 10);
impl_query_builder!(A B C D E F G H 8 9);
impl_query_builder!(A B C D E F G 7 8);
impl_query_builder!(A B C D E F 6 7);
impl_query_builder!(A B C D E 5 6);
impl_query_builder!(A B C D 4 5);
impl_query_builder!(A B C 3 4);
impl_query_builder!(A B 2 3);
impl_query_builder!(A 1 2);
impl_query_builder!(0 1);

#[cfg(test)]
mod test {
    use super::DynQueryBuilder;
    use crate::EcsId;

    #[test]
    fn new() {
        let mut dyn_builder = DynQueryBuilder::new();
        dyn_builder.with_mut(EcsId::new(0, 0));
        dyn_builder.contents();
        panic!()
    }
}
