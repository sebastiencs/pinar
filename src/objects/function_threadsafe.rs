
use std::convert::TryFrom;
use std::ffi::c_void;
use std::marker::PhantomData;

use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::sync::mpsc::{sync_channel, SyncSender};
use napi_sys::*;
use crate::prelude::*;
use serde::de::DeserializeOwned;

/// A Javascript function callable from any thread.
///
/// The function is executed on the main JS thread.  
/// The arguments and return's value types have to be specified with generic parameter.
///
/// The main JS loop won't exit as long as there are existing `JsFunctionThreadSafe`.
///
/// Result of the function can be retrieved or ignored.
///
/// # Example with result
///
/// The current thread will block until the js function returns its result.
///
/// ```
/// #[pinar]
/// fn my_func(fun: JsFunction) -> JsResult<()> {
///     let fun = fun.make_threadsafe::<(String, i64), PathBuf>()?;
///
///     std::thread::spawn(move || {
///         // The Javascript function will be called on the JS main thread
///         // and the return value is transfered back to this thread
///         let res: PathBuf = fun.call(("hello".to_string(), 124)).unwrap();
///     });
///
///     Ok(())
/// }
/// ```
///
/// # Example ignoring result
///
/// The current thread will _not_ block.
///
/// ```
/// #[pinar]
/// fn my_func(fun: JsFunction) -> JsResult<()> {
///     let fun = fun.make_threadsafe::<(String, i64)>()?;
///
///     std::thread::spawn(move || {
///         // The Javascript function will be called on the JS main thread
///         let res: () = fun.call_ignore_result(("hello".to_string(), 124)).unwrap();
///     });
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct JsFunctionThreadSafe<T, R = ()>
where
    T: MultiJs + 'static,
    R: DeserializeOwned,
{
    fun: Arc<AtomicPtr<napi_threadsafe_function__>>,
    phantom: PhantomData<(T, R)>
}

/// Data transfered between a Rust thread and the main JS thread.  
/// It includes the function arguments and a channel to send the result.  
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
    /// Creates a `JsFunctionThreadSafe` from a raw `napi_threadsafe_function`
    pub(crate) fn new(fun: napi_threadsafe_function) -> JsResult<JsFunctionThreadSafe<T, R>> {
        let fun = JsFunctionThreadSafe {
            fun: Arc::new(AtomicPtr::new(fun)),
            phantom: PhantomData
        };
        // Acquire the js function, the main JS loop won't exit until this `JsFunctionThreadSafe`
        // is dropped
        fun.acquire()?;
        Ok(fun)
    }

    /// Call the js function and wait for its result.
    pub fn call(&self, args: impl Into<Box<T>>) -> JsResult<R> {
        let (sender, receiver) = sync_channel(0);
        let data = Box::new(DataThreadSafe {
            send_result: Some(sender),
            args: args.into()
        });
        napi_call!(napi_call_threadsafe_function(
            self.fun.load(Ordering::Relaxed),
            Box::into_raw(data) as *mut c_void,
            napi_threadsafe_function_call_mode::napi_tsfn_blocking
        ))?;
        Ok(receiver.recv().unwrap())
    }

    /// Call the js function, this function _does not_ wait for the result.
    pub fn call_ignore_result(&self, args: impl Into<Box<T>>) -> JsResult<()> {
        let data = Box::new(DataThreadSafe::<_, ()> {
            send_result: None,
            args: args.into()
        });
        napi_call!(napi_call_threadsafe_function(
            self.fun.load(Ordering::Relaxed),
            Box::into_raw(data) as *mut c_void,
            napi_threadsafe_function_call_mode::napi_tsfn_blocking
        ))?;
        Ok(())
    }

    /// See https://nodejs.org/api/n-api.html#n_api_napi_acquire_threadsafe_function
    fn acquire(&self) -> JsResult<()> {
        napi_call!(napi_acquire_threadsafe_function(
            self.fun.load(Ordering::Relaxed)
        ))?;
        Ok(())
    }

    /// See https://nodejs.org/api/n-api.html#n_api_napi_release_threadsafe_function
    fn release(&self) {
        let _ = napi_call!(napi_release_threadsafe_function(
            self.fun.load(Ordering::Relaxed),
            napi_threadsafe_function_release_mode::napi_tsfn_release
        ));
    }
}

impl<T, R> Drop for JsFunctionThreadSafe<T, R>
where
    T: MultiJs + 'static,
    R: DeserializeOwned,
{
    fn drop(&mut self) {
        if Arc::strong_count(&self.fun) == 1 {
            // Release the function, so the main JS thread can exit.
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

    fn try_from(fun: &JsFunction) -> JsResult<JsFunctionThreadSafe<T, R>> {
        let mut result: napi_threadsafe_function = std::ptr::null_mut();

        let resource_name = "rust_threadsafe_function".to_js(fun.value.env)?;

        napi_call!(napi_create_threadsafe_function(
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

        JsFunctionThreadSafe::<T, R>::new(result)
    }
}

use crate::pinar_serde::de::from_any;

fn display_exception(env: Env) {
    let mut result = Value::new(env);
    unsafe {
        napi_get_and_clear_last_exception(env.env(), result.get_mut());
    }
    let _ = env.console_error(("An exception occured with a threadsafe function:\n", result));
}

/// Function executed on the main JS thread.
///
/// It is responsible of:
/// - converting values to/from JS
/// - Call the javascript function
/// - Send the result to the other rust thread
/// 
extern "C" fn __pinar_threadsafe_function<T, R>(
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
        let env = Env::from(env);
        let fun = JsFunction::from(Value::from(env, js_callback));
        let mut data: Box<DataThreadSafe<T, R>> = unsafe { Box::from_raw(data as *mut DataThreadSafe<T, R>) };
        let args = data.args;

        // TODO: Send errors to the other thread instead of panicking.
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
}
