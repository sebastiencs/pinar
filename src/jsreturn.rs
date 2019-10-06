use crate::JsUndefined;
use crate::JsValue;
use crate::Env;
use crate::error::Error;
use crate::ToJs;
use crate::Value;

/// JsReturn

pub trait JsReturn<'e> {
    type Value: JsValue;
    type Error: Into<Error>;
    fn get_result(self, env: Env) -> Result<Option<Value>, Self::Error>;
}

impl<'e, T> JsReturn<'e> for T
where
    T: ToJs<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Value>, Self::Error> {
        Ok(Some(self.to_js(env)?.get_value()))
    }
}

impl<'e, T> JsReturn<'e> for Option<T>
where
    T: JsReturn<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Value>, Self::Error> {
        match self {
            Some(v) => v.get_result(env).map_err(Into::into),
            None => Ok(None)
        }
    }
}

impl<'e, T> JsReturn<'e> for crate::Result<T>
where
    T: JsReturn<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Value>, Self::Error> {
        self?.get_result(env).map_err(Into::into)
    }
}

impl<'e> JsReturn<'e> for () {
    type Value = JsUndefined<'e>;
    type Error = Error;
    fn get_result(self, _: Env) -> Result<Option<Value>, Self::Error> {
        Ok(None)
    }
}

use crate::classes::JsClass;
use crate::classes::AsJsClass;

impl<'e, C> JsReturn<'e> for AsJsClass<C>
where
    C: JsClass
{
    type Value = Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Value>, Self::Error> {
        self.to_js_class(env).map(Some)
    }
}


/// JsReturnRef

pub trait JsReturnRef<'e> {
    type Value: JsValue;
    type Error: Into<Error>;
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Self::Error>;
}

impl<'e, T> JsReturnRef<'e> for T
where
    T: ToJs<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Self::Error> {
        Ok(Some(self.to_js(env)?.get_value()))
    }
}

impl<'e, T> JsReturnRef<'e> for Option<&T>
where
    T: JsReturnRef<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Self::Error> {
        match self {
            Some(v) => v.get_result_from_ref(env).map_err(Into::into),
            None => Ok(None)
        }
    }
}

impl<'e> JsReturnRef<'e> for ()
{
    type Value = Value;
    type Error = Error;
    fn get_result_from_ref(&self, _env: Env) -> Result<Option<Value>, Self::Error> {
        Ok(None)
    }
}
