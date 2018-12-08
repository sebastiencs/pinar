
use napi_sys::*;
use crate::Result;
use crate::status::Status;

mod array;
mod external;
mod function;
mod jsref;
mod number;
mod object;
mod string;
mod symbol;
mod value;

pub use self::{
    value::Value,
    jsref::{JsRef, AsJsRef},
    array::JsArray,
    external::JsExternal,
    function::JsFunction,
    number::JsNumber,
    object::{
        JsObject,
        KeyProperty
    },
    string::JsString,
    symbol::JsSymbol,
};

pub struct JsUndefined {
    pub(crate) value: Value
}

pub struct JsNull {
    pub(crate) value: Value
}

pub struct JsBoolean {
    pub(crate) value: Value
}

pub struct JsBigInt {
    pub(crate) value: Value
}

pub trait JsValue {
    fn get_value(&self) -> Value;
}

macro_rules! impl_jsref {
    (
        $( $jstype:ident ),*
    ) => {
        $(
            impl JsRef<$jstype> {
                fn get(&self) -> Result<$jstype> {
                    let mut result = Value::new(self.env);
                    unsafe {
                        Status::result(napi_get_reference_value(
                            self.env.env(),
                            self.js_ref,
                            result.get_mut())
                        )?;
                    }
                    Ok($jstype::from(result))
                }
            }

            impl JsValue for $jstype {
                #[inline]
                fn get_value(&self) -> Value {
                    self.value
                }
            }

            impl $jstype {
                #[inline]
                pub(crate) fn from(value: Value) -> Self {
                    Self { value }
                }

                #[inline]
                pub(crate) fn clone(&self) -> Self {
                    Self { value: self.value }
                }
            }

        )*
    }
}

impl_jsref!(
    JsString,
    JsObject,
    JsArray,
    JsNumber,
    JsSymbol,
    JsUndefined,
    JsFunction,
    JsExternal,
    JsNull,
    JsBoolean,
    JsBigInt
);
