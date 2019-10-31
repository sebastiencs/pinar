
use crate::Env;
use crate::ToJs;
use crate::Result;
use crate::JsValue;
use napi_sys::*;

// &napi_property_descriptor {
//     utf8name: name.as_ptr() as *const i8,
//     name: std::ptr::null_mut(),
//     method: None,
//     getter: None,
//     setter: None,
//     value: class_data.get_value().value,
//     attributes: napi_property_attributes::napi_default,
//     data: std::ptr::null_mut(),
// });

/// Struct to create a property on an object
#[allow(dead_code)]
pub struct PropertyDescriptor {
    name: napi_value,
    method: Option<napi_callback>,
    setter_getter: Option<napi_callback>,
    value: Option<napi_value>,
    attributes: napi_property_attributes,
    data: ()
}

impl PropertyDescriptor {
    /// Creates a method on the object
    ///
    /// Todo: Remove this exposure to n-api detail
    pub fn method<S: AsRef<str>>(env: Env, name: S, cb: napi_callback) -> Result<Self> {
        Ok(PropertyDescriptor {
            name: env.string(name.as_ref())?.get_value().value,
            method: Some(cb),
            setter_getter: None,
            value: None,
            attributes: napi_property_attributes::napi_default,
            data: ()
        })
    }

    /// Creates a setter on the object
    ///
    /// Todo: Remove this exposure to n-api detail
    pub fn set_get<S: AsRef<str>>(env: Env, name: S, cb: napi_callback) -> Result<Self> {
        Ok(PropertyDescriptor {
            name: env.string(name.as_ref())?.get_value().value,
            method: None,
            setter_getter: Some(cb),
            value: None,
            attributes: napi_property_attributes::napi_default,
            data: ()
        })
    }

    /// Creates a value on the object
    pub fn value<'e, S: AsRef<str>, V: ToJs<'e>>(env: Env, name: S, value: V) -> Result<Self> {
        Ok(PropertyDescriptor {
            name: env.string(name.as_ref())?.get_value().value,
            method: None,
            setter_getter: None,
            value: Some(value.to_js(env)?.get_value().value),
            attributes: napi_property_attributes::napi_default,
            data: ()
        })
    }
}

impl Into<napi_property_descriptor> for PropertyDescriptor {
    fn into(self) -> napi_property_descriptor {
        napi_property_descriptor {
            utf8name: std::ptr::null_mut(),
            name: self.name,
            method: if self.method.is_some() { self.method.unwrap() } else { None },
            getter: if self.setter_getter.is_some() { self.setter_getter.unwrap() } else { None },
            setter: if self.setter_getter.is_some() { self.setter_getter.unwrap() } else { None },
            value: if self.value.is_some() { self.value.unwrap() } else { std::ptr::null_mut() },
            attributes: napi_property_attributes::napi_default,
            data: std::ptr::null_mut(),
        }
    }
}
