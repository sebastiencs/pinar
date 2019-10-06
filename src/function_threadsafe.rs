
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
pub struct JsFunctionThreadSafe<T: MultiJs + 'static, R: DeserializeOwned = ()> {
    fun: Arc<AtomicPtr<napi_threadsafe_function__>>,
    phantom: PhantomData<(T, R)>
}

pub(crate) struct DataThreadSafe<T: MultiJs + 'static, R: DeserializeOwned = ()> {
    pub(crate) send_result: Option<SyncSender<R>>,
    pub(crate) args: Box<T>
}

unsafe impl<T: MultiJs, R: DeserializeOwned> Send for JsFunctionThreadSafe<T, R> {}

impl<T: MultiJs + 'static, R: DeserializeOwned> JsFunctionThreadSafe<T, R> {

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

impl<T: MultiJs + 'static, R: DeserializeOwned> Drop for JsFunctionThreadSafe<T, R> {
    fn drop(&mut self) {
        if Arc::strong_count(&self.fun) == 1 {
            self.release();
        }
    }
}
