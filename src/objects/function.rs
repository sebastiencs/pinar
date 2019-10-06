
use std::marker::PhantomData;
use crate::prelude::*;
use crate::function_threadsafe::{JsFunctionThreadSafe, DataThreadSafe};
use crate::*;

pub struct JsFunction<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

impl<'e> JsFunction<'e> {
    pub fn call(&self, args: impl MultiJs) -> Result<JsAny<'e>> {
        let global = self.value.env.global()?;
        self.call_with_this(global, args)
    }

    pub fn call_with_this(&self, this: impl ToJs<'e>, args: impl MultiJs) -> Result<JsAny<'e>> {
        let args: Vec<_> = args.make_iter(self.value.env)?
                               .into_iter()
                               .map(|v| v.value)
                               .collect();
        let mut result = Value::new(self.value.env);
        let this = this.to_js(self.value.env)?;
        unsafe {
            Status::result(napi_call_function(
                self.value.env(),
                this.get_value().value,
                self.value.get(),
                args.len(),
                args.as_ptr(),
                result.get_mut()
            ))?;
        };
        JsAny::from(result)
    }

    pub fn new_instance(&self, args: impl MultiJs) -> Result<JsObject<'e>> {
        let args: Vec<_> = args.make_iter(self.value.env)?
                               .into_iter()
                               .map(|v| v.value)
                               .collect();
        let mut result = Value::new(self.value.env);
        unsafe {
            Status::result(napi_new_instance(
                self.value.env(),
                self.value.get(),
                args.len(),
                args.as_ptr(),
                result.get_mut()
            ))?;
        }
        Ok(JsObject::from(result))
    }

    pub fn make_threadsafe<T: MultiJs + 'static, R: DeserializeOwned>(&self) -> Result<JsFunctionThreadSafe<T, R>> {
        let mut result: napi_threadsafe_function = std::ptr::null_mut();

        let resource_name = "rust_threadsafe_function".to_js(self.value.env)?;

        unsafe {
            Status::result(napi_create_threadsafe_function(
                self.value.env(),
                self.value.get(),
                std::ptr::null_mut(),
                resource_name.get_value().get(),
                0,
                1,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                Some(__pinar_threadsafe_function::<T, R>),
                &mut result
            ))?;
        }
        JsFunctionThreadSafe::<T, R>::new(result)
    }
}

use serde::de::DeserializeOwned;
use crate::pinar_serde::de::from_any;

fn display_exception(env: Env) {
    let mut result = Value::new(env);
    unsafe {
        napi_get_and_clear_last_exception(env.env(), result.get_mut());
    }
    env.error(("An exception occured with a threadsafe function:\n", result));
}

pub(crate) unsafe extern "C" fn __pinar_threadsafe_function<T: MultiJs + 'static, R: DeserializeOwned>(
    env: napi_env,
    js_callback: napi_value,
    _context: *mut ::std::os::raw::c_void,
    data: *mut ::std::os::raw::c_void,
) {
    if !env.is_null() && !js_callback.is_null() {
        let env = Env::from(env);
        let fun = JsFunction::from(Value::from(env, js_callback));
        let mut data: Box<DataThreadSafe<T, R>> = Box::from_raw(data as *mut DataThreadSafe<T, R>);
        let args = data.args;

        let result = match fun.call(*args) {
            Ok(result) => result,
            Err(e) => {
                // if let Some(Status::PendingException) = e.is_type::<Status>() {
                //     display_exception(env);
                // };
                if let Some(Status::PendingException) = e.downcast_ref::<Status>() {
                    display_exception(env);
                };
                panic!("Threadsafe functions throwing exception is not supported with Pinar");
            }
        };

        if let Some(sender) = data.send_result.take() {
            let result: R = match from_any(env, result) {
                Ok(result) => result,
                Err(e) => panic!("An error occured while deserializing result of a threadsafe function: {:?}", e)
            };
            sender.send(result).unwrap()
        };
    }
}
