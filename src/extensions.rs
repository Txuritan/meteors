use std::any::{Any, TypeId};

use crate::utils::ArrayMap;

pub struct Extensions {
    inner: ArrayMap<TypeId, Box<dyn Any + Send + Sync>, 32>,
}

impl Extensions {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: ArrayMap::new(),
        }
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        fn downcast_owned<T: 'static>(boxed: Box<dyn Any + Send + Sync>) -> Option<T> {
            boxed.downcast().ok().map(|boxed| *boxed)
        }

        self.inner
            .insert(TypeId::of::<T>(), box val)
            .and_then(downcast_owned)
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        fn downcast_ref<T: 'static>(boxed: &Box<dyn Any + Send + Sync>) -> Option<&T> {
            boxed.downcast_ref()
        }

        self.inner.get(&TypeId::of::<T>()).and_then(downcast_ref)
    }
}

impl const Default for Extensions {
    fn default() -> Self {
        Self::new()
    }
}
