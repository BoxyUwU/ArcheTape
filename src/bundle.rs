use super::entities::Entity;
use super::world::Archetype;
use std::any::TypeId;

pub trait Bundle {
    fn arity() -> usize;
    fn type_ids() -> Vec<TypeId>;
    fn new_archetype() -> Archetype;
    fn add_to_archetype(self, archetype: &mut Archetype, entity: Entity);
}

macro_rules! impl_bundle {
    ($($x:ident) *) => {
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        impl<$($x: 'static),*> Bundle for ($($x,)*) {
            fn arity() -> usize {
                let mut n = 0;
                $(
                    let $x = (); n += 1;
                )*
                n
            }

            fn type_ids() -> Vec<TypeId> {
                vec![$(TypeId::of::<$x>(),)*]
            }

            fn new_archetype() -> Archetype {
                let type_ids = Self::type_ids();

                use std::sync::RwLock;
                let component_storages = vec![$(RwLock::new(crate::untyped_vec::UntypedVec::new::<$x>()),)*];

                use std::collections::HashMap;
                let mut hashmap = HashMap::with_hasher(crate::anymap::TypeIdHasherBuilder());

                let mut _n = 0;
                $(
                    hashmap.insert(TypeId::of::<$x>(), _n);
                    _n += 1;
                )*

                Archetype {
                    lookup: hashmap,
                    component_storages,
                    entities: Vec::new(),
                    type_ids,
                }
            }

            fn add_to_archetype(self, archetype: &mut Archetype, entity: Entity) {
                assert!(Self::arity() == archetype.component_storages.len());

                let ($($x,)*) = self;

                archetype.entities.push(entity);
                $(
                    let id = TypeId::of::<$x>();
                    let idx = archetype.lookup[&id];
                    let storage = &mut archetype.component_storages[idx];
                    let lock = storage.get_mut().expect("Bundle-Archetype mismatch. Attempted to add entity to archetype with wrong components");
                    lock.push($x);
                )*
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
