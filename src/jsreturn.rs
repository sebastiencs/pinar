use crate::JsUndefined;
use crate::JsValue;
use crate::Env;
use crate::error::Error;
use crate::ToJs;

pub trait JsReturn {
    type Value: JsValue;
    type Error: Into<Error>;
    fn get_result(self, env: Env) -> Result<Option<Self::Value>, Self::Error>;
}

impl<T> JsReturn for T
where
    T: ToJs
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Self::Value>, Self::Error> {
        Ok(Some(self.to_js(&env)?))
    }
}

impl<T> JsReturn for Option<T>
where
    T: JsReturn
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Self::Value>, Self::Error> {
        match self {
            Some(v) => v.get_result(env).map_err(|e| e.into()),
            None => Ok(None)
        }
    }
}

impl<T> JsReturn for crate::Result<T>
where
    T: JsReturn
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Self::Value>, Self::Error> {
        self?.get_result(env).map_err(|e| e.into())
    }
}

impl JsReturn for () {
    type Value = JsUndefined;
    type Error = Error;
    fn get_result(self, _: Env) -> Result<Option<Self::Value>, Self::Error> {
        Ok(None)
    }
}
