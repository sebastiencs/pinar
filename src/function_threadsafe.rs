
use std::convert::TryFrom;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::cell::Cell;
use std::sync::atomic::{AtomicPtr, AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc::{sync_channel, SyncSender};
use napi_sys::*;
use crate::Result;
use crate::prelude::*;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub struct JsFunctionThreadSafe<T, R = ()>
where
    T: MultiJs + 'static,
    R: DeserializeOwned,
{
    fun: Arc<AtomicPtr<napi_threadsafe_function__>>,
    phantom: PhantomData<(T, R)>
}

struct DataThreadSafe<T, R = ()>
where
    T: MultiJs + 'static,
    R: DeserializeOwned,
{
    send_result: Option<SyncSender<R>>,
    args: Box<T>
}

unsafe impl<T: MultiJs, R: DeserializeOwned> Send for JsFunctionThreadSafe<T, R> {}

impl<T, R> JsFunctionThreadSafe<T, R>
where
    T: MultiJs + 'static,
    R: DeserializeOwned,
{
    pub(crate) fn new(fun: napi_threadsafe_function) -> Result<JsFunctionThreadSafe<T, R>> {
        let fun = JsFunctionThreadSafe {
            fun: Arc::new(AtomicPtr::new(fun)),
            phantom: PhantomData
        };
        fun.acquire()?;
        Ok(fun)
    }

    pub fn call(&self, args: impl Into<Box<T>>) -> Result<R> {
        let (sender, receiver) = sync_channel(0);
        let data = Box::new(DataThreadSafe {
            send_result: Some(sender),
            args: args.into()
        });
        unsafe {
            Status::result(napi_call_threadsafe_function(
                self.fun.load(Ordering::Relaxed),
                Box::into_raw(data) as *mut c_void,
                napi_threadsafe_function_call_mode::napi_tsfn_nonblocking
            ))?;
        }
        Ok(receiver.recv().unwrap())
    }

    pub fn call_ignore_result(&self, args: impl Into<Box<T>>) -> Result<()> {
        let data = Box::new(DataThreadSafe::<_, ()> {
            send_result: None,
            args: args.into()
        });
        unsafe {
            Status::result(napi_call_threadsafe_function(
                self.fun.load(Ordering::Relaxed),
                Box::into_raw(data) as *mut c_void,
                napi_threadsafe_function_call_mode::napi_tsfn_nonblocking
            ))?;
        }
        Ok(())
    }

    fn acquire(&self) -> Result<()> {
        unsafe {
            Status::result(napi_acquire_threadsafe_function(
                self.fun.load(Ordering::Relaxed)
            ))?;
        }
        Ok(())
    }

    fn release(&self) {
        unsafe {
            napi_release_threadsafe_function(
                self.fun.load(Ordering::Relaxed),
                napi_threadsafe_function_release_mode::napi_tsfn_release
            );
        }
    }
}

impl<T, R> Drop for JsFunctionThreadSafe<T, R>
where
    T: MultiJs + 'static,
    R: DeserializeOwned,
{
    fn drop(&mut self) {
        if Arc::strong_count(&self.fun) == 1 {
            self.release();
        }
    }
}

impl<'e, T, R> TryFrom<&JsFunction<'e>> for JsFunctionThreadSafe<T, R>
where
    T: MultiJs + 'static,
    R: DeserializeOwned,
{
    type Error = crate::error::Error;

    fn try_from(fun: &JsFunction) -> Result<JsFunctionThreadSafe<T, R>> {
        let mut result: napi_threadsafe_function = std::ptr::null_mut();

        let resource_name = "rust_threadsafe_function".to_js(fun.value.env)?;

        unsafe {
            Status::result(napi_create_threadsafe_function(
                fun.value.env(),
                fun.value.get(),
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

use crate::pinar_serde::de::from_any;

fn display_exception(env: Env) {
    let mut result = Value::new(env);
    unsafe {
        napi_get_and_clear_last_exception(env.env(), result.get_mut());
    }
    env.error(("An exception occured with a threadsafe function:\n", result));
}

#[inline(always)]
fn threadsafe_callback<T, R>(
    env: napi_env,
    js_callback: napi_value,
    data: *mut ::std::os::raw::c_void,
)
where
    T: MultiJs + 'static,
    R: DeserializeOwned,
{
    let env = Env::from(env);
    let fun = JsFunction::from(Value::from(env, js_callback));
    let mut data: Box<DataThreadSafe<T, R>> = unsafe { Box::from_raw(data as *mut DataThreadSafe<T, R>) };
    let args = data.args;

    let result = match fun.call(*args) {
        Ok(result) => result,
        Err(e) => {
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

unsafe extern "C" fn __pinar_threadsafe_function<T, R>(
    env: napi_env,
    js_callback: napi_value,
    _context: *mut ::std::os::raw::c_void,
    data: *mut ::std::os::raw::c_void,
)
where
    T: MultiJs + 'static,
    R: DeserializeOwned,
{
    if !env.is_null() && !js_callback.is_null() {
        threadsafe_callback::<T, R>(env, js_callback, data);
    }
}
