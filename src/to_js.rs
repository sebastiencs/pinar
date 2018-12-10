
use std::hash::Hash;
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::Rc;
use crate::prelude::*;
use crate::Result;

pub trait ToJs {
    type Value: JsValue;
    fn to_js(self, _: &Env) -> Result<Self::Value>;
}

impl ToJs for JsFunction {
    type Value = Self;
    fn to_js(self, _: &Env) -> Result<Self> {
        Ok(self)
    }
}

impl ToJs for JsArray {
    type Value = Self;
    fn to_js(self, _: &Env) -> Result<Self> {
        Ok(self)
    }
}

impl ToJs for Value {
    type Value = Self;
    fn to_js(self, _: &Env) -> Result<Self> {
        Ok(self)
    }
}

impl ToJs for JsObject {
    type Value = Self;
    fn to_js(self, _: &Env) -> Result<Self> {
        Ok(self)
    }
}

impl ToJs for i64 {
    type Value = JsNumber;
    fn to_js(self, env: &Env) -> Result<JsNumber> {
        env.number(self)
    }
}

impl ToJs for String {
    type Value = JsString;
    fn to_js(self, env: &Env) -> Result<JsString> {
        env.string(self)
    }
}

impl<K, V> ToJs for HashMap<K, V>
where
    K: Hash + Eq + KeyProperty + ToJs,
    V: ToJs
{
    type Value = JsObject;
    fn to_js(self, env: &Env) -> Result<JsObject> {
        let object = env.object()?;
        for (key, value) in self.into_iter() {
            object.set(key, value)?;
        }
        Ok(object)
    }
}

impl<T> ToJs for std::vec::Vec<T>
where
    T: ToJs
{
    type Value = JsArray;
    fn to_js(self, env: &Env) -> Result<JsArray> {
        let array = env.array_with_capacity(self.len())?;
        for (index, value) in self.into_iter().enumerate() {
            array.set(index as u32, value)?;
        }
        Ok(array)
    }
}

impl<'s> ToJs for &'s str {
    type Value = JsString;
    fn to_js(self, env: &Env) -> Result<JsString> {
        env.string(self)
    }
}

impl<T: 'static> ToJs for Box<T> {
    type Value = JsExternal;
    fn to_js(self, env: &Env) -> Result<JsExternal> {
        env.external_box(self)
    }
}

impl<T: 'static> ToJs for Rc<T> {
    type Value = JsExternal;
    fn to_js(self, env: &Env) -> Result<JsExternal> {
        env.external_rc(self)
    }
}

impl<T: 'static> ToJs for Arc<T> {
    type Value = JsExternal;
    fn to_js(self, env: &Env) -> Result<JsExternal> {
        env.external_arc(self)
    }
}
