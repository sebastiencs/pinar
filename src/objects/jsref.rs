
use std::marker::PhantomData;
use napi_sys::*;
use crate::Result;
use crate::prelude::*;

#[allow(dead_code)]
pub struct JsRef<T: JsValue> {
    pub(crate) env: Env,
    pub(crate) js_ref: napi_ref,
    phantom: PhantomData<T>
}

pub trait AsJsRef<T: JsValue> {
    fn as_js_ref(&self) -> Result<JsRef<T>>;
}

impl<T: JsValue> AsJsRef<T> for T {
    fn as_js_ref(&self) -> Result<JsRef<T>> {
        let env = self.get_value().env;
        let mut js_ref: napi_ref = std::ptr::null_mut();
        unsafe {
            Status::result(napi_create_reference(
                env.env(),
                self.get_value().value,
                1,
                &mut js_ref as *mut napi_ref
            ))?;
        }
        Ok(JsRef {
            env,
            js_ref,
            phantom: PhantomData
        })
    }
}
