use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

pub struct PointerIdentityArc<T>(Arc<T>);

impl<T> PointerIdentityArc<T> {
    pub fn new(arc: Arc<T>) -> Self {
        Self(arc)
    }
}

impl<T> Clone for PointerIdentityArc<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Deref for PointerIdentityArc<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> PartialEq for PointerIdentityArc<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for PointerIdentityArc<T> {}

impl<T> Hash for PointerIdentityArc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
