
use std::ffi::c_void;
use std::marker::PhantomData;
use std::cell::Cell;
use napi_sys::*;
use crate::Result;
use crate::prelude::*;

pub struct JsFunctionThreadSafe<T: MultiJs> {
    fun: napi_threadsafe_function,
    acquired: Cell<bool>,
    phantom: PhantomData<T>
}

unsafe impl<T: MultiJs> Send for JsFunctionThreadSafe<T> {}

impl<T: MultiJs> JsFunctionThreadSafe<T> {

    pub(crate) fn new(fun: napi_threadsafe_function) -> JsFunctionThreadSafe<T> {
        JsFunctionThreadSafe {
            fun,
            acquired: Cell::new(false),
            phantom: PhantomData
        }
    }

    pub fn call(&self, args: impl Into<Box<T>>) -> Result<()> {
        self.acquire()?;
        unsafe {
            napi_call_threadsafe_function(
                self.fun,
                Box::into_raw(args.into()) as *mut c_void,
                napi_threadsafe_function_call_mode::napi_tsfn_nonblocking
            );
        }
        Ok(())
    }

    fn acquire(&self) -> Result<()> {
        if !self.acquired.get() {
            unsafe {
                Status::result(napi_acquire_threadsafe_function(
                    self.fun
                ))?;
            }
            self.acquired.set(true);
        }
        Ok(())
    }

    fn release(&self) {
        if self.acquired.get() {
            unsafe {
                napi_release_threadsafe_function(
                    self.fun,
                    napi_threadsafe_function_release_mode::napi_tsfn_release
                );
            }
        }
    }
}

impl<T: MultiJs> Drop for JsFunctionThreadSafe<T> {
    fn drop(&mut self) {
        self.release();
    }
}
