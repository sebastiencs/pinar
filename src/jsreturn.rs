use crate::JsUndefined;
use crate::JsValue;
use crate::Env;
use crate::error::Error;
use crate::ToJs;

pub trait JsReturn<'e> {
    type Value: JsValue;
    type Error: Into<Error>;
    fn get_result(self, env: Env) -> Result<Option<Self::Value>, Self::Error>;
}

impl<'e, T> JsReturn<'e> for T
where
    T: ToJs<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Self::Value>, Self::Error> {
        Ok(Some(self.to_js(&env)?))
    }
}

impl<'e, T> JsReturn<'e> for Option<T>
where
    T: JsReturn<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Self::Value>, Self::Error> {
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
    fn get_result(self, env: Env) -> Result<Option<Self::Value>, Self::Error> {
        self?.get_result(env).map_err(Into::into)
    }
}

impl<'e> JsReturn<'e> for () {
    type Value = JsUndefined<'e>;
    type Error = Error;
    fn get_result(self, _: Env) -> Result<Option<Self::Value>, Self::Error> {
        Ok(None)
    }
}
