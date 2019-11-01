
use std::marker::PhantomData;
use crate::*;
use crate::prelude::*;
use super::*;
use crate::error::JsClassError;

/// A Javascript object
pub struct JsObject<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

impl<'e> JsObject<'e> {
    #[doc(hidden)]
    pub fn env(&self) -> Env {
        self.value.env
    }

    /// Set a property on the object.
    /// # Example
    /// ```
    /// #[pinar]
    /// fn my_func(obj: JsObject) -> JsResult<()> {
    ///     obj.set("my_prop", vec![1, 2, 3])?;
    ///     
    ///     obj.set(4, HashMap::new())?;
    ///     
    ///     obj.set("an_external", Arc::new(HashMap::new()))?;
    ///     
    ///     Ok(())
    /// }    
    /// ```
    pub fn set<K, V>(&self, key: K, value: V) -> JsResult<()>
    where
        K: KeyProperty + ToJs<'e>,
        V: ToJs<'e>
    {
        let key = key.to_js(self.value.env)?.get_value();
        let value = value.to_js(self.value.env)?.get_value();

        napi_call!(napi_set_property(
            self.value.env(),
            self.value.get(),
            key.get(),
            value.get()
        ))?;

        Ok(())
    }

    /// Similar to `set` but takes references.
    /// Used internally only.
    pub(crate) fn set_ref<K, V>(&self, key: &K, value: &V) -> JsResult<()>
    where
        K: KeyProperty + ToJs<'e>,
        V: ToJs<'e>
    {
        let key = key.to_js(self.value.env)?.get_value();
        let value = value.to_js(self.value.env)?.get_value();

        napi_call!(napi_set_property(
            self.value.env(),
            self.value.get(),
            key.get(),
            value.get()
        ))?;

        Ok(())
    }

    /// Returns the requested property on the object.
    ///
    /// # Example
    /// ```
    /// #[pinar]
    /// fn my_func(obj: JsObject) -> JsResult<()> {
    ///     let array = obj.get("my_prop")?.as_jsarray()?;
    ///     
    ///     let o = obj.get(4)?.as_jsobject()?;
    ///     
    ///     let ext = obj.get("an_external")?.as_jsexternal()?;
    ///     
    ///     Ok(())
    /// }    
    /// ```
    pub fn get<K>(&self, key: K) -> JsResult<JsAny<'e>>
    where
        K: KeyProperty + ToJs<'e>
    {
        let key = key.to_js(self.value.env)?.get_value();
        let mut value = Value::new(self.value.env);

        napi_call!(napi_get_property(
            self.value.env(),
            self.value.get(),
            key.get(),
            value.get_mut()
        ))?;

        Ok(JsAny::from(value)?)
    }

    /// Returns the names of the enumerable properties of the object.  
    /// The properties whose key is a symbol will not be included.
    pub fn get_property_names(&self) -> JsResult<Vec<String>> {
        let mut value = Value::new(self.value.env);

        napi_call!(napi_get_property_names(
            self.value.env(),
            self.value.get(),
            value.get_mut()
        ))?;

        let array = JsArray::from(value);
        Ok(array.iter()?.filter_map(|v| v.as_string().ok()).collect())
    }

    /// Similar to `get_property_names` but returns a `Vec` of `JsAny` instead.
    pub(crate) fn get_property_names_any(&self) -> JsResult<Vec<JsAny<'e>>> {
        let mut value = Value::new(self.value.env);

        napi_call!(napi_get_property_names(
            self.value.env(),
            self.value.get(),
            value.get_mut()
        ))?;

        let array = JsArray::from(value);
        array.with_values(JsAny::from)
    }

    /// Checks if the object has the named property.
    pub fn has_property<K>(&self, key: K) -> JsResult<bool>
    where
        K: KeyProperty + ToJs<'e>
    {
        let mut result = false;
        let key = key.to_js(self.value.env)?.get_value();

        napi_call!(napi_has_property(
            self.value.env(),
            self.value.get(),
            key.get(),
            &mut result
        ))?;

        Ok(result)
    }

    /// Checks if the Object passed in has the named own property.  
    /// The key must be a string or symbol.  
    /// A Rust string will be converted to a Javascript string.  
    pub fn has_own_property<K>(&self, key: K) -> JsResult<bool>
    where
        K: OwnProperty + ToJs<'e>
    {
        let mut result = false;
        let key = key.to_js(self.value.env)?.get_value();

        napi_call!(napi_has_own_property(
            self.value.env(),
            self.value.get(),
            key.get(),
            &mut result
        ))?;

        Ok(result)
    }

    /// Delete the key own property from object
    pub fn delete_property<K>(&self, key: K) -> JsResult<bool>
    where
        K: KeyProperty + ToJs<'e>
    {
        let mut result = false;
        let key = key.to_js(self.value.env)?.get_value();

        napi_call!(napi_delete_property(
            self.value.env(),
            self.value.get(),
            key.get(),
            &mut result
        ))?;

        Ok(result)
    }

    /// Allows the efficient definition of multiple properties on the object.  
    pub fn define_properties(&self, props: impl IntoIterator<Item = PropertyDescriptor>) -> JsResult<()> {
        let props = props.into_iter()
                         .map(Into::into)
                         .collect::<Vec<_>>();

        napi_call!(napi_define_properties(
            self.value.env(),
            self.value.get(),
            props.len(),
            props.as_ptr()
        ))?;

        Ok(())
    }

    /// Define a property on the object.
    pub fn define_property(&self, prop: PropertyDescriptor) -> JsResult<()> {
        napi_call!(napi_define_properties(
            self.value.env(),
            self.value.get(),
            1,
            &prop.into()
        ))?;

        Ok(())
    }

    /// Retrieves a native instance that was previously wrapped in
    /// a JavaScript object using napi_wrap()
    pub(crate) fn napi_unwrap<T>(&self) -> JsResult<*mut T> {
        let mut obj: *mut T = std::ptr::null_mut();

        napi_call!(napi_unwrap(
            self.value.env(),
            self.get_value().value,
            &mut obj as *mut *mut T as *mut *mut std::ffi::c_void
        ))?;

        if obj.is_null() {
            return Err(JsClassError::Unwrap.into())
        }

        Ok(obj)
    }
}

/// Trait implemented with types that can be property keys
// - Named: a simple UTF8-encoded string
// - Integer-Indexed: an index value represented by uint32_t
// - JavaScript value: these are represented in N-API by napi_value.
//   This can be a napi_value representing a String, Number, or Symbol.
pub trait KeyProperty {}

impl KeyProperty for JsString<'_> {}
impl KeyProperty for JsNumber<'_> {}
impl KeyProperty for JsSymbol<'_> {}
impl KeyProperty for Value {}
impl KeyProperty for &'_ str {}
impl KeyProperty for String {}
impl KeyProperty for i64 {}
impl KeyProperty for i32 {}
impl KeyProperty for i16 {}
impl KeyProperty for i8 {}
impl KeyProperty for u64 {}
impl KeyProperty for u32 {}
impl KeyProperty for u16 {}
impl KeyProperty for u8 {}

/// Trait implemented with types that can be own property keys
pub trait OwnProperty {}
impl OwnProperty for JsString<'_> {}
impl OwnProperty for JsSymbol<'_> {}
impl OwnProperty for &'_ str {}
impl OwnProperty for String {}

