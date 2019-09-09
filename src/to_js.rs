
use std::hash::Hash;
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::Rc;
use std::hash::BuildHasher;
use std::path::PathBuf;
use crate::prelude::*;
use crate::Result;

pub trait ToJs<'e> {
    type Value: JsValue;
    fn to_js(self, _: Env) -> Result<Self::Value>;
}

macro_rules! impl_tojs {
    (
        $( $jstype:ident ),*
    ) => {
        $(
            impl<'e, 'v> ToJs<'e> for $jstype<'v> {
                type Value = Self;
                fn to_js(self, _: Env) -> Result<Self> {
                    Ok(self)
                }
            }
        )*
    }
}

impl_tojs!(
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
    JsBigInt,
    JsAny
);

impl<'e> ToJs<'e> for Value {
    type Value = Self;
    fn to_js(self, _: Env) -> Result<Self> {
        Ok(self)
    }
}

impl<'e> ToJs<'e> for i64 {
    type Value = JsNumber<'e>;
    fn to_js(self, env: Env) -> Result<JsNumber<'e>> {
        env.number(self)
    }
}

impl<'e> ToJs<'e> for bool {
    type Value = JsBoolean<'e>;
    fn to_js(self, env: Env) -> Result<JsBoolean<'e>> {
        env.boolean(self)
    }
}

impl<'e> ToJs<'e> for PathBuf {
    type Value = JsString<'e>;
    fn to_js(self, env: Env) -> Result<JsString<'e>> {
        env.string(self.as_os_str().to_str().unwrap())
    }
}

impl<'e> ToJs<'e> for String {
    type Value = JsString<'e>;
    fn to_js(self, env: Env) -> Result<JsString<'e>> {
        env.string(self)
    }
}

use crate::classes::AsJsClass;

impl<'e, C> ToJs<'e> for AsJsClass<C>
where
    C: JsClass
{
    type Value = JsObject<'e>;
    fn to_js(self, env: Env) -> Result<JsObject<'e>> {
        self.to_js_class(env)
    }
}

impl<'e, 'p> ToJs<'e> for &'p PathBuf {
    type Value = JsString<'e>;
    fn to_js(self, env: Env) -> Result<JsString<'e>> {
        env.string(self.as_os_str().to_str().unwrap())
    }
}

impl<'e, K, V, S> ToJs<'e> for HashMap<K, V, S>
where
    K: Hash + Eq + KeyProperty + ToJs<'e>,
    V: ToJs<'e>,
    S: BuildHasher
{
    type Value = JsObject<'e>;
    fn to_js(self, env: Env) -> Result<JsObject<'e>> {
        let object = env.object()?;
        for (key, value) in self.into_iter() {
            object.set(key, value)?;
        }
        Ok(object)
    }
}

impl<'e, A, R> ToJs<'e> for fn(A) -> R
where
    A: FromArguments + 'static,
    R: for<'env> JsReturn<'env> + 'static,
{
    type Value = JsFunction<'e>;
    fn to_js(self, env: Env) -> Result<JsFunction<'e>> {
        env.function("_pinar_anonymous_", self)
    }
}

// This doesn't work :(
// Will probably work with Chalk and specialization ?
//
// impl<'e, F, A, R> ToJs<'e, A, R> for F
// where
//     A: FromArguments + 'static,
//     R: for<'env> JsReturn<'env> + 'static,
//     F: Fn(A) -> R + 'static
// {
//     type Value = JsFunction<'e>;
//     fn to_js(self, env: Env) -> Result<JsFunction<'e>> {
//         //env.function("_pinar_anonymous_", self)
//     }
// }

impl<'e, T> ToJs<'e> for std::vec::Vec<T>
where
    T: ToJs<'e>
{
    type Value = JsArray<'e>;
    fn to_js(self, env: Env) -> Result<JsArray<'e>> {
        let array = env.array_with_capacity(self.len())?;
        for (index, value) in self.into_iter().enumerate() {
            array.set(index as u32, value)?;
        }
        Ok(array)
    }
}

impl<'e, 's> ToJs<'e> for &'s str {
    type Value = JsString<'e>;
    fn to_js(self, env: Env) -> Result<JsString<'e>> {
        env.string(self)
    }
}

impl<'e, T: 'static> ToJs<'e> for Box<T> {
    type Value = JsExternal<'e>;
    fn to_js(self, env: Env) -> Result<JsExternal<'e>> {
        env.external_box(self)
    }
}

impl<'e, T: 'static> ToJs<'e> for Rc<T> {
    type Value = JsExternal<'e>;
    fn to_js(self, env: Env) -> Result<JsExternal<'e>> {
        env.external_rc(self)
    }
}

impl<'e, T: 'static> ToJs<'e> for Arc<T> {
    type Value = JsExternal<'e>;
    fn to_js(self, env: Env) -> Result<JsExternal<'e>> {
        env.external_arc(self)
    }
}
