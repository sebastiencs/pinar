
use crate::prelude::*;
use crate::function_threadsafe::JsFunctionThreadSafe;
use crate::*;

pub struct JsFunction {
    pub(crate) value: Value
}

impl JsFunction {
    pub fn call_with_this(&self, this: impl AsJs, args: impl MultiJs) -> Result<JsUnknown> {
        let args: Vec<_> = args.make_iter(&self.value.env)?
                               .into_iter()
                               .map(|v| v.value)
                               .collect();
        let mut result = Value::new(self.value.env);
        let this = this.as_js(&self.value.env)?;
        unsafe {
            Status::result(napi_call_function(self.value.env(),
                                              this.get_value().value,
                                              self.value.get(),
                                              args.len(),
                                              args.as_ptr(),
                                              result.get_mut()))?;
        };
        JsUnknown::from(result)
    }

    pub fn call(&self, args: impl MultiJs) -> Result<JsUnknown> {
        let global = self.value.env.global()?;
        self.call_with_this(global, args)
    }

    pub fn new_instance(&self, args: impl MultiJs) -> Result<JsObject> {
        let args: Vec<_> = args.make_iter(&self.value.env)?
                               .into_iter()
                               .map(|v| v.value)
                               .collect();
        let mut result = Value::new(self.value.env);
        unsafe {
            Status::result(napi_new_instance(self.value.env(),
                                             self.value.get(),
                                             args.len(),
                                             args.as_ptr(),
                                             result.get_mut()))?;
        }
        Ok(JsObject::from(result))
    }

    pub fn make_threadsafe<T: MultiJs>(&self) -> Result<JsFunctionThreadSafe<T>> {
        let mut result: napi_threadsafe_function = std::ptr::null_mut();
        unsafe {
            Status::result(napi_create_threadsafe_function(
                self.value.env(),
                self.value.get(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                1,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                Some(__pinar_threadsafe_function::<T>),
                &mut result
            ))?;
        }
        Ok(JsFunctionThreadSafe::<T>::new(result))
    }
}

unsafe extern "C" fn __pinar_threadsafe_function<T: MultiJs>(
    env: napi_env,
    js_callback: napi_value,
    _context: *mut ::std::os::raw::c_void,
    data: *mut ::std::os::raw::c_void,
) {
    if !env.is_null() && !js_callback.is_null() {
        let env = Env::from(env);
        let fun = JsFunction::from(Value::from(env, js_callback));
        let args: Box<T> = Box::from_raw(data as *mut T);
        fun.call(*args).ok();
    }
}
