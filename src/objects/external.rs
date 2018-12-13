
use std::marker::PhantomData;
use crate::error::JsExternalError;
use std::sync::Arc;
use std::rc::Rc;
use napi_sys::*;
use crate::prelude::*;
use crate::external::External;
use crate::Result;

pub struct JsExternal<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

impl<'e> JsExternal<'e> {
    fn get_external<T>(&self) -> Result<*mut External<T>> {
        let mut external: *mut External<T> = std::ptr::null_mut();
        unsafe {
            Status::result(napi_get_value_external(
                self.value.env(),
                self.get_value().value,
                &mut external as *mut *mut External<T> as *mut *mut std::ffi::c_void
            ))?;
        }
        if external.is_null() {
            return Err(JsExternalError.into())
        }
        Ok(external)
    }

    pub fn take_box<T: 'static>(&self) -> Result<Option<Box<T>>> {
        let external = self.get_external::<T>()?;
        Ok(unsafe { (*external).take_box::<T>() })
    }

    pub fn get_rc<T: 'static>(&self) -> Result<Rc<T>> {
        let external = self.get_external::<T>()?;
        Ok(unsafe { (*external).get_rc::<T>() })
    }

    pub fn get_arc<T: 'static>(&self) -> Result<Arc<T>> {
        let external = self.get_external::<T>()?;
        Ok(unsafe { (*external).get_arc::<T>() })
    }
}
