
use std::ops::Deref;
use std::marker::PhantomData;

use crate::JsResult;
use crate::status::Status;
use crate::value::ValueType;
use crate::to_rust::ToRust;
use crate::error::JsAnyError;
use crate::env::Env;

mod array;
mod external;
mod function;
mod jsref;
mod number;
mod object;
mod string;
mod symbol;
mod value;
mod function_threadsafe;

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
    function_threadsafe::JsFunctionThreadSafe,
    number::JsNumber,
    object::{
        JsObject,
        KeyProperty,
        OwnProperty,
    },
    string::JsString,
    symbol::JsSymbol,
    value::Value,
};

/// A Javascript undefined.
pub struct JsUndefined<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

/// A Javascript null.
pub struct JsNull<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

/// A Javascript boolean.
pub struct JsBoolean<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

/// A Javascript BigInt.
pub struct JsBigInt<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

/// Enum representing any kind of Javascript value
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
            JsAny::String(_) => {
                // let s = s.to_string().as_str();
                // &s
                "String"
            }
            JsAny::Object(_) => { "Object" }
            JsAny::Array(_) => { "Array" }
            JsAny::Number(_) => { "Number" }
            JsAny::Symbol(_) => { "Symbol" }
            JsAny::External(_) => { "External" }
            JsAny::Function(_) => { "Function" }
            JsAny::Undefined(_) => { "Undefined" }
            JsAny::Null(_) => { "Null" }
            JsAny::Boolean(_) => { "Boolean" }
            JsAny::BigInt(_) => { "BigInt" }
        };
        f.debug_struct("JsAny")
         .field("inner", &inner)
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
        $(pub fn $fn_name(&self) -> JsResult<$jstype<'e>> {
            match self {
                JsAny::$any(s) => Ok(s.clone()),
                _ => Err(JsAnyError::WrongAny.into())
            }
        })*
        $(pub fn $rfn_name(&self) -> JsResult<$rtype> {
            match self {
                JsAny::$rany(s) => s.to_rust(),
                _ => Err(JsAnyError::WrongAny.into())
            }
        })*
    }
}

impl<'e> JsAny<'e> {
    #[inline]
    pub(crate) fn from(value: Value) -> JsResult<JsAny<'e>> {
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

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn env(&self) -> Env {
        match self {
            JsAny::String(s) => s.value.env,
            JsAny::Object(s) => s.value.env,
            JsAny::Array(s) => s.value.env,
            JsAny::Number(s) => s.value.env,
            JsAny::Symbol(s) => s.value.env,
            JsAny::External(s) => s.value.env,
            JsAny::Function(s) => s.value.env,
            JsAny::Undefined(s) => s.value.env,
            JsAny::Null(s) => s.value.env,
            JsAny::Boolean(s) => s.value.env,
            JsAny::BigInt(s) => s.value.env,
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

/// A Javascript value representing the `this` on a function call.
///
/// `JsThis` dereferences to a [`JsAny`]
///
/// # Example
/// ```
/// #[pinar]
/// fn my_func(this: JsThis) -> JsResult<()> {
///     let this: JsObject = this.as_js_object()?;
///     Ok(())
/// }
/// ```
/// [`JsAny`]: ./enum.JsAny.html
pub struct JsThis<'e>(pub(crate) JsAny<'e>);

impl<'e> JsThis<'e> {
    /// Returns the inner `JsAny`
    pub fn get_any(self) -> JsAny<'e> {
        self.0
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

/// Helper trait to get the inner [`Value`]
///
/// [`Value`]: ./struct.Value.html
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
