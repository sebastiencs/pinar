
use std::rc::Rc;
use std::sync::Arc;
use std::os::raw::c_char;
use napi_sys::*;
use crate::prelude::*;
use crate::Result;

pub trait ToRust<R> {
    fn to_rust(&self) -> Result<R>;
}

impl ToRust<String> for JsString
{
    fn to_rust(&self) -> Result<String> {
        let len = self.len()?;
        let mut buffer: Vec<u8> = Vec::with_capacity(len + 1); // + '\0'
        let mut written = 0usize;
        unsafe {
            Status::result(napi_get_value_string_utf8(
                self.value.env(),
                self.value.get(),
                buffer.as_mut_ptr() as *mut c_char,
                len + 1,
                &mut written as *mut usize
            ))?;
            buffer.set_len(written);
            // It's probably safe to assume that it's valid ut8
            Ok(String::from_utf8_unchecked(buffer))
        }
    }
}

impl ToRust<i64> for JsNumber {
    fn to_rust(&self) -> Result<i64> {
        let mut number = 0i64;
        unsafe {
            Status::result(napi_get_value_int64(
                self.value.env(),
                self.value.get(),
                &mut number as *mut i64
            ))?;
        }
        Ok(number)
    }
}

impl ToRust<i32> for JsNumber {
    fn to_rust(&self) -> Result<i32> {
        let mut number = 0i32;
        unsafe {
            Status::result(napi_get_value_int32(
                self.value.env(),
                self.value.get(),
                &mut number as *mut i32
            ))?;
        }
        Ok(number)
    }
}

impl ToRust<u32> for JsNumber {
    fn to_rust(&self) -> Result<u32> {
        let mut number = 0u32;
        unsafe {
            Status::result(napi_get_value_uint32(
                self.value.env(),
                self.value.get(),
                &mut number as *mut u32
            ))?;
        }
        Ok(number)
    }
}

impl ToRust<f64> for JsNumber {
    fn to_rust(&self) -> Result<f64> {
        let mut number = 0f64;
        unsafe {
            Status::result(napi_get_value_double(
                self.value.env(),
                self.value.get(),
                &mut number as *mut f64
            ))?;
        }
        Ok(number)
    }
}

impl ToRust<bool> for JsBoolean {
    fn to_rust(&self) -> Result<bool> {
        let mut result = false;
        unsafe {
            Status::result(napi_get_value_bool(
                self.value.env(),
                self.value.get(),
                &mut result as *mut bool
            ))?;
        }
        Ok(result)
    }
}

impl<T: 'static> ToRust<Arc<T>> for JsExternal
{
    fn to_rust(&self) -> Result<Arc<T>> {
        self.get_arc()
    }
}

impl<T: 'static> ToRust<Rc<T>> for JsExternal
{
    fn to_rust(&self) -> Result<Rc<T>> {
        self.get_rc()
    }
}

impl<T: 'static> ToRust<Box<T>> for JsExternal
{
    fn to_rust(&self) -> Result<Box<T>> {
        match self.take_box()? {
            Some(b) => Ok(b),
            _ => panic!("Extracting a box from a JsExternal that have
already been extracted. Use Option<Box<T>>")
        }
    }
}
