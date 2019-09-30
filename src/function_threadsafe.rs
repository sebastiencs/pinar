
use std::ffi::c_void;
use std::marker::PhantomData;
use std::cell::Cell;
use std::sync::mpsc::{sync_channel, SyncSender};
use napi_sys::*;
use crate::Result;
use crate::prelude::*;
use serde::de::DeserializeOwned;

pub struct JsFunctionThreadSafe<T: MultiJs, R: DeserializeOwned> {
    fun: napi_threadsafe_function,
    acquired: Cell<bool>,
    phantom: PhantomData<(T, R)>
}

impl<T: MultiJs, R: DeserializeOwned> std::fmt::Debug for JsFunctionThreadSafe<T, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("JsFunctionThreadSafe")
            .field("function ptr", &self.fun)
            .field("acquired", &self.acquired)
            .finish()
    }
}

pub(crate) struct DataThreadSafe<T: MultiJs, R: DeserializeOwned> {
    pub(crate) send_result: Option<SyncSender<R>>,
    pub(crate) args: Box<T>
}

unsafe impl<T: MultiJs, R: DeserializeOwned> Send for JsFunctionThreadSafe<T, R> {}

impl<T: MultiJs, R: DeserializeOwned> JsFunctionThreadSafe<T, R> {

    pub(crate) fn new(fun: napi_threadsafe_function) -> JsFunctionThreadSafe<T, R> {
        JsFunctionThreadSafe {
            fun,
            acquired: Cell::new(false),
            phantom: PhantomData
        }
    }

    pub fn call(&self, args: impl Into<Box<T>>) -> Result<R> {
        self.acquire()?;
        let (sender, receiver) = sync_channel(0);
        let data = Box::new(DataThreadSafe {
            send_result: Some(sender),
            args: args.into()
        });
        unsafe {
            Status::result(napi_call_threadsafe_function(
                self.fun,
                Box::into_raw(data) as *mut c_void,
                napi_threadsafe_function_call_mode::napi_tsfn_nonblocking
            ))?;
        }
        Ok(receiver.recv().unwrap())
    }

    pub fn call_ignore_result(&self, args: impl Into<Box<T>>) -> Result<()> {
        self.acquire()?;
        let data = Box::new(DataThreadSafe::<_, ()> {
            send_result: None,
            args: args.into()
        });
        unsafe {
            Status::result(napi_call_threadsafe_function(
                self.fun,
                Box::into_raw(data) as *mut c_void,
                napi_threadsafe_function_call_mode::napi_tsfn_nonblocking
            ))?;
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

impl<T: MultiJs, R: DeserializeOwned> Drop for JsFunctionThreadSafe<T, R> {
    fn drop(&mut self) {
        self.release();
    }
}
