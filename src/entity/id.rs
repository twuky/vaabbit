use std::any::TypeId;
use std::fmt::Debug;
use std::hash::Hash;

pub struct ID<T: ?Sized> {
    pub index: slotmap::DefaultKey,
    pub _type: std::marker::PhantomData<T>,
}

#[derive(Clone, Copy, Eq, Hash)]
pub struct TypedID {
    pub index: slotmap::DefaultKey,
    pub type_id: TypeId,
}

impl TypedID {
    pub fn from_id<T: 'static>(id: ID<T>) -> Self {
        Self {
            index: id.index,
            type_id: std::any::TypeId::of::<T>(),
        }
    }

    pub fn is<T: 'static>(&self) -> Option<ID<T>> {
        match self.type_id {
            id if id == std::any::TypeId::of::<T>() => {
                Some(ID::<T>::from(*self))
            }
            _ => None,
        }
    }
}

impl <T: 'static> From<ID<T>> for TypedID {
    fn from(id: ID<T>) -> Self {
        Self::from_id(id)
    }
}

impl Debug for TypedID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypedID<{:?}> index: {:?}", self.type_id, self.index)
    }
}

impl PartialEq for TypedID {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.type_id == other.type_id
    }
}

impl<T: 'static> ID<T> {
    pub fn new(index: slotmap::DefaultKey) -> Self {
        Self {
            index,
            _type: std::marker::PhantomData,
        }
    }

    pub fn from(id: TypedID) -> Self {
        Self {
            index: id.index,
            _type: std::marker::PhantomData,
        }
    }
    pub fn from_typed_id(id: TypedID) -> Self {
        Self {
            index: id.index,
            _type: std::marker::PhantomData,
        }
    }

    pub fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
    pub fn type_name(&self) -> &'static str {
        std::any::type_name::<T>().split("::").last().unwrap()
    }

    pub fn into_typed_id(self) -> TypedID {
        TypedID {
            index: self.index,
            type_id: TypeId::of::<T>(),
        }
    }
}

impl<T> Clone for ID<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _type: std::marker::PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = *source
    }
}

impl <T> Copy for ID<T> {}

impl <T> Eq for ID<T> {

}

impl <T> Hash for ID<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl <T> Debug for ID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID<{}>, index: {:?}", std::any::type_name::<T>(), self.index)
    }
}

impl <T> PartialEq for ID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index

    }
}