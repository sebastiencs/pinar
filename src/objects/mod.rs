
use std::ops::Deref;
use std::marker::PhantomData;
use napi_sys::*;
use crate::Result;
use crate::status::Status;
use crate::value::ValueType;
use crate::to_rust::ToRust;
use crate::error::JsAnyError;

mod array;
mod external;
mod function;
mod jsref;
mod number;
mod object;
mod string;
mod symbol;
mod value;

#[doc(hidden)]
pub use self::value::Value;

pub use self::{
    jsref::{
        JsRef,
        AsJsRef
    },
    array::{
        JsArray,
        JsArrayIterator
    },
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

pub enum JsAny<'e> {
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


impl<'e> std::fmt::Debug for JsAny<'e> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let inner = match self {
            JsAny::String(s) => {
                // let s = s.to_string().as_str();
                // &s
                &"String"
            }
            JsAny::Object(s) => { &"Object" }
            JsAny::Array(s) => { &"Array" }
            JsAny::Number(s) => { &"Number" }
            JsAny::Symbol(s) => { &"Symbol" }
            JsAny::External(s) => { &"External" }
            JsAny::Function(s) => { &"Function" }
            JsAny::Undefined(s) => { &"Undefined" }
            JsAny::Null(s) => { &"Null" }
            JsAny::Boolean(s) => { &"Boolean" }
            JsAny::BigInt(s) => { &"BigInt" }
        };
        f.debug_struct("JsAny")
         .field("inner", inner)
         .finish()
    }
}

macro_rules! impl_jsany {
    (
        RUST_TYPES:
        $( ($rfn_name:ident, $rtype:ident, $rany:ident) ),*,
        JS_TYPES:
        $( ($fn_name:ident, $jstype:ident, $any:ident) ),*,
    ) => {
        $(pub fn $fn_name(&self) -> Result<$jstype<'e>> {
            match self {
                JsAny::$any(s) => Ok(s.clone()),
                _ => Err(JsAnyError::WrongAny.into())
            }
        })*
        $(pub fn $rfn_name(&self) -> Result<$rtype> {
            match self {
                JsAny::$rany(s) => s.to_rust(),
                _ => Err(JsAnyError::WrongAny.into())
            }
        })*
    }
}

impl<'e> JsAny<'e> {
    #[inline]
    pub(crate) fn from(value: Value) -> Result<JsAny<'e>> {
        let value = match value.type_of()? {
            ValueType::Object => {
                match value.is_array()? {
                    true => JsAny::Array(JsArray::from(value)),
                    _ => JsAny::Object(JsObject::from(value))
                }
            },
            ValueType::String => JsAny::String(JsString::from(value)),
            ValueType::Number => JsAny::Number(JsNumber::from(value)),
            ValueType::External => JsAny::External(JsExternal::from(value)),
            ValueType::Symbol => JsAny::Symbol(JsSymbol::from(value)),
            ValueType::Undefined => JsAny::Undefined(JsUndefined::from(value)),
            ValueType::Function => JsAny::Function(JsFunction::from(value)),
            ValueType::Null => JsAny::Null(JsNull::from(value)),
            ValueType::Boolean => JsAny::Boolean(JsBoolean::from(value)),
            ValueType::Bigint => JsAny::BigInt(JsBigInt::from(value)),
        };
        Ok(value)
    }
    #[inline]
    pub(crate) fn clone(&self) -> JsAny<'e> {
        match self {
            JsAny::String(s) => JsAny::String(s.clone()),
            JsAny::Object(s) => JsAny::Object(s.clone()),
            JsAny::Array(s) => JsAny::Array(s.clone()),
            JsAny::Number(s) => JsAny::Number(s.clone()),
            JsAny::Symbol(s) => JsAny::Symbol(s.clone()),
            JsAny::External(e) => JsAny::External(e.clone()),
            JsAny::Function(e) => JsAny::Function(e.clone()),
            JsAny::Undefined(e) => JsAny::Undefined(e.clone()),
            JsAny::Null(e) => JsAny::Null(e.clone()),
            JsAny::Boolean(e) => JsAny::Boolean(e.clone()),
            JsAny::BigInt(e) => JsAny::BigInt(e.clone()),
        }
    }

    impl_jsany!(
        RUST_TYPES:
        (as_string, String, String),
        (as_number, i64, Number),
        (as_bool, bool, Boolean),
        JS_TYPES:
        (as_jsarray, JsArray, Array),
        (as_jsstring, JsString, String),
        (as_jsobject, JsObject, Object),
        (as_jsnumber, JsNumber, Number),
        (as_jssymbol, JsSymbol, Symbol),
        (as_jsexternal, JsExternal, External),
        (as_jsfunction, JsFunction, Function),
        (as_jsboolean, JsBoolean, Boolean),
        (as_jsbigint, JsBigInt, BigInt),
    );
}

pub struct JsThis<'e>(pub(crate) JsAny<'e>);

impl<'e> JsThis<'e> {
    pub fn get_any(&self) -> JsAny<'e> {
        self.0.clone()
    }
}

impl<'e> Deref for JsThis<'e> {
    type Target = JsAny<'e>;
    fn deref(&self) -> &JsAny<'e> {
        &self.0
    }
}

impl<'e> JsValue for JsThis<'e> {
    fn get_value(&self) -> Value {
        self.0.get_value()
    }
}

pub trait JsValue {
    fn get_value(&self) -> Value;
}

impl<'e> JsValue for JsAny<'e> {
    fn get_value(&self) -> Value {
        match self {
            JsAny::String(s) => s.value,
            JsAny::Object(s) => s.value,
            JsAny::Array(s) => s.value,
            JsAny::Number(s) => s.value,
            JsAny::Symbol(s) => s.value,
            JsAny::External(s) => s.value,
            JsAny::Function(s) => s.value,
            JsAny::Undefined(s) => s.value,
            JsAny::Null(s) => s.value,
            JsAny::Boolean(s) => s.value,
            JsAny::BigInt(s) => s.value,
        }
    }
}

impl<'e> JsValue for &JsAny<'e> {
    fn get_value(&self) -> Value {
        match self {
            JsAny::String(s) => s.value,
            JsAny::Object(s) => s.value,
            JsAny::Array(s) => s.value,
            JsAny::Number(s) => s.value,
            JsAny::Symbol(s) => s.value,
            JsAny::External(s) => s.value,
            JsAny::Function(s) => s.value,
            JsAny::Undefined(s) => s.value,
            JsAny::Null(s) => s.value,
            JsAny::Boolean(s) => s.value,
            JsAny::BigInt(s) => s.value,
        }
    }
}
