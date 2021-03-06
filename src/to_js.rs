
use std::hash::Hash;
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::Rc;
use std::hash::BuildHasher;
use std::path::{Path, PathBuf};
use crate::prelude::*;

/// Trait to convert a Rust value to Javascript
///
/// Is it implemented for basic Rust types and users of Pinar
/// can use the derive macro [`Pinar`] to implement it.
///
/// # Example
/// ```
/// #[derive(Serialize, Deserialize, Pinar)]
/// struct MyStruct {
///     s: String,
///     n: i64
/// }
/// ```
/// [`Pinar`]: ./derive.Pinar.html
pub trait ToJs<'e> {
    type Value: JsValue;
    fn to_js(&self, _: Env) -> JsResult<Self::Value>;
}

macro_rules! impl_tojs {
    (
        $( $jstype:ident ),*
    ) => {
        $(
            impl<'e, 'v> ToJs<'e> for $jstype<'v> {
                type Value = Self;
                fn to_js(&self, _: Env) -> JsResult<Self> {
                    Ok(self.clone())
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
    fn to_js(&self, _: Env) -> JsResult<Self> {
        Ok(*self)
    }
}

impl<'e> ToJs<'e> for i64 {
    type Value = JsNumber<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsNumber<'e>> {
        env.number(*self)
    }
}

impl<'e> ToJs<'e> for bool {
    type Value = JsBoolean<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsBoolean<'e>> {
        env.boolean(*self)
    }
}

impl<'e> ToJs<'e> for PathBuf {
    type Value = JsString<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsString<'e>> {
        env.string(self.as_os_str().to_str().unwrap())
    }
}

impl<'e> ToJs<'e> for String {
    type Value = JsString<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsString<'e>> {
        env.string(self)
    }
}


#[cfg(feature = "json")]
impl<'e> ToJs<'e> for serde_json::Value {
    type Value = Value;

    fn to_js(&self, env: Env) -> JsResult<Value> {
        Ok(serialize_to_js(env, self).unwrap())
    }
}

// impl<'e, 'p> ToJs<'e> for &'p PathBuf {
//     type Value = JsString<'e>;
//     fn to_js(&self, env: Env) -> JsResult<JsString<'e>> {
//         env.string(self.as_os_str().to_str().unwrap())
//     }
// }

impl<'e, K, V, S> ToJs<'e> for HashMap<K, V, S>
where
    K: Hash + Eq + KeyProperty + ToJs<'e>,
    V: ToJs<'e>,
    S: BuildHasher
{
    type Value = JsObject<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsObject<'e>> {
        let object = env.object()?;
        for (key, value) in self.iter() {
            object.set_ref(key, value)?;
        }
        Ok(object)
    }
}

// impl<'e, A, R> ToJs<'e> for fn(A) -> R
// where
//     A: FromArguments + 'static,
//     R: for<'env> JsReturn<'env> + 'static,
// {
//     type Value = JsFunction<'e>;
//     fn to_js(&self, env: Env) -> JsResult<JsFunction<'e>> {
//         env.function("_pinar_anonymous_", self)
//     }
// }

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
//     fn to_js(&self, env: Env) -> JsResult<JsFunction<'e>> {
//         //env.function("_pinar_anonymous_", self)
//     }
// }

impl<'e, T> ToJs<'e> for std::vec::Vec<T>
where
    T: ToJs<'e>
{
    type Value = JsArray<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsArray<'e>> {
        let array = env.array_with_capacity(self.len())?;
        for (index, value) in self.iter().enumerate() {
            array.set_ref(index as u32, value)?;
        }
        Ok(array)
    }
}

impl<'e> ToJs<'e> for str {
    type Value = JsString<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsString<'e>> {
        env.string(self)
    }
}

impl<'e> ToJs<'e> for &'_ str {
    type Value = JsString<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsString<'e>> {
        env.string(self)
    }
}

// trait RefString {
//     fn as_str(&self) -> &'_ str;
// }

// impl RefString for &'_ str {
//     fn as_str(&self) -> &'_ str { self }
// }

// impl<'e, T> ToJs<'e> for T
// where
//     T: RefString
// {
//     type Value = JsString<'e>;
//     fn to_js(&self, env: Env) -> JsResult<JsString<'e>> {
//         env.string(self.as_str())
//     }

// }

// impl<'e> ToJs<'e> for str {
//     type Value = JsString<'e>;
//     fn to_js(&self, env: Env) -> JsResult<JsString<'e>> {
//         env.string(self)
//     }
// }

impl<'e, 'p> ToJs<'e> for &'p Path {
    type Value = JsString<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsString<'e>> {
        env.string(self.as_os_str().to_str().unwrap())
    }
}

// impl<'e, T: 'static> ToJs<'e> for Box<T> {
//     type Value = JsExternal<'e>;
//     fn to_js(&self, env: Env) -> JsResult<JsExternal<'e>> {
//         env.external_box(self)
//     }
// }

impl<'e, T: 'static> ToJs<'e> for Rc<T> {
    type Value = JsExternal<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsExternal<'e>> {
        env.external_rc(Rc::clone(self))
    }
}

impl<'e, T: 'static> ToJs<'e> for Arc<T> {
    type Value = JsExternal<'e>;
    fn to_js(&self, env: Env) -> JsResult<JsExternal<'e>> {
        env.external_arc(Arc::clone(self))
    }
}
