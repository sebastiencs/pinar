
use crate::prelude::*;
use crate::Result;

pub trait AsJs {
    type JsType: JsValue;
    fn as_js(self, _: &Env) -> Result<Self::JsType>;
}

impl<T> AsJs for T
where
    T: IntoJs
{
    type JsType = T::JsType;
    fn as_js(self, env: &Env) -> Result<Self::JsType> {
        self.into_js(env)
    }
}

impl AsJs for JsFunction {
    type JsType = Self;
    fn as_js(self, _: &Env) -> Result<Self::JsType> {
        Ok(self)
    }
}

impl AsJs for JsObject {
    type JsType = Self;
    fn as_js(self, _: &Env) -> Result<Self::JsType> {
        Ok(self)
    }
}
