
use crate::module::__pinar_dispatch_function;
use crate::ToJs;
use crate::classes::__pinar_drop_rc;
use std::os::raw::c_char;
//use crate::__pinar_callback_function;
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
use crate::multi_js::MultiJs;

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
use crate::{JsResult, Value, JsValue};
use crate::status::Status;

/// Represent the Javascript context in which the native function has been invoked.
///
/// It can be used to create javascript values, throw errors, access global object, ..
///
/// Javascript values can be created manually with methods of `Env`, but it is
/// recommended to use the traits [`ToJs`] and [`ToRust`].
///
/// [`ToJs`]: ./trait.ToJs.html
/// [`ToRust`]: ./trait.ToRust.html
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

    /// Calls the Javascript `console.log` function.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<()> {
    ///     env.console_log("hello on stdout")?;
    ///     env.console_log((1, vec![], Arc::new(10)))?;
    ///     Ok(())
    /// }
    /// ```
    pub fn console_log(&self, args: impl MultiJs) -> JsResult<()> {
        let log = self.global()?
                      .get("console")?
                      .as_jsobject()?
                      .get("log")?
                      .as_jsfunction()?;

        log.call(args)?;
        Ok(())
    }

    /// Calls the Javascript `console.error` function.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<()> {
    ///     env.console_error("hello on stderr")?;
    ///     env.console_error((9, Rc::new(1), vec![]))?;
    ///     Ok(())
    /// }
    /// ```
    pub fn console_error(&self, args: impl MultiJs) -> JsResult<()> {
        let error = self.global()?
                        .get("console")?
                        .as_jsobject()?
                        .get("error")?
                        .as_jsfunction()?;

        error.call(args)?;
        Ok(())
    }

    /// Creates a [`JsBoolean`].
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsBoolean> {
    ///     env.boolean(true)
    /// }
    /// ```
    pub fn boolean<'e>(&self, b: bool) -> JsResult<JsBoolean<'e>> {
        let mut value = Value::new(*self);

        napi_call!(napi_get_boolean(
            self.env,
            b,
            value.get_mut()
        ))?;

        Ok(JsBoolean::from(value))
    }

    /// Creates a [`JsNumber`] from a [`f64`].
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsNumber> {
    ///     env.double(42.0)
    /// }
    /// ```
    pub fn double<'e>(&self, d: f64) -> JsResult<JsNumber<'e>> {
        let mut value = Value::new(*self);

        napi_call!(napi_create_double(
            self.env,
            d,
            value.get_mut()
        ))?;

        Ok(JsNumber::from(value))
    }

    /// Creates an empty javascript object
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsObject> {
    ///     let mut obj = env.object()?;
    ///     obj.set("a", 10)?;
    ///     Ok(obj)
    /// }
    /// ```
    pub fn object<'e>(&self) -> JsResult<JsObject<'e>> {
        let mut value = Value::new(*self);

        napi_call!(napi_create_object(
            self.env,
            value.get_mut()
        ))?;

        Ok(JsObject::from(value))
    }

    /// Creates a [`JsNumber`] from a [`i64`].
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsNumber> {
    ///     env.number(42)
    /// }
    /// ```
    pub fn number<'e>(&self, n: i64) -> JsResult<JsNumber<'e>> {
        let mut value = Value::new(*self);

        napi_call!(napi_create_int64(
            self.env,
            n,
            value.get_mut()
        ))?;

        Ok(JsNumber::from(value))
    }

    /// Creates a javascript string.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsString> {
    ///     env.string("hello")
    /// }
    /// ```
    pub fn string<'e, S: AsRef<str>>(&self, s: S) -> JsResult<JsString<'e>> {
        let mut value = Value::new(*self);
        let s = s.as_ref();

        napi_call!(napi_create_string_utf8(
            self.env,
            s.as_ptr() as *const c_char,
            s.len(),
            value.get_mut()
        ))?;

        Ok(JsString::from(value))
    }

    /// Creates an empty javascript array.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsArray> {
    ///     let mut array = env.array()?;
    ///     array.set(0, 10)
    ///     Ok(array)
    /// }
    /// ```
    pub fn array<'e>(&self) -> JsResult<JsArray<'e>> {
        let mut value = Value::new(*self);

        napi_call!(napi_create_array(
            self.env,
            value.get_mut()
        ))?;

        Ok(JsArray::from(value))
    }

    /// Creates a javascript array with the specified capacity.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsArray> {
    ///     env.array_with_capacity(100)
    /// }
    /// ```
    pub fn array_with_capacity<'e>(&self, cap: usize) -> JsResult<JsArray<'e>> {
        let mut value = Value::new(*self);

        napi_call!(napi_create_array_with_length(
            self.env,
            cap,
            value.get_mut()
        ))?;

        Ok(JsArray::from(value))
    }

    /// Returns the javascript `global` object.
    pub fn global<'e>(&self) -> JsResult<JsObject<'e>> {
        let mut global = Value::new(*self);

        napi_call!(napi_get_global(
            self.env,
            global.get_mut()
        ))?;

        Ok(JsObject::from(global))
    }

    /// Returns the javascript `undefined`.
    pub fn undefined<'e>(&self) -> JsResult<JsUndefined<'e>> {
        let mut undefined = Value::new(*self);

        napi_call!(napi_get_undefined(
            self.env,
            undefined.get_mut()
        ))?;

        Ok(JsUndefined::from(undefined))
    }

    /// Returns the javascript `null`.
    pub fn null<'e>(&self) -> JsResult<JsNull<'e>> {
        let mut null = Value::new(*self);

        napi_call!(napi_get_null(
            self.env,
            null.get_mut()
        ))?;

        Ok(JsNull::from(null))
    }

    /// Creates a javascript external object from a [`Box`]
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsExternal> {
    ///     env.external_box(Box::new(100))
    /// }
    /// ```
    pub fn external_box<'e, T: 'static>(&self, ptr: Box<T>) -> JsResult<JsExternal<'e>> {
        let mut result = Value::new(*self);
        let external = Box::new(External::new_box(ptr));

        napi_call!(napi_create_external(
            self.env,
            Box::into_raw(external) as *mut c_void,
            Some(__pinar_drop_box::<External<T>>),
            std::ptr::null_mut(),
            result.get_mut()
        ))?;

        Ok(JsExternal::from(result))
    }

    /// Creates a javascript external object from a [`Rc`].
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsExternal> {
    ///     env.external_rc(Rc::new(100))
    /// }
    /// ```
    pub fn external_rc<'e, T: 'static>(&self, ptr: Rc<T>) -> JsResult<JsExternal<'e>> {
        let mut result = Value::new(*self);
        let external = Box::new(External::new_rc(ptr));

        napi_call!(napi_create_external(
            self.env,
            Box::into_raw(external) as *mut c_void,
            Some(__pinar_drop_box::<External<T>>),
            std::ptr::null_mut(),
            result.get_mut()
        ))?;

        Ok(JsExternal::from(result))
    }

    /// Creates a javascript external object from a [`Arc`].
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsExternal> {
    ///     env.external_arc(Arc::new(100))
    /// }
    /// ```
    pub fn external_arc<'e, T: 'static>(&self, ptr: Arc<T>) -> JsResult<JsExternal<'e>> {
        let mut result = Value::new(*self);
        let external = Box::new(External::new_arc(ptr));

        napi_call!(napi_create_external(
            self.env,
            Box::into_raw(external) as *mut c_void,
            Some(__pinar_drop_box::<External<T>>),
            std::ptr::null_mut(),
            result.get_mut()
        ))?;

        Ok(JsExternal::from(result))
    }

    /// Creates a javascript function.
    ///
    /// # Example
    ///
    /// ```
    /// fn my_js_func(n: i64, s: String) -> PathBuf {
    ///     PathBuf::from("/etc/hosts")
    /// }
    ///
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<JsFunction> {
    ///     env.function("jsfunc", my_js_func)
    /// }
    /// ```
    pub fn function<'e, N, F, A, R>(&self, name: N, fun: F) -> JsResult<JsFunction<'e>>
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

    pub(crate) fn function_internal<'e, N>(&self, name: N, fun: Rc<ModuleFunction>) -> JsResult<JsFunction<'e>>
    where
        N: AsRef<str>
    {
        let mut result = Value::new(*self);
        let name = name.as_ref();
        let raw = Rc::into_raw(fun);

        napi_call!(napi_create_function(
            self.env,
            name.as_ptr() as *const i8,
            name.len(),
            Some(__pinar_dispatch_function),
            raw as *mut std::ffi::c_void,
            result.get_mut()
        ))?;
        napi_call!(napi_add_finalizer(
            self.env,
            result.get(),
            raw as *mut std::ffi::c_void,
            Some(__pinar_drop_rc::<ModuleFunction>),
            std::ptr::null_mut(),
            std::ptr::null_mut()
        ))?;

        Ok(JsFunction::from(result))
    }

    pub(crate) fn callback_info<D>(&self, info: napi_callback_info) -> JsResult<(*mut D, Arguments)> {
        let mut argc: usize = 12;
        let mut argv: Vec<napi_value> = Vec::with_capacity(argc);
        let mut this = Value::new(*self);
        let mut data_ptr: *mut D = std::ptr::null_mut();

        napi_call!(napi_get_cb_info(
            self.env,
            info,
            &mut argc as *mut usize,
            argv.as_mut_ptr() as *mut napi_value,
            this.get_mut(),
            &mut data_ptr as *mut *mut D as *mut *mut std::ffi::c_void
        ))?;

        unsafe { argv.set_len(argc) };

        Ok((data_ptr, Arguments::new(*self, this, &argv)?))
    }

    /// Throws the JavaScript value provided
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<()> {
    ///     env.throw("some error")
    /// }
    /// ```
    pub fn throw<'e, V>(&self, error: V) -> JsResult<()>
    where
        V: ToJs<'e>
    {
        let error = error.to_js(*self)?.get_value();

        napi_call!(napi_throw(self.env, error.get()))?;

        Err(Status::PendingException.into())
    }

    /// Throws a JavaScript `Error` with the text provided.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<()> {
    ///     env.throw_error("some message", "my_code")
    /// }
    /// ```
    pub fn throw_error<M>(&self, msg: M, code: impl Into<Option<String>>) -> JsResult<()>
    where
        M: AsRef<str>
    {
        let msg = CString::new(msg.as_ref()).unwrap();
        let code = code.into().map(|c| CString::new(c).unwrap());
        let code_ptr = match code.as_ref() {
            Some(code) => code.as_ptr(),
            _ => std::ptr::null()
        };

        napi_call!(napi_throw_error(
            self.env,
            code_ptr,
            msg.as_ptr()
        ))?;

        Err(Status::PendingException.into())
    }

    /// Throws a JavaScript `TypeError` with the text provided.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<()> {
    ///     env.throw_type_error("some message", "my_code")
    /// }
    /// ```
    pub fn throw_type_error<'s, M>(&self, msg: M, code: impl Into<Option<&'s str>>) -> JsResult<()>
    where
        M: AsRef<str>
    {
        let msg = CString::new(msg.as_ref()).unwrap();
        let code = code.into().map(|c| CString::new(c).unwrap());
        let code_ptr = match code.as_ref() {
            Some(code) => code.as_ptr(),
            _ => std::ptr::null()
        };

        napi_call!(napi_throw_type_error(
            self.env,
            code_ptr,
            msg.as_ptr()
        ))?;

        Err(Status::PendingException.into())
    }

    /// Throws a JavaScript `RangeError` with the text provided.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<()> {
    ///     env.throw_range_error("some message", "my_code")
    /// }
    /// ```
    pub fn throw_range_error<'s, M>(&self, msg: M, code: impl Into<Option<&'s str>>) -> JsResult<()>
    where
        M: AsRef<str>
    {
        let msg = CString::new(msg.as_ref()).unwrap();
        let code = code.into().map(|c| CString::new(c).unwrap());
        let code_ptr = match code.as_ref() {
            Some(code) => code.as_ptr(),
            _ => std::ptr::null()
        };

        napi_call!(napi_throw_range_error(
            self.env,
            code_ptr,
            msg.as_ptr()
        ))?;

        Err(Status::PendingException.into())
    }

    /// Executes the provided script.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(env: Env) -> JsResult<()> {
    ///     let result = env.run_script("console.log('hello')")?;
    ///     Ok(())
    /// }
    /// ```
    ///
    ///
    pub fn run_script<'e, S>(&self, script: S) -> JsResult<JsAny<'e>>
    where
        S: AsRef<str>
    {
        let script = self.string(script)?;
        let mut result = Value::new(*self);

        napi_call!(napi_run_script(
            self.env,
            script.get_value().value,
            result.get_mut()
        ))?;

        JsAny::from(result)
    }
}
