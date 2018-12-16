
use std::marker::PhantomData;
use crate::*;
use crate::prelude::*;
use super::*;

pub struct JsObject<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

impl<'e> JsObject<'e> {
    pub fn set<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: KeyProperty + ToJs<'e>,
        V: ToJs<'e>
    {
        let key = key.to_js(&self.value.env)?.get_value();
        let value = value.to_js(&self.value.env)?.get_value();
        unsafe {
            Status::result(napi_set_property(
                self.value.env(),
                self.value.get(),
                key.get(),
                value.get()
            ))?;
        };
        Ok(())
    }

    pub fn get<K>(&self, key: K) -> Result<JsAny<'e>>
    where
        K: KeyProperty + ToJs<'e>
    {
        let key = key.to_js(&self.value.env)?.get_value();
        let mut value = Value::new(self.value.env);
        unsafe {
            Status::result(napi_get_property(
                self.value.env(),
                self.value.get(),
                key.get(),
                value.get_mut()
            ))?;
        };
        Ok(JsAny::from(value)?)
    }

    pub fn get_property_names(&self) -> Result<Vec<String>> {
        let mut value = Value::new(self.value.env);
        unsafe {
            Status::result(napi_get_property_names(
                self.value.env(),
                self.value.get(),
                value.get_mut()
            ))?;
        };
        let array = JsArray::from(value);
        Ok(array.iter()?.filter_map(|v| v.as_string()).collect())
    }

    pub(crate) fn get_property_names_any(&self) -> Result<Vec<JsAny<'e>>> {
        let mut value = Value::new(self.value.env);
        unsafe {
            Status::result(napi_get_property_names(
                self.value.env(),
                self.value.get(),
                value.get_mut()
            ))?;
        };
        let array = JsArray::from(value);
        let res: Vec<_> = array.iter()?.map(|v| v.clone()).collect();
        Ok(unsafe { std::mem::transmute(res) })
    }

    pub fn has_property<K>(&self, key: K) -> Result<bool>
    where
        K: KeyProperty + ToJs<'e>
    {
        let mut result = false;
        let key = key.to_js(&self.value.env)?.get_value();
        unsafe {
            Status::result(napi_has_property(
                self.value.env(),
                self.value.get(),
                key.get(),
                &mut result
            ))?;
        };
        Ok(result)
    }

    pub fn has_own_property<K>(&self, key: K) -> Result<bool>
    where
        K: KeyProperty + ToJs<'e>
    {
        let mut result = false;
        let key = key.to_js(&self.value.env)?.get_value();
        unsafe {
            Status::result(napi_has_own_property(
                self.value.env(),
                self.value.get(),
                key.get(),
                &mut result
            ))?;
        };
        Ok(result)
    }

    pub fn delete_property<K>(&self, key: K) -> Result<bool>
    where
        K: KeyProperty + ToJs<'e>
    {
        let mut result = false;
        let key = key.to_js(&self.value.env)?.get_value();
        unsafe {
            Status::result(napi_delete_property(
                self.value.env(),
                self.value.get(),
                key.get(),
                &mut result
            ))?;
        };
        Ok(result)
    }

    pub fn define_properties(&self, props: impl IntoIterator<Item = PropertyDescriptor>) -> Result<()> {
        let props = props.into_iter()
                         .map(Into::into)
                         .collect::<Vec<_>>();

        unsafe {
            Status::result(napi_define_properties(
                self.value.env(),
                self.value.get(),
                props.len(),
                props.as_ptr()
            ))?;
        }

        Ok(())
    }

    pub fn define_property(&self, prop: PropertyDescriptor) -> Result<()> {
        unsafe {
            Status::result(napi_define_properties(
                self.value.env(),
                self.value.get(),
                1,
                &prop.into()
            ))?;
        }

        Ok(())
    }

    pub(crate) fn napi_unwrap<T>(&self) -> Result<*mut T> {
        let mut obj: *mut T = std::ptr::null_mut();
        unsafe {
            Status::result(napi_unwrap(
                self.value.env(),
                self.get_value().value,
                &mut obj as *mut *mut T as *mut *mut std::ffi::c_void
            ))?;
        }
        Ok(obj)
    }
}

/// - Named: a simple UTF8-encoded string
/// - Integer-Indexed: an index value represented by uint32_t
/// - JavaScript value: these are represented in N-API by napi_value.
///   This can be a napi_value representing a String, Number, or Symbol.
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
