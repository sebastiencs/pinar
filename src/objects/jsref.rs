
use std::marker::PhantomData;
use std::rc::Rc;

use napi_sys::*;
use crate::Result;
use crate::prelude::*;

pub(crate) struct JsRefInner {
    pub(crate) env: Env,
    pub(crate) js_ref: napi_ref,
}

#[derive(Clone)]
pub struct JsRef<T: JsValue> {
    pub(crate) inner: Rc<JsRefInner>,
    phantom: PhantomData<T>
}

pub trait AsJsRef<T: JsValue> {
    fn as_js_ref(&self) -> Result<JsRef<T>>;
}

impl<T> Drop for JsRef<T>
where
    T: JsValue
{
    fn drop(&mut self) {
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
