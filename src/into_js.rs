
use crate::*;

pub trait IntoJs {
    type JsType: JsValue;
    fn into_js(self, _: &Env) -> Result<Self::JsType>;
}

impl IntoJs for i64 {
    type JsType = JsNumber;
    fn into_js(self, env: &Env) -> Result<JsNumber> {
        env.number(self)
    }
}

impl<T: 'static> IntoJs for Box<T> {
    type JsType = JsExternal;
    fn into_js(self, env: &Env) -> Result<JsExternal> {
        env.external_box(self)
    }
}

impl<T: 'static> IntoJs for Rc<T> {
    type JsType = JsExternal;
    fn into_js(self, env: &Env) -> Result<JsExternal> {
        env.external_rc(self)
    }
}

impl<T: 'static> IntoJs for Arc<T> {
    type JsType = JsExternal;
    fn into_js(self, env: &Env) -> Result<JsExternal> {
        env.external_arc(self)
    }
}

impl IntoJs for String {
    type JsType = JsString;
    fn into_js(self, env: &Env) -> Result<Self::JsType> {
        env.string(self)
    }
}

impl<K, V> IntoJs for HashMap<K, V>
where
    K: Hash + Eq + KeyProperty + AsJs,
    V: AsJs
{
    type JsType = JsObject;
    fn into_js(self, env: &Env) -> Result<Self::JsType> {
        let object = env.object()?;
        for (key, value) in self.into_iter() {
            object.set(key, value)?;
        }
        Ok(object)
    }
}

impl<T> IntoJs for std::vec::Vec<T>
where
    T: AsJs
{
    type JsType = JsArray;
    fn into_js(self, env: &Env) -> Result<Self::JsType> {
        let array = env.array_with_capacity(self.len())?;
        for (index, value) in self.into_iter().enumerate() {
            array.set(index as u32, value)?;
        }
        Ok(array)
    }
}

impl<'s> IntoJs for &'s str {
    type JsType = JsString;
    fn into_js(self, env: &Env) -> Result<Self::JsType> {
        env.string(self)
    }
}
