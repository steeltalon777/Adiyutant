use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Id<T> {
    value: Uuid,
    _marker: std::marker::PhantomData<T>,
}

// All manual impls to avoid conservative bounds from derive macros
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> Id<T> {
    pub fn new() -> Self {
        Self {
            value: Uuid::new_v4(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn value(&self) -> Uuid {
        self.value
    }

    /// Create an Id from an existing Uuid.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            value: uuid,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_new_creates_unique_ids() {
        let id1 = Id::<()>::new();
        let id2 = Id::<()>::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn id_value_returns_uuid() {
        let id = Id::<()>::new();
        let uuid = id.value();
        // Uuid v4 is 36 chars: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
        assert_eq!(uuid.to_string().len(), 36);
    }

    #[test]
    fn id_impl_copy() {
        let id1 = Id::<()>::new();
        let id2 = id1; // Copy
        assert_eq!(id1, id2);
    }

    #[test]
    fn id_type_parameter_creates_distinct_types() {
        struct A;
        struct B;
        let id_a = Id::<A>::new();
        let id_b = Id::<B>::new();
        // Type parameters ensure they are different types at compile time,
        // but values can still be compared through PartialEq
        assert_ne!(id_a.value(), id_b.value());
    }
}
