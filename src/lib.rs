//#![feature(associated_type_defaults)]
//#![feature(optin_builtin_traits)]
//#![feature(specialization)]
//#![feature(tool_lints)]
#![feature(core_intrinsics)]

use core::marker::PhantomData;
use crate::jsreturn::JsReturn;
use crate::classes::__pinar_drop_rc;
use crate::module::ModuleFunction;
use std::sync::Arc;
use crate::external::{External, PtrKind};
use std::rc::Rc;
use crate::error::JsExternalError;
use crate::property_descriptor::PropertyDescriptor;
use crate::classes::__pinar_drop_box;
use std::ffi::c_void;
use crate::error::ArgumentsError;
use crate::error::Error;
use crate::module::dispatch_function;
use std::collections::HashMap;
use std::cell::Cell;
use crate::value::ValueType;
use std::hash::Hash;
use std::os::raw::c_char;
use napi_sys::*;
use crate::module::ModuleBuilder;
//use crate::env::Env;
use crate::objects::*;
use std::ffi::CString;
use std::any::TypeId;
use crate::env::Env;

mod status;
mod value;
mod module;
mod error;
mod jsreturn;
mod classes;
mod property_descriptor;
mod external;
mod objects;
mod env;
mod arguments;

mod prelude {
    pub use crate::env::Env;
}

// use crate::status;

type Result<R> = std::result::Result<R, Error>;
//type Result<R> = std::result::Result<R, Status>;


//#[derive(Copy, Clone)]
pub enum JsUnknown {
    String(JsString),
    Object(JsObject),
    Array(JsArray),
    Number(JsNumber),
    Symbol(JsSymbol),
    External(JsExternal),
    Function(JsFunction),
    Undefined(JsUndefined),
    Null(JsNull),
    Boolean(JsBoolean),
    BigInt(JsBigInt),
}

impl JsUnknown {
    #[inline]
    fn from(value: Value) -> Result<JsUnknown> {
        let value = match value.type_of()? {
            ValueType::Object => {
                match value.is_array()? {
                    true => JsUnknown::Array(JsArray::from(value)),
                    _ => JsUnknown::Object(JsObject::from(value))
                }
            },
            ValueType::String => JsUnknown::String(JsString::from(value)),
            ValueType::Number => JsUnknown::Number(JsNumber::from(value)),
            ValueType::External => JsUnknown::External(JsExternal::from(value)),
            ValueType::Symbol => JsUnknown::Symbol(JsSymbol::from(value)),
            ValueType::Undefined => JsUnknown::Undefined(JsUndefined::from(value)),
            ValueType::Function => JsUnknown::Function(JsFunction::from(value)),
            ValueType::Null => JsUnknown::Null(JsNull::from(value)),
            ValueType::Boolean => JsUnknown::Boolean(JsBoolean::from(value)),
            ValueType::Bigint => JsUnknown::BigInt(JsBigInt::from(value)),
        };
        Ok(value)
    }
    #[inline]
    fn clone(&self) -> JsUnknown {
        match self {
            JsUnknown::String(s) => JsUnknown::String(s.clone()),
            JsUnknown::Object(s) => JsUnknown::Object(s.clone()),
            JsUnknown::Array(s) => JsUnknown::Array(s.clone()),
            JsUnknown::Number(s) => JsUnknown::Number(s.clone()),
            JsUnknown::Symbol(s) => JsUnknown::Symbol(s.clone()),
            JsUnknown::External(e) => JsUnknown::External(e.clone()),
            JsUnknown::Function(e) => JsUnknown::Function(e.clone()),
            JsUnknown::Undefined(e) => JsUnknown::Undefined(e.clone()),
            JsUnknown::Null(e) => JsUnknown::Null(e.clone()),
            JsUnknown::Boolean(e) => JsUnknown::Boolean(e.clone()),
            JsUnknown::BigInt(e) => JsUnknown::BigInt(e.clone()),
        }
    }
}

use crate::status::Status;

trait IntoRust<R> {
    fn into_rust(&self) -> Result<R>;
}

impl IntoRust<String> for JsString
{
    fn into_rust(&self) -> Result<String> {
        let len = self.len()?;
        let mut buffer: Vec<u8> = Vec::with_capacity(len + 1); // + '\0'
        let mut written = 0usize;
        unsafe {
            Status::result(napi_get_value_string_utf8(
                self.value.env(),
                self.value.get(),
                buffer.as_mut_ptr() as *mut c_char,
                len + 1,
                &mut written as *mut usize
            ))?;
            buffer.set_len(written);
            // It's probably safe to assume that it's valid ut8
            Ok(String::from_utf8_unchecked(buffer))
        }
    }
}

impl IntoRust<i64> for JsNumber {
    fn into_rust(&self) -> Result<i64> {
        let mut number = 0i64;
        unsafe {
            Status::result(napi_get_value_int64(
                self.value.env(),
                self.value.get(),
                &mut number as *mut i64
            ))?;
        }
        Ok(number)
    }
}

impl IntoRust<i32> for JsNumber {
    fn into_rust(&self) -> Result<i32> {
        let mut number = 0i32;
        unsafe {
            Status::result(napi_get_value_int32(
                self.value.env(),
                self.value.get(),
                &mut number as *mut i32
            ))?;
        }
        Ok(number)
    }
}

impl IntoRust<u32> for JsNumber {
    fn into_rust(&self) -> Result<u32> {
        let mut number = 0u32;
        unsafe {
            Status::result(napi_get_value_uint32(
                self.value.env(),
                self.value.get(),
                &mut number as *mut u32
            ))?;
        }
        Ok(number)
    }
}

impl IntoRust<f64> for JsNumber {
    fn into_rust(&self) -> Result<f64> {
        let mut number = 0f64;
        unsafe {
            Status::result(napi_get_value_double(
                self.value.env(),
                self.value.get(),
                &mut number as *mut f64
            ))?;
        }
        Ok(number)
    }
}

impl IntoRust<bool> for JsBoolean {
    fn into_rust(&self) -> Result<bool> {
        let mut result = false;
        unsafe {
            Status::result(napi_get_value_bool(
                self.value.env(),
                self.value.get(),
                &mut result as *mut bool
            ))?;
        }
        Ok(result)
    }
}

impl<T: 'static> IntoRust<Arc<T>> for JsExternal
{
    fn into_rust(&self) -> Result<Arc<T>> {
        self.get_arc()
    }
}

impl<T: 'static> IntoRust<Rc<T>> for JsExternal
{
    fn into_rust(&self) -> Result<Rc<T>> {
        self.get_rc()
    }
}

impl<T: 'static> IntoRust<Box<T>> for JsExternal
{
    fn into_rust(&self) -> Result<Box<T>> {
        match self.take_box()? {
            Some(b) => Ok(b),
            _ => panic!("Extracting a box from a JsExternal that have
already been extracted. Use Option<Box<T>>")
        }
    }
}

// trait IntoRust {
//     type Result;
//     fn into_rust(&self) -> Result<Self::Result>;
// }

// impl IntoRust for JsString
// {
//     type Result = String;
//     fn into_rust(&self) -> Result<Self::Result> {
//         let len = self.len()?;
//         let mut buffer: Vec<u8> = Vec::with_capacity(len + 1); // + '\0'
//         let mut written = 0usize;
//         unsafe {
//             Status::result(napi_get_value_string_utf8(self.value.env(), self.value.get(),
//                                                       buffer.as_mut_ptr() as *mut c_char,
//                                                       len + 1,
//                                                       &mut written as *mut usize))?;
//             buffer.set_len(written);
//             // It's probably safe to assume that it's valid ut8
//             Ok(String::from_utf8_unchecked(buffer))
//         }
//     }
// }

// impl IntoRust for JsNumber {
//     type Result = i64;
//     fn into_rust(&self) -> Result<i64> {
//         let mut number = 0i64;
//         unsafe {
//             Status::result(napi_get_value_int64(self.value.env(), self.value.get(), &mut number as *mut i64))?;
//         }
//         Ok(number)
//     }
// }

// impl IntoRust for JsBoolean {
//     type Result = bool;
//     fn into_rust(&self) -> Result<Self::Result> {
//         let mut result = false;
//         unsafe {
//             Status::result(napi_get_value_bool(
//                 self.value.env(),
//                 self.value.get(),
//                 &mut result as *mut bool
//             ))?;
//         }
//         Ok(result)
//     }
// }

pub trait MultiJs {
    type Result: IntoIterator<Item = Value>;
    fn make_iter(self, env: &Env) -> Result<Self::Result>;
}

impl MultiJs for ()
{
    type Result = Vec<Value>;
    fn make_iter(self, env: &Env) -> Result<Self::Result> {
        Ok(vec![])
    }
}

impl<A> MultiJs for A
where
    A: AsJs,
{
    type Result = Vec<Value>;
    fn make_iter(self, env: &Env) -> Result<Self::Result> {
        Ok(vec![
            self.as_js(env)?.get_value(),
        ])
    }
}

impl<A> MultiJs for (A,)
where
    A: AsJs,
{
    type Result = Vec<Value>;
    fn make_iter(self, env: &Env) -> Result<Self::Result> {
        Ok(vec![
            self.0.as_js(env)?.get_value(),
        ])
    }
}

impl<A, B> MultiJs for (A, B)
where
    A: AsJs,
    B: AsJs,
{
    type Result = Vec<Value>;
    fn make_iter(self, env: &Env) -> Result<Self::Result> {
        Ok(vec![
            self.0.as_js(env)?.get_value(),
            self.1.as_js(env)?.get_value(),
        ])
    }
}

impl<A, B, C> MultiJs for (A, B, C)
where
    A: AsJs,
    B: AsJs,
    C: AsJs,
{
    type Result = Vec<Value>;
    fn make_iter(self, env: &Env) -> Result<Self::Result> {
        Ok(vec![
            self.0.as_js(env)?.get_value(),
            self.1.as_js(env)?.get_value(),
            self.2.as_js(env)?.get_value(),
        ])
    }
}

pub trait AsJs {
    type JsType: JsValue;
    fn as_js(self, _: &Env) -> Result<Self::JsType>;
}

impl<T> AsJs for T
where
    T: IntoJs
{
    type JsType = T::JsType;
    fn as_js(self, env: &Env) -> Result<Self::JsType> {
        self.into_js(env)
    }
}

impl AsJs for JsFunction {
    type JsType = Self;
    fn as_js(self, _: &Env) -> Result<Self::JsType> {
        Ok(self)
    }
}

impl AsJs for JsObject {
    type JsType = Self;
    fn as_js(self, _: &Env) -> Result<Self::JsType> {
        Ok(self)
    }
}

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




fn testfn(fun: JsFunction) {
    fun.call((1, "seb")).ok();
    fun.call(()).ok();
    fun.call((234, "coucou", vec![1, 2, 3])).ok();
}

fn test1(args: (Env, String)) {
    println!("TEST1 CALLED with {}", args.1);
}

fn test2(args: (String, i64)) -> i64 {
    println!("TEST2 CALLED with {} {}", args.0, args.1);
    100
}

fn test3(args: (String, i64, Option<String>)) -> String {
    println!("TEST3 CALLED with {} {} {:?}", args.0, args.1, args.2);
    "C'est moi le boss".to_owned()
}

fn test4(args: (String, i64, Option<String>)) -> Option<String> {
    println!("TEST4 CALLED with {} {} {:?}", args.0, args.1, args.2);
    None
}

fn test5(args: (String, i64, Option<String>)) -> Vec<i64> {
    println!("TEST5 CALLED with {} {} {:?}", args.0, args.1, args.2);
    vec![1, 2, 3, 4, 5, 6]
}

fn test6(args: (String, i64, Option<String>)) -> HashMap<i64, String> {
    println!("TEST6 CALLED with {} {} {:?}", args.0, args.1, args.2);
    let mut map = HashMap::new();
    map.insert(1, "coucou".to_owned());
    map.insert(2, "salut".to_owned());
    map.insert(3, "oklm".to_owned());
    map
}

fn test7(args: i64) -> i64 {
    args + 1
}

fn test8(args: Option<i64>) -> Result<Option<i64>> {
    println!("ARGS: {:?}", args);
    Ok(args)
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

                extern "C" fn init_module(env: napi_env, export: napi_value) -> napi_value {
                    match $init(ModuleBuilder::new(env, export)) {
                        Ok(export) => export,
                        _ => unreachable!()
                    }
                }
            }

            register_module
        };

        unsafe extern "C" fn callback_function(env: napi_env, info: napi_callback_info) -> napi_value {
            dispatch_function(env, info)
        }
    };
}

use crate::classes::SomeClass;
use crate::classes::ClassBuilder;

register_module!(sebastien, |module: ModuleBuilder| {
    module.with_function("test1", test1)
          .with_function("my_super_function", test2)
          .with_function("my_other_function", test3)
          .with_function("test4", test4)
          .with_function("test5", test5)
          .with_function("test6", test6)
          .with_function("test7", test7)
          .with_function("test8", test8)
          .with_class("someclass", || {
              ClassBuilder::<SomeClass>::start_build()
                  .with_method("easy", SomeClass::jsfunction)
                  .with_method("easy2", SomeClass::jsother)
                  .with_accessor("easy3", SomeClass::jsaccessor)
                  .with_accessor("easy4", SomeClass::jsbox)
          })
          .build()
});
