
use crate::ToJs;
use crate::classes::__pinar_drop_rc;
use std::os::raw::c_char;
use crate::__pinar_callback_function;
use crate::module::ModuleFunction;
use crate::jsreturn::JsReturn;
use crate::arguments::{FromArguments, Arguments};
use std::sync::Arc;
use crate::external::External;
use std::rc::Rc;
use crate::classes::__pinar_drop_box;
use std::ffi::c_void;
use napi_sys::*;
use std::ffi::CString;

use crate::{
    JsString,
    JsObject,
    JsArray,
    JsNumber,
    JsUndefined,
    JsFunction,
    JsExternal,
    JsBoolean,
    JsNull,
    JsAny,
};
use crate::{Result, Value, JsValue};
use crate::status::Status;

#[derive(Copy, Clone)]
pub struct Env {
    env: napi_env
}

impl Env {
    pub(crate) fn env(&self) -> napi_env {
        self.env
    }

    pub(crate) fn from(env: napi_env) -> Env {
        Env { env }
    }

    pub fn boolean<'e>(&self, b: bool) -> Result<JsBoolean<'e>> {
        let mut value = Value::new(*self);
        unsafe {
            Status::result(napi_get_boolean(
                self.env,
                b,
                value.get_mut()
            ))?
        };
        Ok(JsBoolean::from(value))
    }

    pub fn double<'e>(&self, d: f64) -> Result<JsNumber<'e>> {
        let mut value = Value::new(*self);
        unsafe {
            Status::result(napi_create_double(
                self.env,
                d,
                value.get_mut()
            ))?
        };
        Ok(JsNumber::from(value))
    }

    pub fn object<'e>(&self) -> Result<JsObject<'e>> {
        let mut value = Value::new(*self);
        unsafe {
            Status::result(napi_create_object(
                self.env,
                value.get_mut()
            ))?
        };
        Ok(JsObject::from(value))
    }

    pub fn number<'e>(&self, n: i64) -> Result<JsNumber<'e>> {
        let mut value = Value::new(*self);
        unsafe {
            Status::result(napi_create_int64(
                self.env,
                n,
                value.get_mut()
            ))?
        };
        Ok(JsNumber::from(value))
    }

    pub fn string<'e, S: AsRef<str>>(&self, s: S) -> Result<JsString<'e>> {
        let mut value = Value::new(*self);
        let s = s.as_ref();
        unsafe {
            Status::result(napi_create_string_utf8(
                self.env,
                s.as_ptr() as *const c_char,
                s.len(),
                value.get_mut()
            ))?
        };
        Ok(JsString::from(value))
    }

    pub fn array<'e>(&self) -> Result<JsArray<'e>> {
        let mut value = Value::new(*self);
        unsafe {
            Status::result(napi_create_array(
                self.env,
                value.get_mut()
            ))?
        };
        Ok(JsArray::from(value))
    }

    pub fn array_with_capacity<'e>(&self, cap: usize) -> Result<JsArray<'e>> {
        let mut value = Value::new(*self);
        unsafe {
            Status::result(napi_create_array_with_length(
                self.env,
                cap,
                value.get_mut()
            ))?
        };
        Ok(JsArray::from(value))
    }

    pub fn global<'e>(&self) -> Result<JsObject<'e>> {
        let mut global = Value::new(*self);
        unsafe {
            Status::result(napi_get_global(
                self.env,
                global.get_mut()
            ))?
        };
        Ok(JsObject::from(global))
    }

    pub fn undefined<'e>(&self) -> Result<JsUndefined<'e>> {
        let mut undefined = Value::new(*self);
        unsafe {
            Status::result(napi_get_undefined(
                self.env,
                undefined.get_mut()
            ))?
        };
        Ok(JsUndefined::from(undefined))
    }

    pub fn null<'e>(&self) -> Result<JsNull<'e>> {
        let mut null = Value::new(*self);
        unsafe {
            Status::result(napi_get_null(
                self.env,
                null.get_mut()
            ))?
        };
        Ok(JsNull::from(null))
    }

    pub fn external_box<'e, T: 'static>(&self, ptr: Box<T>) -> Result<JsExternal<'e>> {
        let mut result = Value::new(*self);
        let external = Box::new(External::new_box(ptr));
        unsafe {
            Status::result(napi_create_external(
                self.env,
                Box::into_raw(external) as *mut c_void,
                Some(__pinar_drop_box::<External<T>>),
                std::ptr::null_mut(),
                result.get_mut()
            ))?;
        }
        Ok(JsExternal::from(result))
    }

    pub fn external_rc<'e, T: 'static>(&self, ptr: Rc<T>) -> Result<JsExternal<'e>> {
        let mut result = Value::new(*self);
        let external = Box::new(External::new_rc(ptr));
        unsafe {
            Status::result(napi_create_external(
                self.env,
                Box::into_raw(external) as *mut c_void,
                Some(__pinar_drop_box::<External<T>>),
                std::ptr::null_mut(),
                result.get_mut()
            ))?;
        }
        Ok(JsExternal::from(result))
    }

    pub fn external_arc<'e, T: 'static>(&self, ptr: Arc<T>) -> Result<JsExternal<'e>> {
        let mut result = Value::new(*self);
        let external = Box::new(External::new_arc(ptr));
        unsafe {
            Status::result(napi_create_external(
                self.env,
                Box::into_raw(external) as *mut c_void,
                Some(__pinar_drop_box::<External<T>>),
                std::ptr::null_mut(),
                result.get_mut()
            ))?;
        }
        Ok(JsExternal::from(result))
    }

    pub fn function<'e, N, F, A, R>(&self, name: N, fun: F) -> Result<JsFunction<'e>>
    where
        N: AsRef<str>,
        A: FromArguments + 'static,
        R: for<'env> JsReturn<'env> + 'static,
        F: Fn(A) -> R + 'static
    {
        let name = name.as_ref();
        let data = Rc::new(ModuleFunction::new(name, fun));
        self.function_internal(name, data)
    }

    pub(crate) fn function_internal<'e, N>(&self, name: N, fun: Rc<ModuleFunction>) -> Result<JsFunction<'e>>
    where
        N: AsRef<str>
    {
        let mut result = Value::new(*self);
        let name = name.as_ref();
        let raw = Rc::into_raw(fun);
        unsafe {
            Status::result(napi_create_function(
                self.env,
                name.as_ptr() as *const i8,
                name.len(),
                Some(__pinar_callback_function),
                raw as *mut std::ffi::c_void,
                result.get_mut()
            ))?;
            Status::result(napi_add_finalizer(
                self.env,
                result.get(),
                raw as *mut std::ffi::c_void,
                Some(__pinar_drop_rc::<ModuleFunction>),
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ))?;
        }
        Ok(JsFunction::from(result))
    }

    pub(crate) fn callback_info<D>(&self, info: napi_callback_info) -> Result<(*mut D, Arguments)> {
        let mut argc: usize = 12;
        let mut argv: Vec<napi_value> = Vec::with_capacity(argc);
        let mut this = Value::new(*self);
        let mut data_ptr: *mut D = std::ptr::null_mut();
        unsafe {
            Status::result(napi_get_cb_info(
                self.env,
                info,
                &mut argc as *mut usize,
                argv.as_mut_ptr() as *mut napi_value,
                this.get_mut(),
                &mut data_ptr as *mut *mut D as *mut *mut std::ffi::c_void
            ))?;
            argv.set_len(argc);
        }
        Ok((data_ptr, Arguments::new(*self, this, &argv)?))
    }

    pub fn throw<'e, V>(&self, error: V) -> Result<()>
    where
        V: ToJs<'e>
    {
        let error = error.to_js(*self)?.get_value();
        unsafe {
            Status::result(napi_throw(self.env, error.get()))?;
        }
        Err(Status::PendingException.into())
    }

    pub fn throw_error<M>(&self, msg: M, code: impl Into<Option<String>>) -> Result<()>
    where
        M: AsRef<str>
    {
        let msg = CString::new(msg.as_ref()).unwrap();
        let code = code.into().map(|c| CString::new(c).unwrap());
        let code_ptr = match code.as_ref() {
            Some(code) => code.as_ptr(),
            _ => std::ptr::null()
        };
        unsafe {
            Status::result(napi_throw_error(
                self.env,
                code_ptr,
                msg.as_ptr()
            ))?;
        }
        Err(Status::PendingException.into())
    }

    pub fn throw_type_error<'s, M>(&self, msg: M, code: impl Into<Option<&'s str>>) -> Result<()>
    where
        M: AsRef<str>
    {
        let msg = CString::new(msg.as_ref()).unwrap();
        let code = code.into().map(|c| CString::new(c).unwrap());
        let code_ptr = match code.as_ref() {
            Some(code) => code.as_ptr(),
            _ => std::ptr::null()
        };
        unsafe {
            Status::result(napi_throw_type_error(
                self.env,
                code_ptr,
                msg.as_ptr()
            ))?;
        }
        Err(Status::PendingException.into())
    }

    pub fn throw_range_error<'s, M>(&self, msg: M, code: impl Into<Option<&'s str>>) -> Result<()>
    where
        M: AsRef<str>
    {
        let msg = CString::new(msg.as_ref()).unwrap();
        let code = code.into().map(|c| CString::new(c).unwrap());
        let code_ptr = match code.as_ref() {
            Some(code) => code.as_ptr(),
            _ => std::ptr::null()
        };
        unsafe {
            Status::result(napi_throw_range_error(
                self.env,
                code_ptr,
                msg.as_ptr()
            ))?;
        }
        Err(Status::PendingException.into())
    }

    pub fn run_script<'e, S>(&self, script: S) -> Result<JsAny<'e>>
    where
        S: AsRef<str>
    {
        let script = self.string(script)?;
        let mut result = Value::new(*self);
        unsafe {
            Status::result(napi_run_script(
                self.env,
                script.get_value().value,
                result.get_mut()
            ))?;
        }
        JsAny::from(result)
    }
}
