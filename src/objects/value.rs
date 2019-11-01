
use crate::objects::JsValue;
use crate::value::ValueType;
use napi_sys::*;

use crate::env::Env;
use crate::JsResult;
use crate::status::Status;

/// Opaque structure containing the n-api raw value.
///
/// Users of Pinar shouldn't have to use this struct.
#[derive(Copy, Clone)]
pub struct Value {
    pub(crate) env: Env,
    pub(crate) value: napi_value
}

impl JsValue for Value {
    #[inline]
    fn get_value(&self) -> Value {
        *self
    }
}

impl Value {
    /// Returns a new, empty `Value`
    pub(crate) fn new(env: Env) -> Value {
        Value { env, value: std::ptr::null_mut() }
    }

    /// Returns a `Value` from the raw n-api value
    pub(crate) fn from(env: Env, value: napi_value) -> Value {
        Value { env, value }
    }

    /// Return the env
    pub(crate) fn env(&self) -> napi_env {
        self.env.env()
    }

    /// Returns the n-api value as mutable
    pub(crate) fn get_mut(&mut self) -> *mut napi_value {
        &mut self.value as *mut napi_value
    }

    /// Returns the n-api value
    pub(crate) fn get(&self) -> napi_value {
        self.value
    }

    /// Returns the type of the value
    pub(crate) fn type_of(&self) -> JsResult<ValueType> {
        let mut result: napi_valuetype = unsafe { std::mem::zeroed() };
        napi_call!(napi_typeof(
            self.env.env(),
            self.value,
            &mut result as *mut napi_valuetype
        ))?;
        Ok(ValueType::from(result))
    }

    /// Checks if the value is an array
    pub(crate) fn is_array(&self) -> JsResult<bool> {
        let mut result: bool = false;
        napi_call!(napi_is_array(
            self.env.env(),
            self.value,
            &mut result as *mut bool
        ))?;
        Ok(result)
    }
}
