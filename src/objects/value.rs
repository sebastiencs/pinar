
use crate::objects::JsValue;
use crate::value::ValueType;
use napi_sys::*;

use crate::env::Env;
use crate::Result;
use crate::status::Status;

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

    pub(crate) fn new(env: Env) -> Value {
        Value { env, value: std::ptr::null_mut() }
    }

    pub(crate) fn from(env: Env, value: napi_value) -> Value {
        Value { env, value }
    }

    pub(crate) fn env(&self) -> napi_env {
        self.env.env()
    }

    pub(crate) fn get_mut(&mut self) -> *mut napi_value {
        &mut self.value as *mut napi_value
    }

    pub(crate) fn get(&self) -> napi_value {
        self.value
    }

    pub(crate) fn type_of(&self) -> Result<ValueType> {
        unsafe {
            let mut result: napi_valuetype = std::mem::zeroed();
            Status::result(napi_typeof(
                self.env.env(),
                self.value,
                &mut result as *mut napi_valuetype
            ))?;
            Ok(ValueType::from(result))
        }
    }

    pub(crate) fn is_array(&self) -> Result<bool> {
        unsafe {
            let mut result: bool = false;
            Status::result(napi_is_array(
                self.env.env(),
                self.value,
                &mut result as *mut bool
            ))?;
            Ok(result)
        }
    }
}
