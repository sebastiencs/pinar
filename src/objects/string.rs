
use std::marker::PhantomData;
use std::os::raw::c_char;
use napi_sys::*;
use crate::prelude::*;
use crate::Result;

pub struct JsString<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

impl<'e> JsString<'e> {
    /// Returns the string length
    pub fn len(&self) -> Result<usize> {
        unsafe {
            let mut length = 0;
            Status::result(napi_get_value_string_utf8(
                self.value.env(),
                self.value.get(),
                std::ptr::null_mut() as *mut c_char,
                0,
                &mut length as *mut usize
            ))?;
            Ok(length)
        }
    }
}

impl<'e> std::fmt::Display for JsString<'e> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.to_rust().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", s)
    }
}
