
use std::rc::Rc;
use std::sync::Arc;
use std::os::raw::c_char;
use napi_sys::*;
use std::path::PathBuf;
use crate::prelude::*;
use crate::Result;

pub trait ToRust<R> {
    fn to_rust(&self) -> Result<R>;
}

impl<'e> ToRust<String> for JsString<'e>
{
    fn to_rust(&self) -> Result<String> {
        let len = self.len()?;
        let mut buffer: Vec<u8> = Vec::with_capacity(len + 1); // + '\0'
        let mut written = 0usize;

        napi_call!(napi_get_value_string_utf8(
            self.value.env(),
            self.value.get(),
            buffer.as_mut_ptr() as *mut c_char,
            len + 1,
            &mut written as *mut usize
        ))?;

        unsafe {
            buffer.set_len(written);
            // It's probably safe to assume that it's valid ut8
            Ok(String::from_utf8_unchecked(buffer))
        }
    }
}

impl<'e> ToRust<PathBuf> for JsString<'e>
{
    fn to_rust(&self) -> Result<PathBuf> {
        let result: Result<String> = self.to_rust();
        result.map(PathBuf::from)
    }
}

use crate::error::ArgumentsError;

impl<'e> ToRust<()> for JsString<'e>
{
    fn to_rust(&self) -> Result<()> {
        Err(ArgumentsError::wrong_type("a", 1))
    }
}

impl<'e> ToRust<i64> for JsNumber<'e> {
    fn to_rust(&self) -> Result<i64> {
        let mut number = 0i64;

        napi_call!(napi_get_value_int64(
            self.value.env(),
            self.value.get(),
            &mut number as *mut i64
        ))?;

        Ok(number)
    }
}

impl<'e> ToRust<i32> for JsNumber<'e> {
    fn to_rust(&self) -> Result<i32> {
        let mut number = 0i32;

        napi_call!(napi_get_value_int32(
            self.value.env(),
            self.value.get(),
            &mut number as *mut i32
        ))?;

        Ok(number)
    }
}

impl<'e> ToRust<u32> for JsNumber<'e> {
    fn to_rust(&self) -> Result<u32> {
        let mut number = 0u32;

        napi_call!(napi_get_value_uint32(
            self.value.env(),
            self.value.get(),
            &mut number as *mut u32
        ))?;

        Ok(number)
    }
}

impl<'e> ToRust<f64> for JsNumber<'e> {
    fn to_rust(&self) -> Result<f64> {
        let mut number = 0f64;

        napi_call!(napi_get_value_double(
            self.value.env(),
            self.value.get(),
            &mut number as *mut f64
        ))?;

        Ok(number)
    }
}

impl<'e> ToRust<bool> for JsBoolean<'e> {
    fn to_rust(&self) -> Result<bool> {
        let mut result = false;

        napi_call!(napi_get_value_bool(
            self.value.env(),
            self.value.get(),
            &mut result as *mut bool
        ))?;

        Ok(result)
    }
}

impl<'e, T: 'static> ToRust<Arc<T>> for JsExternal<'e>
{
    fn to_rust(&self) -> Result<Arc<T>> {
        self.get_arc()
    }
}

impl<'e, T: 'static> ToRust<Rc<T>> for JsExternal<'e>
{
    fn to_rust(&self) -> Result<Rc<T>> {
        self.get_rc()
    }
}

impl<'e, T: 'static> ToRust<Box<T>> for JsExternal<'e>
{
    fn to_rust(&self) -> Result<Box<T>> {
        match self.take_box()? {
            Some(b) => Ok(b),
            _ => panic!("Extracting a box from a JsExternal that have
already been extracted. Use Option<Box<T>>")
        }
    }
}

impl<'e, T: 'static> ToRust<Option<Box<T>>> for JsExternal<'e> {
    fn to_rust(&self) -> Result<Option<Box<T>>> {
        self.take_box()
    }
}

use serde::de::DeserializeOwned;

impl<'e, T> ToRust<Vec<T>> for JsArray<'e>
where
    T: DeserializeOwned
{
    fn to_rust(&self) -> Result<Vec<T>> {
        let env = self.value.env;
        let mut vec = Vec::with_capacity(self.len()?);
        for elem in self.iter()? {
            let elem: Result<_> = pinar_serde::de::from_any::<T>(env, elem).map_err(|e| {
                ArgumentsError::Deserialization(format!("{}", e)).into()
            });
            vec.push(elem?);
        }
        Ok(vec)
    }
}
