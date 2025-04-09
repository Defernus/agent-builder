use eyre::ContextCompat;
use std::any::{Any, TypeId};

pub trait ValueTrait: Any + Send + Sync + 'static {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn as_any(&self) -> &dyn Any;

    fn dyn_clone(&self) -> Box<dyn ValueTrait>;

    fn get_type(&self) -> ValueType {
        ValueType {
            type_id: self.as_any().type_id(),
            type_name: self.type_name(),
        }
    }
}

impl<T: Any + Send + Sync + 'static + Clone> ValueTrait for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_clone(&self) -> Box<dyn ValueTrait> {
        Box::new(self.clone())
    }
}

pub struct Value {
    value: Box<dyn ValueTrait>,
}

impl Clone for Value {
    fn clone(&self) -> Self {
        Self {
            value: self.value.dyn_clone(),
        }
    }
}

impl Value {
    pub fn new<T: ValueTrait>(value: T) -> Self {
        Self {
            value: Box::new(value),
        }
    }

    pub fn try_downcast<T: 'static>(&self) -> Option<&T> {
        self.value.as_any().downcast_ref::<T>()
    }

    #[tracing::instrument(
        skip_all,
        fields(value_type = ?self.value.type_name(), expected_type = ?std::any::type_name::<T>())
    )]
    pub fn downcast<T: 'static>(&self) -> eyre::Result<&T> {
        self.try_downcast().context("Value type mismatch")
    }
}

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub struct ValueType {
    pub type_id: TypeId,
    pub type_name: &'static str,
}

impl ValueType {
    pub fn new<T: Any>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        }
    }
}
