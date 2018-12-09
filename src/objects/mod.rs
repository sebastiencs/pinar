
use napi_sys::*;
use crate::Result;
use crate::status::Status;
use crate::value::ValueType;

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


//#[derive(Copy, Clone)]
pub enum JsUnknown {
    String(JsString),
    Object(JsObject),
    Array(JsArray),
    Number(JsNumber),
    Symbol(JsSymbol),
    External(JsExternal),
    Function(JsFunction),
    Undefined(JsUndefined),
    Null(JsNull),
    Boolean(JsBoolean),
    BigInt(JsBigInt),
}

impl JsUnknown {
    #[inline]
    pub(crate) fn from(value: Value) -> Result<JsUnknown> {
        let value = match value.type_of()? {
            ValueType::Object => {
                match value.is_array()? {
                    true => JsUnknown::Array(JsArray::from(value)),
                    _ => JsUnknown::Object(JsObject::from(value))
                }
            },
            ValueType::String => JsUnknown::String(JsString::from(value)),
            ValueType::Number => JsUnknown::Number(JsNumber::from(value)),
            ValueType::External => JsUnknown::External(JsExternal::from(value)),
            ValueType::Symbol => JsUnknown::Symbol(JsSymbol::from(value)),
            ValueType::Undefined => JsUnknown::Undefined(JsUndefined::from(value)),
            ValueType::Function => JsUnknown::Function(JsFunction::from(value)),
            ValueType::Null => JsUnknown::Null(JsNull::from(value)),
            ValueType::Boolean => JsUnknown::Boolean(JsBoolean::from(value)),
            ValueType::Bigint => JsUnknown::BigInt(JsBigInt::from(value)),
        };
        Ok(value)
    }
    #[inline]
    pub(crate) fn clone(&self) -> JsUnknown {
        match self {
            JsUnknown::String(s) => JsUnknown::String(s.clone()),
            JsUnknown::Object(s) => JsUnknown::Object(s.clone()),
            JsUnknown::Array(s) => JsUnknown::Array(s.clone()),
            JsUnknown::Number(s) => JsUnknown::Number(s.clone()),
            JsUnknown::Symbol(s) => JsUnknown::Symbol(s.clone()),
            JsUnknown::External(e) => JsUnknown::External(e.clone()),
            JsUnknown::Function(e) => JsUnknown::Function(e.clone()),
            JsUnknown::Undefined(e) => JsUnknown::Undefined(e.clone()),
            JsUnknown::Null(e) => JsUnknown::Null(e.clone()),
            JsUnknown::Boolean(e) => JsUnknown::Boolean(e.clone()),
            JsUnknown::BigInt(e) => JsUnknown::BigInt(e.clone()),
        }
    }
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
                pub fn get(&self) -> Result<$jstype> {
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
