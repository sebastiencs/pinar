
use std::convert::TryInto;
use std::marker::PhantomData;
use serde::de::DeserializeOwned;

use crate::prelude::*;
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

    pub fn make_threadsafe<T, R>(&self) -> Result<JsFunctionThreadSafe<T, R>>
    where
        T: MultiJs + 'static,
        R: DeserializeOwned,
    {
        self.try_into()
    }
}
