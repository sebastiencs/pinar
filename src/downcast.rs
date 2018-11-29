
use crate::{JsSymbol, JsString, JsObject, JsNumber, JsArray, JsHandle};
use crate::Value;
use crate::status::Status;
use crate::value::ValueType;

enum DowncastError {
    Napi(Status),
    FailToCast
}

trait Downcast<'e, T> {
    fn downcast(self) -> std::result::Result<Value<'e, T>, DowncastError>;
}

macro_rules! downcast_types {
    ( $(($type:ident, $value_type:ident)),* ) => {
        $(
            impl<'e> Downcast<'e, $type> for Value<'e, JsHandle> {
                fn downcast(self) -> std::result::Result<Value<'e, $type>, DowncastError> {
                    match self.type_of().map_err(DowncastError::Napi)? {
                        ValueType::$value_type => Ok(unsafe { std::mem::transmute(self) }),
                        _ => Err(DowncastError::FailToCast)
                    }
                }
            }
        )*
    }
}

downcast_types!(
    (JsString, String),
    (JsNumber, Number),
    (JsObject, Object),
    (JsSymbol, Symbol)
);

impl<'e> Downcast<'e, JsArray> for Value<'e, JsHandle> {
    fn downcast(self) -> std::result::Result<Value<'e, JsArray>, DowncastError> {
        match self.is_array().map_err(DowncastError::Napi)? {
            true => Ok(unsafe { std::mem::transmute(self) }),
            _ => Err(DowncastError::FailToCast)
        }
    }
}
