//#![feature(associated_type_defaults)]
//#![feature(optin_builtin_traits)]
//#![feature(specialization)]
//#![feature(tool_lints)]
#![cfg_attr(
    feature = "nightly",
    feature(core_intrinsics, unsized_locals)
)]
//#![feature(core_intrinsics)]
#![feature(unsized_locals)]
#![feature(default_type_parameter_fallback)]
#![feature(specialization)]
#![warn(
     clippy::all,
     clippy::cargo,
//     clippy::restriction,
//     clippy::pedantic,
//     clippy::nursery,
)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

//use crate::arguments::{Arguments, FromArguments};
use crate::error::ArgumentsError;
//use crate::module::__pinar_dispatch_function;
use std::collections::HashMap;
use napi_sys::*;
//use crate::module::ModuleBuilder;
use crate::objects::*;
//use crate::env::Env;
//use crate::to_js::ToJs;

pub mod status;
mod value;
mod module;
mod error;
mod jsreturn;
mod classes;
pub mod property_descriptor;
mod external;
pub mod objects;
mod env;
mod arguments;
mod function_threadsafe;
mod to_rust;
mod multi_js;
mod to_js;

#[cfg(feature = "pinar-serde")]
pub mod pinar_serde;

pub(crate) type Result<R> = std::result::Result<R, Error>;
pub type JsResult<R> = Result<R>;

pub use crate::error::Error;
pub use crate::env::Env;
pub use crate::multi_js::MultiJs;
//pub use crate::objects::*;
//pub use crate::status::Status;
pub use crate::to_js::ToJs;
pub use crate::to_rust::ToRust;
pub use crate::function_threadsafe::JsFunctionThreadSafe;
pub use crate::module::ModuleBuilder;
//pub use crate::property_descriptor::PropertyDescriptor;
pub use crate::jsreturn::JsReturn;
//pub use crate::module::__pinar_dispatch_function;
pub use crate::arguments::{FromArguments, Arguments};
pub use crate::classes::{JsClass, ClassBuilder};
//pub use crate::JsResult;
// #[doc(hidden)]
// #[cfg(feature = "pinar-serde")]
// pub use crate::pinar_serde::ser::serialize_to_js;
// #[cfg(feature = "pinar-serde")]
// pub use pinar_derive::ToJs;

pub mod prelude {
    pub use crate::env::Env;
    pub use crate::multi_js::MultiJs;
    pub use crate::objects::*;
    pub use crate::status::Status;
    pub use crate::to_js::ToJs;
    pub use crate::to_rust::ToRust;
    pub use crate::function_threadsafe::JsFunctionThreadSafe;
    pub use crate::module::ModuleBuilder;
    pub use crate::property_descriptor::PropertyDescriptor;
    pub use crate::jsreturn::JsReturn;
    //pub use crate::module::__pinar_dispatch_function;
    pub use crate::arguments::{FromArguments, Arguments};
    pub use crate::classes::{JsClass, ClassBuilder};
    pub use crate::JsResult;
    #[cfg(feature = "pinar-serde")]
    pub use crate::pinar_serde::ser::serialize_to_js;
    #[cfg(feature = "pinar-serde")]
    pub use pinar_derive::{ToJs, FromArguments};
}


use crate::pinar_serde::ser::serialize_to_js;

fn testfn(fun: JsFunction) {
    fun.call((1, "seb")).ok();
    fun.call(()).ok();
    fun.call((234, "coucou", vec![1, 2, 3])).ok();
    fun.call(vec![1, 2, 3]).ok();
    fun.call("salut").ok();
    fun.call(10).ok();
    fun.call(Box::new(91)).ok();
    fun.call((10, "a", 12, vec![1, 2, 3])).ok();
    fun.call(ABC { a: 1, b: 2, c: 3, d: TestEnum::B(1), e: None }).ok();
    fun.call((|Env| {}) as fn(Env) -> ()).ok();
}

use serde_derive::{Serialize, Deserialize};

use pinar_derive::{ToJs, FromArguments};

#[derive(Debug, Serialize, Deserialize)]
enum TestEnum {
    A(String),
    B(usize),
    C(Vec<usize>),
    D(Option<usize>),
    E(Box<usize>)
}

#[derive(Debug, Serialize, Deserialize)]
struct AR {
    s: String
}

#[derive(Debug, Serialize, Deserialize, ToJs, FromArguments)]
struct ABC {
    a: i32,
    b: i32,
    c: i32,
    d: TestEnum,
    e: Option<AR>
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

fn test9(args: ()) -> ABC {
    ABC {
        a: 10,
        b: 31,
        c: 22,
        d: TestEnum::E(Box::new(123)),
        e: None
    }
}

fn test10(_: ()) -> Box<usize> {
    Box::new(1234)
}

fn test11<'e>(args: (JsString<'e>, JsObject)) -> JsString<'e> {
    args.0
}

fn test12<'e>((env, s1, obj): (Env, JsString, JsObject)) -> JsString<'e> {
    env.string("weeesh").unwrap()
}

fn test13<'e>((env, abc): (Env, ABC)) -> ABC {
    println!("ABC: {:?}", abc);
    abc
    //env.string("weeesh").unwrap()
}

fn testab(args: String) {
}

fn test15<'e>(obj: JsObject) -> JsObject {
    obj.set("wesh", (|arg| { arg }) as fn(String) -> String);
    obj
}

// fn test16(fun: JsFunction) {
//     //fun(("1"));
//      fun(1);
//     // fun(vec![1, 2, 3]);
//     //fun();
// }

fn test17<'e>(s1: JsString, s2: JsString, vec: Vec<i64>) -> Result<()> {
    println!("FIRST: {}", s1.to_rust()?);
    println!("SECOND: {}", s2.to_rust()?);
    println!("VEC: {:?}", vec);
    Ok(())
}

fn test18() {
    println!("OK");
}

/// Register the node module
///
/// It takes a closure as parameter.
/// The closure takes a [`ModuleBuilder`] as argument
/// and returns a [`JsObject`] describing the module
///
/// # Example:
/// ```rust, no_run
/// register_module!(|module: ModuleBuilder| {
///     module.with_function("my_function1", my_fn1)
///           .with_function("my_function2", my_fn2)
///           .with_class("my_class", || {
///               ClassBuilder::<SomeClass>::start_build()
///                   .with_method("easy", SomeClass::jsfunction)
///                   .with_accessor("easy3", SomeClass::jsaccessor)
///           })
///           .build()
/// });
/// ```
#[macro_export]
macro_rules! register_module {
    ($init:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        #[cfg_attr(target_os = "linux", link_section = ".ctors")]
        #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
        #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
        pub static __REGISTER_MODULE: extern "C" fn() = {
            use napi_sys::*;

            extern "C" fn register_module() {
                static mut MODULE_DESCRIPTOR: napi_module = napi_module {
                    nm_version: 1,
                    nm_flags: 0,
                    nm_filename: std::ptr::null(),
                    nm_modname: std::ptr::null(),
                    nm_register_func: Some(init_module),
                    nm_priv: 0 as *mut _,
                    reserved: [0 as *mut _; 4],
                };

                unsafe { napi_module_register(&mut MODULE_DESCRIPTOR) };

                extern "C" fn init_module(env: napi_env, export: napi_value) -> napi_value {
                    match $init(ModuleBuilder::new(env, export)) {
                        Ok(export) => export,
                        _ => unreachable!()
                    }
                }
            }

            register_module
        };
    };
}

use crate::classes::SomeClass;
//use crate::classes::ClassBuilder;

register_module!(|module: ModuleBuilder| {
    module.with_function("test1", test1)
          .with_function("my_super_function", test2)
          .with_function("my_other_function", test3)
          .with_function("test4", test4)
          .with_function("test5", test5)
          .with_function("test6", test6)
          .with_function("test7", test7)
          .with_function("test8", test8)
          .with_function("test9", test9)
          .with_function("test10", test10)
          .with_function("test11", test11)
          .with_function("test12", test12)
          .with_function("test13", test13)
          .with_function("test14", |()| {
              1234
          })
          .with_function("test15", test15)
//          .with_function("test16", test16)
          .with_function("test17", test17)
          .with_function("test17", test18)
          .with_class("someclass", || {
              ClassBuilder::<SomeClass>::start_build()
                  .with_method("easy", SomeClass::jsfunction)
                  .with_method("easy2", SomeClass::jsother)
                  .with_accessor("easy3", SomeClass::jsaccessor)
                  // .with_accessor("easy4", SomeClass::jsbox)
          })
          .build()
});
