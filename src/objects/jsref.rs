
use std::marker::PhantomData;
use std::rc::Rc;

use napi_sys::*;
use crate::Result;
use crate::prelude::*;

pub(crate) struct JsRefInner {
    pub(crate) env: Env,
    pub(crate) js_ref: napi_ref,
}

/// A reference to a Javascript value.
///
/// Javascript objects live as long as the lifespan of the native method call.  
/// Making a `JsRef` of a Javascript object extends that lifespan, allowing
/// the user to keep references to a Javascript value.
///
/// The Javascript references have the `'static` lifetime.
///
/// # Example
/// ```
/// struct MyClass {
///     obj: JsRef<JsObject<'static>>
/// }
/// 
/// #[pinar]
/// impl MyClass {
///     fn my_func(&mut self, obj: JsObject) -> JsResult<()> {
///         self.obj = obj.as_js_ref()?;
///         Ok(())
///     }
///
///     fn retrieve_value(&self) -> JsResult<JsObject> {
///         self.obj.deref()
///     }
/// }
/// ```
#[derive(Clone)]
pub struct JsRef<T: JsValue> {
    pub(crate) inner: Rc<JsRefInner>,
    phantom: PhantomData<T>
}

/// Trait for creating a reference to a Javascript value
///
/// More information with [`JsRef`]
///
/// [`JsRef`]: ./struct.JsRef.html
pub trait AsJsRef<T: JsValue> {
    /// Method to get a reference to a Javascript value.  
    /// The reference will have the `'static` lifetime.
    /// # Example
    /// ```
    /// #[pinar]
    /// fn my_func(fun: JsFunction) -> JsResult<()> {
    ///     let ref_fun: JsRef<JsFunction<'static>> = fun.as_js_ref()?;
    ///     Ok(())
    /// }
    /// ```
    fn as_js_ref(&self) -> Result<JsRef<T>>;
}

impl<T> Drop for JsRef<T>
where
    T: JsValue
{
    fn drop(&mut self) {
        // We delete the napi reference only when there is no other
        // references
        if Rc::strong_count(&self.inner) == 1 {
            let inner = &self.inner;
            napi_call!(napi_delete_reference(
                inner.env.env(),
                inner.js_ref
            )).expect("Fail to drop a JsRef");
        }
    }
}


macro_rules! impl_jsref {
    (
        $( $jstype:ident ),*
    ) => {
        $(

            impl<'e> AsJsRef<$jstype<'static>> for $jstype<'e>
            {
                fn as_js_ref(&self) -> Result<JsRef<$jstype<'static>>> {
                    let env = self.get_value().env;
                    let mut js_ref: napi_ref = std::ptr::null_mut();

                    napi_call!(napi_create_reference(
                        env.env(),
                        self.get_value().value,
                        1,
                        &mut js_ref as *mut napi_ref
                    ))?;

                    Ok(JsRef {
                        inner: Rc::new(JsRefInner {
                            env,
                            js_ref,
                        }),
                        phantom: PhantomData
                    })
                }
            }

            impl<'a, 'e> JsRef<$jstype<'a>> {
                /// Returns the Javascript value associated to that reference
                pub fn deref(&self) -> Result<$jstype<'e>> {
                    let mut result = Value::new(self.inner.env);

                    napi_call!(napi_get_reference_value(
                        self.inner.env.env(),
                        self.inner.js_ref,
                        result.get_mut()
                    ))?;

                    Ok($jstype::from(result))
                }
            }

            impl<'e> JsValue for &$jstype<'e> {
                #[inline]
                fn get_value(&self) -> Value {
                    self.value
                }
            }

            impl<'e> JsValue for $jstype<'e> {
                #[inline]
                fn get_value(&self) -> Value {
                    self.value
                }
            }

            impl<'e> $jstype<'e> {
                #[doc(hidden)]
                #[inline]
                pub fn from(value: Value) -> Self {
                    Self { value, phantom: PhantomData }
                }

                #[inline]
                pub(crate) fn clone(&self) -> Self {
                    Self { value: self.value, phantom: PhantomData }
                }
            }

        )*
    }
}

impl_jsref!(
    JsString,
    JsObject,
    JsArray,
    JsNumber,
    JsSymbol,
    JsUndefined,
    JsFunction,
    JsExternal,
    JsNull,
    JsBoolean,
    JsBigInt
);
