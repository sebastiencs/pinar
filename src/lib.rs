#![feature(associated_type_defaults)]

use crate::value::ValueType;
use std::hash::Hash;
use std::os::raw::c_char;
use std::marker::PhantomData;
use core::ops::Deref;
use napi_sys::*;

mod status;
mod value;
mod module;
mod downcast;

// use crate::status;

type Result<R> = std::result::Result<R, Status>;

pub struct Env {
    env: napi_env
}

impl Env {
    fn from(env: napi_env) -> Env {
        Env { env }
    }
}

pub struct Value<'e, T> {
    env: &'e Env,
    value: napi_value,
    data: PhantomData<T>
}

impl<'e, T> Deref for Value<'e, T> {
    type Target = napi_value;
    fn deref(&self) -> &napi_value {
        &self.value
    }
}

pub struct JsHandle;

pub struct JsString;
pub struct JsObject;
pub struct JsArray;
pub struct JsNumber;
pub struct JsSymbol;
pub struct JsUndefined;

impl<'e, T> Value<'e, T> {
    fn get_mut(&mut self) -> *mut napi_value {
        &mut self.value as *mut napi_value
    }

    fn into_raw(self) -> napi_value {
        self.value
    }

    fn from(env: &Env, value: napi_value) -> Value<T> {
        Value {
            env,
            value,
            data: PhantomData
        }
    }

    fn env(&self) -> napi_env {
        self.env.env
    }

    pub fn new(env: &Env) -> Value<T> {
        Value {
            env: env,
            value: unsafe { std::mem::uninitialized() },
            data: PhantomData
        }
    }

    pub fn upcast<'a>(self) -> Value<'a, JsHandle> {
        unsafe {
            std::mem::transmute(self)
        }
    }

    pub fn is_undefined(&self) -> Result<bool> {
        let mut result = false;
        let mut undefined: Value<JsUndefined> = Value::new(self.env);
        unsafe {
            Status::result(napi_get_undefined(self.env(), undefined.get_mut()))?;
            Status::result(napi_strict_equals(self.env(), *undefined, self.value, &mut result as *mut bool))?;
        };
        Ok(result)
    }

    pub fn type_of(&self) -> Result<ValueType> {
        unsafe {
            let mut result: napi_valuetype = std::mem::uninitialized();
            Status::result(napi_typeof(self.env(), self.value, &mut result as *mut napi_valuetype))?;
            Ok(ValueType::from(result))
        }
    }
}

use crate::status::Status;

impl Env {
    fn object(&self) -> Result<Value<JsObject>> {
        let mut value = Value::new(self);
        unsafe {
            Status::result(napi_create_object(self.env, value.get_mut()))?
        };
        Ok(value)
    }

    fn number<'e>(&'e self, n: i64) -> Result<Value<'e, JsNumber>> {
        let mut value = Value::new(self);
        unsafe {
            Status::result(napi_create_int64(self.env, n, value.get_mut()))?
        };
        Ok(value)
    }

    fn string<'e, S: AsRef<str>>(&'e self, s: S) -> Result<Value<JsString>> {
        let mut value = Value::new(self);
        let s = s.as_ref();
        unsafe {
            Status::result(napi_create_string_utf8(self.env, s.as_ptr() as *const c_char, s.len(), value.get_mut()))?
        };
        Ok(value)
    }

    fn array(&self) -> Result<Value<JsArray>> {
        let mut value = Value::new(self);
        unsafe {
            Status::result(napi_create_array(self.env, value.get_mut()))?
        };
        Ok(value)
    }

    fn array_with_capacity(&self, cap: usize) -> Result<Value<JsArray>> {
        let mut value = Value::new(self);
        unsafe {
            Status::result(napi_create_array_with_length(self.env, cap, value.get_mut()))?
        };
        Ok(value)
    }

    // napi_create_function(napi_env env,
    //                              const char* utf8name,
    //                              size_t length,
    //                              napi_callback cb,
    //                              void* data,
    //                              napi_value* result);

    fn function(&self) {

        unsafe {
            //napi_create_function(self.env, "some_function", 13, );
        }
    }
}

trait IntoRust<R> {
    fn into_rust(self) -> Result<R>;
}

impl<'e> IntoRust<String> for Value<'e, JsString> {
    fn into_rust(self) -> Result<String> {
        let len = self.len()?;
        let mut buffer: Vec<u8> = Vec::with_capacity(len + 1); // + '\0'
        let mut written = 0usize;
        unsafe {
            Status::result(napi_get_value_string_utf8(self.env(), self.value,
                                                      buffer.as_mut_ptr() as *mut c_char,
                                                      len + 1,
                                                      &mut written as *mut usize))?;
            buffer.set_len(written);
            // It's probably safe to assume that it's valid ut8
            Ok(String::from_utf8_unchecked(buffer))
        }
    }
}

impl<'e> IntoRust<i64> for Value<'e, JsNumber> {
    fn into_rust(self) -> Result<i64> {
        let mut number = 0i64;
        unsafe {
            Status::result(napi_get_value_int64(self.env(), self.value, &mut number as *mut i64))?;
        }
        Ok(number)
    }
}

impl<'e> IntoRust<i32> for Value<'e, JsNumber> {
    fn into_rust(self) -> Result<i32> {
        let mut number = 0i32;
        unsafe {
            Status::result(napi_get_value_int32(self.env(), self.value, &mut number as *mut i32))?;
        }
        Ok(number)
    }
}

impl<'e> IntoRust<u32> for Value<'e, JsNumber> {
    fn into_rust(self) -> Result<u32> {
        let mut number = 0u32;
        unsafe {
            Status::result(napi_get_value_int32(self.env(), self.value, &mut number as *mut u32))?;
        }
        Ok(number)
    }
}

/// - Named: a simple UTF8-encoded string
/// - Integer-Indexed: an index value represented by uint32_t
/// - JavaScript value: these are represented in N-API by napi_value.
///   This can be a napi_value representing a String, Number, or Symbol.
pub trait KeyProperty {}

impl KeyProperty for Value<'_, JsString> {}
impl KeyProperty for Value<'_, JsNumber> {}
impl KeyProperty for Value<'_, JsSymbol> {}
impl KeyProperty for &'_ str {}
impl KeyProperty for i64 {}
impl KeyProperty for i32 {}
impl KeyProperty for i16 {}
impl KeyProperty for i8 {}
impl KeyProperty for u64 {}
impl KeyProperty for u32 {}
impl KeyProperty for u16 {}
impl KeyProperty for u8 {}

trait IntoJs {
    type JsType;
    fn into_js<'e>(self, _: &'e Env) -> Result<Value<'e, Self::JsType>>;
}

impl IntoJs for i64 {
    type JsType = JsNumber;
    fn into_js<'e>(self, env: &'e Env) -> Result<Value<'e, JsNumber>> {
        env.number(self)
    }
}

impl IntoJs for String {
    type JsType = JsString;
    fn into_js<'e>(self, env: &'e Env) -> Result<Value<'e, Self::JsType>> {
        env.string(self)
    }
}

impl<K, V> IntoJs for std::collections::HashMap<K, V>
where
    K: Hash + Eq + KeyProperty + IntoHandle,
    V: IntoHandle
{
    type JsType = JsObject;
    fn into_js<'e>(self, env: &'e Env) -> Result<Value<'e, Self::JsType>> {
        let object = env.object()?;
        for (key, value) in self.into_iter() {
            object.set(key, value)?;
        }
        Ok(object)
    }
}

impl<T> IntoJs for std::vec::Vec<T>
where T: IntoHandle
{
    type JsType = JsArray;
    fn into_js<'e>(self, env: &'e Env) -> Result<Value<'e, Self::JsType>> {
        let array = env.array_with_capacity(self.len())?;
        for (index, value) in self.into_iter().enumerate() {
            array.set(index as u32, value)?;
        }
        Ok(array)
    }
}

impl<'s> IntoJs for &'s str {
    type JsType = JsString;
    fn into_js<'e>(self, env: &'e Env) -> Result<Value<'e, Self::JsType>> {
        env.string(self)
    }
}

pub trait IntoHandle {
    fn into_handle<'e>(self, _: &'e Env) -> Result<Value<'e, JsHandle>>;
}

impl<T> IntoHandle for T
where
    T: IntoJs
{
    fn into_handle<'e>(self, env: &'e Env) -> Result<Value<'e, JsHandle>> {
        Ok(self.into_js(env)?.upcast())
    }
}

impl<'a, T> IntoHandle for Value<'a, T> {
    fn into_handle<'e>(self, _: &'e Env) -> Result<Value<'e, JsHandle>> {
        Ok(self.upcast())
    }
}

impl<'e> Value<'e, JsHandle> {
    pub fn is_array(&self) -> Result<bool> {
        unsafe {
            let mut result: bool = std::mem::uninitialized();
            Status::result(napi_is_array(self.env(), self.value, &mut result as *mut bool))?;
            Ok(result)
        }
    }
}

impl<'e> Value<'e, JsString> {
    /// Returns the string length
    pub fn len(&self) -> Result<usize> {
        unsafe {
            let mut length = 0;
            Status::result(napi_get_value_string_utf8(self.env(), self.value,
                                                      std::ptr::null_mut() as *mut c_char,
                                                      0,
                                                      &mut length as *mut usize))?;
            Ok(length)
        }
    }
}

impl<'e> Value<'e, JsObject> {
    pub fn set<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: KeyProperty + IntoHandle,
        V: IntoHandle
    {
        let key = key.into_handle(self.env)?;
        let value = value.into_handle(self.env)?;
        unsafe {
            Status::result(napi_set_property(self.env(), self.value, *key, *value))?;
        };
        Ok(())
    }

    pub fn get<K>(&self, key: K) -> Result<Value<JsHandle>>
    where
        K: KeyProperty + IntoHandle
    {
        let mut value = Value::new(self.env);
        let key = key.into_handle(self.env)?;
        unsafe {
            Status::result(napi_get_property(self.env(), self.value, *key, value.get_mut()))?;
        };
        Ok(value)
    }
}

impl<'e> Value<'e, JsArray> {
    pub fn set<V>(&self, index: u32, value: V) -> Result<()>
    where
        V: IntoHandle
    {
        let value = value.into_handle(self.env)?;
        unsafe {
            Status::result(napi_set_element(self.env(), self.value, index, *value))?;
        };
        Ok(())
    }

    pub fn get(&self, index: u32) -> Result<Option<Value<JsHandle>>>
    {
        let mut value = Value::new(self.env);
        unsafe {
            Status::result(napi_get_element(self.env(), self.value, index, value.get_mut()))?;
        };
        match value.is_undefined()? {
            true => Ok(None),
            _ => Ok(Some(value))
        }
    }
}

fn test(env: &Env) {
    let obj = env.object().unwrap();

    obj.set("coucou", 1).unwrap();
    obj.get("salut").unwrap();

    let mut map = std::collections::HashMap::new();
    map.insert(1, "seb");

    let a = test;

    let mapjs = map.into_js(env).unwrap();

    //let _ = mapjs.get_handle::<JsString, _>("salut");
}

#[macro_export]
macro_rules! register_module {
    ($module_name:ident, $init:expr) => {
        #[no_mangle]
        #[cfg_attr(target_os = "linux", link_section = ".ctors")]
        #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
        #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
        pub static __REGISTER_MODULE: extern "C" fn() = {
            use napi_sys::*;

            extern "C" fn register_module() {
                unsafe {
                    static mut MODULE_DESCRIPTOR: napi_module =
                        napi_module {
                            nm_version: 1,
                            nm_flags: 0,
                            nm_filename: std::ptr::null(),
                            nm_modname: std::ptr::null(),
                            nm_register_func: Some(init_module),
                            nm_priv: 0 as *mut _,
                            reserved: [0 as *mut _; 4],
                        };
                    napi_module_register(&mut MODULE_DESCRIPTOR);
                }

                extern "C" fn init_module(env: napi_env, v: napi_value) -> napi_value {
                    // {
                    //     let env = Env::from(env);
                    //     let v: Value<JsHandle> = Value::from(&env, v);
                    //     println!("{:?}", v.type_of());
                    // }
                    {
                        let env = Env::from(env);
                        //let s = env.string("seb").unwrap();
                        let s = env.string("a𐐀b").unwrap();
                        //println!("TYPEOF: {:?}", s.type_of().unwrap());
                        println!("'{}'", s.into_rust().unwrap());

                        // let s: Value<JsObject> = unsafe { std::mem::transmute(s) };
                        // let res = s.get("length").unwrap();
                        // let mut result: i32 = 0;
                        // unsafe { napi_get_value_int32(env.env, *res, &mut result as *mut i32) };
                        // println!("LENGTH: {}", result);
                    }
                    match $init(&Env::from(env)) {
                        Ok(export) => export.into_raw(),
                        _ => std::ptr::null_mut()
                    }
                }
            }

            register_module
        };
    };
}

register_module!(sebastien, |env| {
    {
        //let s = env.string("coucou").unwrap();
    }
    let mut map = std::collections::HashMap::new();
    map.insert(1, "seb");
    map.insert(98, "sylvia");
    map.insert(93, "yana");
    map.into_js(env)
});
