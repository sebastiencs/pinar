
use std::marker::PhantomData;
use crate::error::JsExternalError;
use std::sync::Arc;
use std::rc::Rc;
use napi_sys::*;
use crate::prelude::*;
use crate::external::External;

/// A Javascript external value.  
///   
/// From the Node API documentation:
/// ```
/// This is used to pass external data through JavaScript
/// code, so it can be retrieved later by native code
/// ```
///   
/// `JsExternal` values are created when passing an [`Rc`] or [`Arc`] to Javascript.
///   
/// # Create
///
/// ```
/// #[pinar]
/// fn my_func(obj: JsObject) -> JsResult<Arc<String>> {
///     // This will create an external value
///     obj.set("external", Rc::new(10))?;
///
///     // Return an external value
///     Ok(Arc::new(String::from("hello")))
/// }
/// ```
///   
/// # Retrieve
///
/// ```
/// #[pinar]
/// fn my_func(external: Arc<String>) {
///     // The parameter is converted from a JsExternal to a Rust Arc
///     // This will panic if the external value is another type
///     // e.g: Arc<u64>
/// }
///
/// #[pinar]
/// fn my_func2(external: JsExternal) {
///     let external = external.get_arc::<u64>();
///     // Panics if the external value is not an Arc<u64>
/// }
/// ```
///
///   
/// More information on the napi reference:
/// [here](https://nodejs.org/api/n-api.html#n_api_napi_create_external)
/// 
/// [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
/// [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
pub struct JsExternal<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

impl<'e> JsExternal<'e> {
    /// Retrieves the raw `External` from a `JsExternal`
    fn get_external<T>(&self) -> JsResult<*mut External<T>> {
        let mut external: *mut External<T> = std::ptr::null_mut();
        napi_call!(napi_get_value_external(
            self.value.env(),
            self.get_value().value,
            &mut external as *mut *mut External<T> as *mut *mut std::ffi::c_void
        ))?;
        if external.is_null() {
            return Err(JsExternalError.into())
        }
        Ok(external)
    }

    /// Takes the box in the JsExternal value, leaving a None in place
    /// # Example
    /// ```
    /// // Let's consider that the external value is a Box<String>
    /// #[pinar]
    /// fn external(external: JsExternal) -> JsResult<()> {
    ///     let value = external.take_box::<String>()?;
    ///     // value is Some(Box<String>)
    ///     
    ///     let value = external.take_box::<String>()?;
    ///     // The box has been taken, it's not possible to take it more than once.
    ///     // value is None
    ///     
    ///     external.get_rc::<String>()?;
    ///     // This panics, the external value is not a Rc
    ///     
    ///     external.take_box::<usize>()?;
    ///     // This panics, the external value is not a usize
    ///
    ///     Ok(())
    /// }
    /// ```
    /// # Panics
    /// This function panics if the type of the Box is different, or if
    /// the external value is not a Box (Rc, or Arc)
    pub fn take_box<T: 'static>(&self) -> JsResult<Option<Box<T>>> {
        let external = self.get_external::<T>()?;
        // Deref raw pointer is unsafe
        Ok(unsafe { (*external).take_box::<T>() })
    }

    /// Clone the Rc from the `JsExternal`
    /// # Example
    /// ```
    /// // Let's consider that the external value is a Rc<usize>
    /// #[pinar]
    /// fn external(external: JsExternal) -> JsResult<()> {
    ///     let value = external.get_rc::<usize>()?;
    ///     // value is a Rc<usize>
    ///     
    ///     let value = external.get_rc::<usize>()?;
    ///     // value is another Rc<usize>
    ///     
    ///     external.take_box::<usize>()?; // panics, it's not a Box
    ///     external.get_rc::<String>()?; // panics, it's not a Rc<String>
    ///
    ///     Ok(())
    /// }
    /// ```
    /// # Panics
    /// This function panics if the type of the Rc is different, or if
    /// the external value is not a Rc (Box, or Arc)
    pub fn get_rc<T: 'static>(&self) -> JsResult<Rc<T>> {
        let external = self.get_external::<T>()?;
        // Deref raw pointer is unsafe
        Ok(unsafe { (*external).get_rc::<T>() })
    }

    /// Clone the Arc from the `JsExternal`
    /// # Example
    /// ```
    /// // Let's consider that the external value is an Arc<usize>
    /// #[pinar]
    /// fn external(external: JsExternal) -> JsResult<()> {
    ///     let value = external.get_arc::<usize>()?;
    ///     // value is an Arc<usize>
    ///     
    ///     let value = external.get_arc::<usize>()?;
    ///     // value is another Arc<usize>
    ///     
    ///     external.take_rc::<usize>()?; // panics, it's not a Rc
    ///     external.get_arc::<String>()?; // panics, it's not an Arc<String>
    ///
    ///     Ok(())
    /// }
    /// ```
    /// # Panics
    /// This function panics if the type of the Arc is different, or if
    /// the external value is not a Arc (Box, or Rc)
    pub fn get_arc<T: 'static>(&self) -> JsResult<Arc<T>> {
        let external = self.get_external::<T>()?;
        // Deref raw pointer is unsafe
        Ok(unsafe { (*external).get_arc::<T>() })
    }
}
