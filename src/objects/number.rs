
use std::marker::PhantomData;
use super::*;

pub struct JsNumber<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}
