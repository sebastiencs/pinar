
use std::marker::PhantomData;
use super::*;

/// A Javascript number
pub struct JsNumber<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}
