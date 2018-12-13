
use std::marker::PhantomData;
use super::*;

pub struct JsSymbol<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}
