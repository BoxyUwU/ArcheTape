use super::anymap::AnyMap;
use super::world::Archetype;
use std::any::TypeId;
use std::error::Error;

pub trait Bundle {
    fn type_ids() -> Vec<TypeId>;
    fn new_archetype() -> Archetype;
    fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>>;
}

macro_rules! impl_bundle {
    ($($x:ident) *) => {
        #[allow(non_snake_case)]
        impl<$($x: 'static),*> Bundle for ($($x,)*) {
            fn type_ids() -> Vec<TypeId> {
                vec![$(TypeId::of::<$x>(),)*]
            }

            fn new_archetype() -> Archetype {
                let type_ids = Self::type_ids();

                let mut data = AnyMap::new();
                $(
                    let item = Vec::<$x>::new();
                    data.insert(item);
                )*

                Archetype {
                    data,
                    type_ids,
                }
            }

            fn add_to_archetype(self, archetype: &mut Archetype) -> Result<(), Box<dyn Error>> {
                let ($($x,)*) = self;

                if Self::type_ids() != archetype.type_ids {
                    return Err("Components did not match archetype".into());
                }

                $(
                    archetype.data.get_mut::<Vec<$x>>().unwrap().push($x);
                )*

                Ok(())
            }
        }
    };
}

impl_bundle!(A B C D E F G H I J);
impl_bundle!(A B C D E F G H I);
impl_bundle!(A B C D E F G H);
impl_bundle!(A B C D E F G);
impl_bundle!(A B C D E F);
impl_bundle!(A B C D E);
impl_bundle!(A B C D);
impl_bundle!(A B C);
impl_bundle!(A B);
impl_bundle!(A);
