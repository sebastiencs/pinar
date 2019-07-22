
use std::ops::Deref;
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

macro_rules! impl_jsany {
    (
        RUST_TYPES:
        $( ($rfn_name:ident, $rtype:ident, $rany:ident) ),*,
        JS_TYPES:
        $( ($fn_name:ident, $jstype:ident, $any:ident) ),*,
    ) => {
        $(pub fn $fn_name(&self) -> Option<$jstype<'e>> {
            match self {
                JsAny::$any(s) => Some(s.clone()),
                _ => None
            }
        })*
        $(pub fn $rfn_name(&self) -> Option<$rtype> {
            match self {
                JsAny::$rany(s) => s.to_rust().ok(),
                _ => None
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
                            result.get_mut()
                        ))?;
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
                #[doc(hidden)]
                #[inline]
                pub fn from(value: Value) -> Self {
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
