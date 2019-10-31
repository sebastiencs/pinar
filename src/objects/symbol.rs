
use std::marker::PhantomData;
use super::*;

/// A Javascript symbol
pub struct JsSymbol<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}
