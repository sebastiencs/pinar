
use std::marker::PhantomData;
use napi_sys::*;
use crate::Result;
use crate::status::Status;
use crate::value::ValueType;
use crate::to_rust::ToRust;

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

pub struct JsUndefined<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

pub struct JsNull<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

pub struct JsBoolean<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

pub struct JsBigInt<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}


//#[derive(Copy, Clone)]
pub enum JsUnknown<'e> {
    String(JsString<'e>),
    Object(JsObject<'e>),
    Array(JsArray<'e>),
    Number(JsNumber<'e>),
    Symbol(JsSymbol<'e>),
    External(JsExternal<'e>),
    Function(JsFunction<'e>),
    Undefined(JsUndefined<'e>),
    Null(JsNull<'e>),
    Boolean(JsBoolean<'e>),
    BigInt(JsBigInt<'e>),
}

impl<'e> JsUnknown<'e> {
    #[inline]
    pub(crate) fn from(value: Value) -> Result<JsUnknown<'e>> {
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
    pub(crate) fn clone(&self) -> JsUnknown<'e> {
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

    pub fn as_string(&self) -> Option<String> {
        match self {
            JsUnknown::String(s) => s.to_rust().ok(),
            _ => None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsUnknown::Boolean(s) => s.to_rust().ok(),
            _ => None
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
            impl<'e> JsRef<$jstype<'e>> {
                pub fn get(&self) -> Result<$jstype<'e>> {
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

            impl<'e> JsValue for $jstype<'e> {
                #[inline]
                fn get_value(&self) -> Value {
                    self.value
                }
            }

            impl<'e> $jstype<'e> {
                #[inline]
                pub(crate) fn from(value: Value) -> Self {
                    Self { value, phantom: PhantomData }
                }

                #[inline]
                pub(crate) fn clone(&self) -> Self {
                    Self { value: self.value, phantom: PhantomData }
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
