
use std::rc::Rc;
use std::sync::Arc;
use std::os::raw::c_char;
use napi_sys::*;
use std::path::PathBuf;
use crate::prelude::*;

/// Trait to convert a Javascript value to Rust
///
/// Is it implemented for basic Rust types and users of Pinar
/// can use the derive macro [`Pinar`] to implement it.
///
/// # Example
/// ```
/// #[derive(Serialize, Deserialize, Pinar)]
/// struct MyStruct {
///     s: String,
///     n: i64
/// }
/// ```
/// [`Pinar`]: ./derive.Pinar.html
pub trait ToRust<R> {
    fn to_rust(&self) -> JsResult<R>;
}

impl<'e> ToRust<String> for JsString<'e>
{
    fn to_rust(&self) -> JsResult<String> {
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
    fn to_rust(&self) -> JsResult<PathBuf> {
        let result: JsResult<String> = self.to_rust();
        result.map(PathBuf::from)
    }
}

impl<'e> ToRust<i64> for JsNumber<'e> {
    fn to_rust(&self) -> JsResult<i64> {
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
    fn to_rust(&self) -> JsResult<i32> {
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
    fn to_rust(&self) -> JsResult<u32> {
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
    fn to_rust(&self) -> JsResult<f64> {
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
    fn to_rust(&self) -> JsResult<bool> {
        let mut result = false;

        napi_call!(napi_get_value_bool(
            self.value.env(),
            self.value.get(),
            &mut result as *mut bool
        ))?;

        Ok(result)
    }
}


#[cfg(feature = "json")]
impl<'e> ToRust<serde_json::Value> for JsAny<'e> {
    fn to_rust(&self) -> JsResult<serde_json::Value> {
        crate::pinar_serde::de::from_any(self.env(), self.clone())
            .map_err(|e| e.into())
    }
}

// impl<'e, T> ToRust<T> for JsAny<'e>
// where
//     JsString<'e>: ToRust<T>,
//     JsObject<'e>: ToRust<T>,
//     JsArray<'e>: ToRust<T>,
//     JsNumber<'e>: ToRust<T>,
//     JsSymbol<'e>: ToRust<T>,
//     JsExternal<'e>: ToRust<T>,
//     JsFunction<'e>: ToRust<T>,
//     JsUndefined<'e>: ToRust<T>,
//     JsNull<'e>: ToRust<T>,
//     JsBoolean<'e>: ToRust<T>,
//     JsBigInt<'e>: ToRust<T>,
// {
//     fn to_rust(&self) -> JsResult<T> {
//         match self {
//             JsAny::String(s) => s.to_rust(),
//             JsAny::Object(s) => s.to_rust(),
//             JsAny::Array(s) => s.to_rust(),
//             JsAny::Number(s) => s.to_rust(),
//             JsAny::Symbol(s) => s.to_rust(),
//             JsAny::External(s) => s.to_rust(),
//             JsAny::Function(s) => s.to_rust(),
//             JsAny::Undefined(s) => s.to_rust(),
//             JsAny::Null(s) => s.to_rust(),
//             JsAny::Boolean(s) => s.to_rust(),
//             JsAny::BigInt(s) => s.to_rust(),
//         }
//     }    
// }

// use crate::error::JsAnyError::WrongAny;

// impl<'e, T> ToRust<T> for JsAny<'e>
// where
//     JsString<'e>: ToRust<T>,
// {
//     fn to_rust(&self) -> JsResult<T> {
//         match self {
//             JsAny::String(s) => s.to_rust(),
//             _ => Err(WrongAny)
//         }
//     }    
// }

// impl<'e, T> ToRust<T> for JsAny<'e>
// where
//     JsObject<'e>: ToRust<T>,
// {
//     fn to_rust(&self) -> JsResult<T> {
//         match self {
//             JsAny::Object(s) => s.to_rust(),
//             _ => Err(WrongAny)
//         }
//     }    
// }

impl<'e, T: 'static> ToRust<Arc<T>> for JsExternal<'e>
{
    fn to_rust(&self) -> JsResult<Arc<T>> {
        self.get_arc()
    }
}

impl<'e, T: 'static> ToRust<Rc<T>> for JsExternal<'e>
{
    fn to_rust(&self) -> JsResult<Rc<T>> {
        self.get_rc()
    }
}

impl<'e, T: 'static> ToRust<Box<T>> for JsExternal<'e>
{
    fn to_rust(&self) -> JsResult<Box<T>> {
        match self.take_box()? {
            Some(b) => Ok(b),
            _ => panic!("Extracting a box from a JsExternal that have
already been extracted. Use Option<Box<T>>")
        }
    }
}

impl<'e, T: 'static> ToRust<Option<Box<T>>> for JsExternal<'e> {
    fn to_rust(&self) -> JsResult<Option<Box<T>>> {
        self.take_box()
    }
}

use serde::de::DeserializeOwned;

impl<'e, T> ToRust<Vec<T>> for JsArray<'e>
where
    T: DeserializeOwned
{
    fn to_rust(&self) -> JsResult<Vec<T>> {
        let env = self.value.env;
        let mut vec = Vec::with_capacity(self.len()?);
        for elem in self.iter()? {
            let elem: JsResult<_> = pinar_serde::de::from_any::<T>(env, elem).map_err(Into::into);
            vec.push(elem?);
        }
        Ok(vec)
    }
}
