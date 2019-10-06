//#![feature(associated_type_defaults)]
//#![feature(optin_builtin_traits)]
// #![feature(specialization)]
//#![feature(tool_lints)]
// #![cfg_attr(
//     feature = "nightly",
//     feature(core_intrinsics, unsized_locals)
// )]
//#![feature(core_intrinsics)]
// #![feature(unsized_locals)]
// #![feature(default_type_parameter_fallback)]
// #![feature(specialization)]
#![warn(
     clippy::all,
     clippy::cargo,
//     clippy::restriction,
//     clippy::pedantic,
//     clippy::nursery,
)]
// #![feature(unboxed_closures)]
// #![feature(fn_traits)]

#![allow(clippy::trivially_copy_pass_by_ref)]

/// A convenient macro to call napi functions
/// Convert the result to a Status
macro_rules! napi_call {
    (
        $fun:expr
    ) => {
        // Calls to napi require unsafe
        Status::result(unsafe { $fun })
    }
}

//use crate::arguments::{Arguments, FromArguments};

//use crate::module::__pinar_dispatch_function;

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
    pub use crate::jsreturn::{JsReturn, JsReturnRef};
    //pub use crate::module::__pinar_dispatch_function;
    pub use crate::arguments::{FromArguments, Arguments};
    pub use crate::classes::{JsClass, AsJsClass, ClassBuilder};
    pub use crate::JsResult;
    #[cfg(feature = "pinar-serde")]
    pub use crate::pinar_serde::ser::serialize_to_js;
    #[cfg(feature = "pinar-serde")]
    pub use pinar_derive::Pinar;
    // #[cfg(feature = "pinar-serde")]
    // pub use pinar_derive;
    // #[cfg(feature = "pinar-serde")]
    // pub use serde::{Serialize, Deserialize};
    // pub use super::register_module;
    #[doc(hidden)]
    pub use napi_sys::{napi_env, napi_value};
    pub use crate::error::{ArgumentsError, JsAnyError};

    pub use linkme::distributed_slice;
    pub use linkme;

    pub use super::{PINAR_CLASSES,PINAR_FUNCTIONS};
    pub use pinar_derive::pinar;
    pub use super::pinar_serde;
}

use linkme::distributed_slice;

#[distributed_slice]
pub static PINAR_CLASSES: [fn(&mut ModuleBuilder)] = [..];

#[distributed_slice]
pub static PINAR_FUNCTIONS: [fn(&mut ModuleBuilder)] = [..];

use std::cell::RefCell;

thread_local! {
    pub(crate) static BACKTRACE: RefCell<Option<backtrace::Backtrace>> = RefCell::new(None);
}

#[doc(hidden)]
#[no_mangle]
#[cfg_attr(target_os = "linux", link_section = ".ctors")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
pub static PINAR_REGISTER: extern "C" fn() = __pinar_register;

extern "C" fn __pinar_register() {
    use napi_sys::*;

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

        std::panic::set_hook(Box::new(|_info| {
            let bt = backtrace::Backtrace::new();
            BACKTRACE.with(move |bt_ref| {
                *bt_ref.borrow_mut() = Some(bt);
            });
        }));

        let mut builder = ModuleBuilder::new(env, export);

        for initializer in PINAR_CLASSES {
            initializer(&mut builder);
        }

        for initializer in PINAR_FUNCTIONS {
            initializer(&mut builder);
        }

        builder.build().expect("ModuleBuilder")
    }
}
