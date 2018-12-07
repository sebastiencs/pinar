//#![feature(associated_type_defaults)]
//#![feature(optin_builtin_traits)]
//#![feature(specialization)]
//#![feature(tool_lints)]
#![feature(core_intrinsics)]

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
use std::ffi::CString;
use std::any::TypeId;

mod status;
mod value;
mod module;
mod downcast;
mod error;
mod jsreturn;
mod classes;
mod property_descriptor;
mod external;

// use crate::status;

type Result<R> = std::result::Result<R, Error>;
//type Result<R> = std::result::Result<R, Status>;

#[derive(Copy, Clone)]
struct EnvInner {
    env: napi_env
}

pub struct JsRef<T: JsValue> {
    inner: T
}

impl<T: JsValue> JsRef<T> {
    pub fn new(obj: T) -> JsRef<T> {
        JsRef { inner: obj }
    }
}

#[derive(Copy, Clone)]
pub struct Env {
    inner: EnvInner
}

impl Env {
    fn from(env: napi_env) -> Env {
        Env { inner: EnvInner { env } }
    }
}

#[derive(Copy, Clone)]
pub struct Value {
    env: Env,
    value: napi_value
}

impl Value {
    fn new(env: Env) -> Value {
        Value { env, value: std::ptr::null_mut() }
    }
    fn from(env: Env, value: napi_value) -> Value {
        Value { env, value }
    }
    fn get_mut(&mut self) -> *mut napi_value {
        &mut self.value as *mut napi_value
    }
    fn get(&self) -> napi_value {
        self.value
    }
    fn type_of(&self) -> Result<ValueType> {
        unsafe {
            let mut result: napi_valuetype = std::mem::zeroed();
            Status::result(napi_typeof(self.env.inner(), self.value, &mut result as *mut napi_valuetype))?;
            Ok(ValueType::from(result))
        }
    }
    pub fn is_array(&self) -> Result<bool> {
        unsafe {
            let mut result: bool = false;
            Status::result(napi_is_array(self.env.inner(), self.value, &mut result as *mut bool))?;
            Ok(result)
        }
    }
}

pub struct JsString {
    value: Value
}
pub struct JsObject {
    value: Value
}
pub struct JsArray {
    value: Value
}
pub struct JsNumber {
    value: Value
}
pub struct JsSymbol {
    value: Value
}
pub struct JsUndefined {
    value: Value
}
pub struct JsFunction {
    value: Value
}
pub struct JsExternal {
    value: Value
}

// trait Father {}
pub trait JsValue {
    fn get_value(&self) -> Value;
}

impl JsValue for JsString {
    fn get_value(&self) -> Value {
        self.value
    }
}
impl JsValue for JsObject {
    fn get_value(&self) -> Value {
        self.value
    }
}
impl JsValue for JsArray {
    fn get_value(&self) -> Value {
        self.value
    }
}
impl JsValue for JsNumber {
    fn get_value(&self) -> Value {
        self.value
    }
}
impl JsValue for JsSymbol {
    fn get_value(&self) -> Value {
        self.value
    }
}
impl JsValue for JsUndefined {
    fn get_value(&self) -> Value {
        self.value
    }
}
impl JsValue for JsFunction {
    fn get_value(&self) -> Value {
        self.value
    }
}
impl JsValue for JsExternal {
    fn get_value(&self) -> Value {
        self.value
    }
}

//#[derive(Copy, Clone)]
pub enum JsUnknown {
    String(JsString),
    Object(JsObject),
    Array(JsArray),
    Number(JsNumber),
    Symbol(JsSymbol),
    External(JsExternal),
    //Function(JsFunction),
}

impl JsUnknown {
    #[inline]
    fn from(value: Value) -> Result<JsUnknown> {
        let value = match value.type_of()? {
            ValueType::Object => JsUnknown::Object(JsObject::from(value)),
            ValueType::String => JsUnknown::String(JsString::from(value)),
            ValueType::Number => JsUnknown::Number(JsNumber::from(value)),
            ValueType::External => JsUnknown::External(JsExternal::from(value)),
            //ValueType::Symbol => JsUnknown::Symbol(JsSymbol::from(value)),
            _ => {
                if value.is_array()? {
                    JsUnknown::Array(JsArray::from(value))
                } else {
                    panic!()
                }
            }
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
        }
    }
}

impl JsString {
    #[inline]
    fn from(value: Value) -> JsString {
        JsString { value }
    }
    #[inline]
    fn clone(&self) -> JsString {
        JsString { value: self.value }
    }
}

impl JsObject {
    #[inline]
    fn from(value: Value) -> JsObject {
        JsObject { value }
    }
    #[inline]
    fn clone(&self) -> JsObject {
        JsObject { value: self.value }
    }
}

impl JsArray {
    #[inline]
    fn from(value: Value) -> JsArray {
        JsArray { value }
    }
    #[inline]
    fn clone(&self) -> JsArray {
        JsArray { value: self.value }
    }
}

impl JsNumber {
    #[inline]
    fn from(value: Value) -> JsNumber {
        JsNumber { value }
    }
    #[inline]
    fn clone(&self) -> JsNumber {
        JsNumber { value: self.value }
    }
}

impl JsSymbol {
    #[inline]
    fn from(value: Value) -> JsSymbol {
        JsSymbol { value }
    }
    #[inline]
    fn clone(&self) -> JsSymbol {
        JsSymbol { value: self.value }
    }
}

impl JsFunction {
    #[inline]
    fn from(value: Value) -> JsFunction {
        JsFunction { value }
    }
    #[inline]
    fn clone(&self) -> JsFunction {
        JsFunction { value: self.value }
    }
}

impl JsUndefined {
    #[inline]
    fn from(value: Value) -> JsUndefined {
        JsUndefined { value }
    }
    #[inline]
    fn clone(&self) -> JsUndefined {
        JsUndefined { value: self.value }
    }
}

impl JsExternal {
    #[inline]
    fn from(value: Value) -> JsExternal {
        JsExternal { value }
    }
    #[inline]
    fn clone(&self) -> JsExternal {
        JsExternal { value: self.value }
    }
}

use crate::status::Status;

impl Env {
    fn inner(&self) -> napi_env {
        self.inner.env
    }

    fn object(&self) -> Result<JsObject> {
        let mut value = Value::new(*self);
        unsafe {
            Status::result(napi_create_object(self.inner(), value.get_mut()))?
        };
        Ok(JsObject::from(value))
    }

    fn number(&self, n: i64) -> Result<JsNumber> {
        let mut value = Value::new(*self);
        unsafe {
            Status::result(napi_create_int64(self.inner(), n, value.get_mut()))?
        };
        Ok(JsNumber::from(value))
    }

    fn string<S: AsRef<str>>(&self, s: S) -> Result<JsString> {
        let mut value = Value::new(*self);
        let s = s.as_ref();
        unsafe {
            Status::result(napi_create_string_utf8(self.inner(), s.as_ptr() as *const c_char, s.len(), value.get_mut()))?
        };
        Ok(JsString::from(value))
    }

    fn array(&self) -> Result<JsArray> {
        let mut value = Value::new(*self);
        unsafe {
            Status::result(napi_create_array(self.inner(), value.get_mut()))?
        };
        Ok(JsArray::from(value))
    }

    fn array_with_capacity(&self, cap: usize) -> Result<JsArray> {
        let mut value = Value::new(*self);
        unsafe {
            Status::result(napi_create_array_with_length(self.inner(), cap, value.get_mut()))?
        };
        Ok(JsArray::from(value))
    }

    pub fn global(&self) -> Result<JsObject> {
        let mut global = Value::new(*self);
        Status::result(unsafe {
            napi_get_global(self.inner(), global.get_mut())
        })?;
        Ok(JsObject::from(global))
    }

    pub fn undefined(&self) -> Result<JsUndefined> {
        let mut undefined = Value::new(*self);
        Status::result(unsafe {
            napi_get_undefined(self.inner(), undefined.get_mut())
        })?;
        Ok(JsUndefined::from(undefined))
    }

    pub fn external<T>(&self, ptr: *mut T) -> Result<JsExternal> {
        let mut external = Value::new(*self);
        unsafe {
            Status::result(napi_create_external(self.inner(),
                                                ptr as *mut c_void,
                                                Some(__pinar_drop_box::<T>),
                                                std::ptr::null_mut(),
                                                external.get_mut()))?;
        }
        Ok(JsExternal::from(external))
    }

    pub fn external_rc<T: 'static>(&self, ptr: Rc<T>) -> Result<JsExternal> {
        let mut result = Value::new(*self);
        let external = Box::new(External::new_rc(ptr));
        unsafe {
            Status::result(napi_create_external(self.inner(),
                                                Box::into_raw(external) as *mut c_void,
                                                Some(__pinar_drop_box::<External<T>>),
                                                std::ptr::null_mut(),
                                                result.get_mut()))?;
        }
        Ok(JsExternal::from(result))
    }

    pub fn external_arc<T: 'static>(&self, ptr: Arc<T>) -> Result<JsExternal> {
        let mut result = Value::new(*self);
        let external = Box::new(External::new_arc(ptr));
        unsafe {
            Status::result(napi_create_external(self.inner(),
                                                Box::into_raw(external) as *mut c_void,
                                                Some(__pinar_drop_box::<External<T>>),
                                                std::ptr::null_mut(),
                                                result.get_mut()))?;
        }
        Ok(JsExternal::from(result))
    }

    pub fn function<N, F, A, R>(&self, name: N, fun: F) -> Result<JsFunction>
    where
        N: AsRef<str>,
        A: FromArguments + 'static,
        R: JsReturn + 'static,
        F: Fn(A) -> R + 'static
    {
        let name = name.as_ref();
        let data = Rc::new(ModuleFunction::new(name, fun));
        self.function_internal(name, data)
    }

    pub(crate) fn function_internal<N>(&self, name: N, fun: Rc<ModuleFunction>) -> Result<JsFunction>
    where
        N: AsRef<str>
    {
        let mut result = Value::new(*self);
        let name = name.as_ref();
        let raw = Rc::into_raw(fun);
        unsafe {
            Status::result(napi_create_function(self.inner.env,
                                                name.as_ptr() as *const i8,
                                                name.len(),
                                                Some(callback_function),
                                                raw as *mut std::ffi::c_void,
                                                result.get_mut()))?;
            Status::result(napi_add_finalizer(self.inner.env,
                                              result.get(),
                                              raw as *mut std::ffi::c_void,
                                              Some(__pinar_drop_rc::<ModuleFunction>),
                                              std::ptr::null_mut(),
                                              std::ptr::null_mut()))?;
        }
        Ok(JsFunction::from(result))
    }

    pub(crate) fn callback_info<D>(&self, info: napi_callback_info) -> Result<(*mut D, Arguments)> {
        let mut argc: usize = 12;
        let mut argv: Vec<napi_value> = Vec::with_capacity(argc);
        let mut this = Value::new(*self);
        let mut data_ptr: *mut D = std::ptr::null_mut();
        unsafe {
            Status::result(napi_get_cb_info(self.inner.env,
                                            info,
                                            &mut argc as *mut usize,
                                            argv.as_mut_ptr() as *mut napi_value,
                                            this.get_mut(),
                                            std::mem::transmute(&mut data_ptr)))?;
            argv.set_len(argc);
        }
        let this = match JsUnknown::from(this)? {
            JsUnknown::Object(this) => this,
            _ => unreachable!() // TODO: Is 'this' always an object ?
        };
        Ok((data_ptr, Arguments::new(*self, this, &argv)?))
    }

    pub fn throw<V>(&self, error: V) -> Result<()>
    where
        V: AsJs
    {
        let error = error.as_js(&self)?.get_value();
        unsafe {
            Status::result(napi_throw(self.inner.env, error.get()))?;
        }
        Err(Status::PendingException.into())
    }

    pub fn throw_error<M>(&self, msg: M, code: Option<String>) -> Result<()>
    where
        M: AsRef<str>
    {
        let msg = CString::new(msg.as_ref()).unwrap();
        let code = code.map(|c| CString::new(c).unwrap());
        let code_ptr = match code.as_ref() {
            Some(code) => code.as_ptr(),
            _ => std::ptr::null()
        };
        unsafe {
            Status::result(napi_throw_error(self.inner.env, code_ptr, msg.as_ptr()))?;
        }
        Err(Status::PendingException.into())
    }

    pub fn throw_type_error<M>(&self, msg: M, code: Option<&str>) -> Result<()>
    where
        M: AsRef<str>
    {
        let msg = CString::new(msg.as_ref()).unwrap();
        let code = code.map(|c| CString::new(c).unwrap());
        let code_ptr = match code.as_ref() {
            Some(code) => code.as_ptr(),
            _ => std::ptr::null()
        };
        unsafe {
            Status::result(napi_throw_type_error(self.inner.env, code_ptr, msg.as_ptr()))?;
        }
        Err(Status::PendingException.into())
    }

    pub fn throw_range_error<M>(&self, msg: M, code: Option<&str>) -> Result<()>
    where
        M: AsRef<str>
    {
        let msg = CString::new(msg.as_ref()).unwrap();
        let code = code.map(|c| CString::new(c).unwrap());
        let code_ptr = match code.as_ref() {
            Some(code) => code.as_ptr(),
            _ => std::ptr::null()
        };
        unsafe {
            Status::result(napi_throw_range_error(self.inner.env, code_ptr, msg.as_ptr()))?;
        }
        Err(Status::PendingException.into())
    }

    // pub(crate) fn wrap<C>(&self, obj: &mut Value, native: &mut C) -> Result<JsRef<JsFunction>> {
    //     let mut result = Value::new(*self);
    //     unsafe {
    //         // napi_status napi_wrap(napi_env env,
    //         //           napi_value js_object,
    //         //           void* native_object,
    //         //           napi_finalize finalize_cb,
    //         //           void* finalize_hint,
    //         //           napi_ref* result);
    //         Status::result(napi_wrap(self.inner(),
    //                                  obj.value,
    //                                  std::mem::transmute(native),
    //                                  None,
    //                                  std::ptr::null_mut(),
    //                                  std::mem::transmute(result.get_mut())))?;
    //     };
    //     println!("OKLM", );
    //     Ok(JsRef::new(JsFunction::from(result)))
    // }
}

pub struct Arguments {
    env: Env,
    args: Vec<JsUnknown>,
    this: JsObject,
//    cb_key: usize,
    current_arg: Cell<usize>
}

impl Arguments {
    pub(crate) fn new(env: Env, this: JsObject, args: &[napi_value]) -> Result<Arguments> {
        Ok(Arguments {
            env,
            this,
            //cb_key,
            args: {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    let value = Value::from(env, *arg);
                    values.push(JsUnknown::from(value)?);
                }
                values
            },
            current_arg: Cell::new(0)
        })
    }

    // pub(crate) fn new_class<T>(env: Env, data_ptr: T, this: JsObject, args: &[napi_value]) -> Result<Arguments<T>> {
    //     Ok(Arguments {
    //         env,
    //         this,
    //         cb_key: data_ptr,
    //         args: {
    //             let mut values = Vec::with_capacity(args.len());
    //             for arg in args {
    //                 let value = Value::from(env, *arg);
    //                 values.push(JsUnknown::from(value)?);
    //             }
    //             values
    //         },
    //         current_arg: Cell::new(0)
    //     })
    // }

    pub fn this(&self) -> JsObject {
        self.this.clone()
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn arg_number(&self) -> usize {
        self.current_arg.get()
    }

    pub fn next_arg(&self) -> Option<JsUnknown> {
        let current = self.current_arg.get();
        self.current_arg.set(current + 1);
        self.args.get(current).map(|v| v.clone())
    }
}

pub trait FromArguments: Sized {
    fn from_args(args: &Arguments) -> Result<Self>;
}

impl<A> FromArguments for (A,)
where
    A: FromArguments
{
    fn from_args(args: &Arguments) -> Result<Self> {
        Ok((A::from_args(args)?,))
    }
}

impl<A, B> FromArguments for (A, B)
where
    A: FromArguments,
    B: FromArguments
{
    fn from_args(args: &Arguments) -> Result<Self> {
        let res_1 = A::from_args(args)?;
        let res_2 = B::from_args(args)?;
        Ok((res_1, res_2))
    }
}

impl<A, B, C> FromArguments for (A, B, C)
where
    A: FromArguments,
    B: FromArguments,
    C: FromArguments
{
    fn from_args(args: &Arguments) -> Result<Self> {
        let res_1 = A::from_args(args)?;
        let res_2 = B::from_args(args)?;
        let res_3 = C::from_args(args)?;
        Ok((res_1, res_2, res_3))
    }
}

impl<A, B, C, D> FromArguments for (A, B, C, D)
where
    A: FromArguments,
    B: FromArguments,
    C: FromArguments,
    D: FromArguments
{
    fn from_args(args: &Arguments) -> Result<Self> {
        let res_1 = A::from_args(args)?;
        let res_2 = B::from_args(args)?;
        let res_3 = C::from_args(args)?;
        let res_4 = D::from_args(args)?;
        Ok((res_1, res_2, res_3, res_4))
    }
}

impl<A> FromArguments for Option<A>
where
    A: FromArguments
{
    fn from_args(args: &Arguments) -> Result<Self> {
        match A::from_args(args) {
            Ok(arg) => Ok(Some(arg)),
            Err(e) => {
                if let Some(ArgumentsError::Missing(_)) = e.downcast_ref::<ArgumentsError>() {
                    return Ok(None);
                }
                Err(e)
            }
        }
    }
}

impl FromArguments for JsString {
    fn from_args(args: &Arguments) -> Result<Self> {
        match args.next_arg() {
            Some(JsUnknown::String(s)) => Ok(s),
            Some(_) => Err(ArgumentsError::wrong_type("string", args.arg_number())),
            _ => Err(ArgumentsError::missing(args.arg_number()))
        }
    }
}

impl FromArguments for String {
    fn from_args(args: &Arguments) -> Result<Self> {
        match args.next_arg() {
            Some(JsUnknown::String(s)) => s.into_rust(),
            Some(_) => Err(ArgumentsError::wrong_type("string", args.arg_number())),
            _ => Err(ArgumentsError::missing(args.arg_number()))
        }
    }
}

impl FromArguments for Env {
    fn from_args(args: &Arguments) -> Result<Self> {
        Ok(args.env.clone())
    }
}

impl FromArguments for i64 {
    fn from_args(args: &Arguments) -> Result<Self> {
        match args.next_arg() {
            Some(JsUnknown::Number(n)) => n.into_rust(),
            Some(_) => Err(ArgumentsError::wrong_type("number", args.arg_number())),
            _ => Err(ArgumentsError::missing(args.arg_number()))
        }
    }
}

impl FromArguments for JsNumber {
    fn from_args(args: &Arguments) -> Result<Self> {
        match args.next_arg() {
            Some(JsUnknown::Number(n)) => Ok(n),
            Some(_) => Err(ArgumentsError::wrong_type("number", args.arg_number())),
            _ => Err(ArgumentsError::missing(args.arg_number()))
        }
    }
}

impl FromArguments for JsSymbol {
    fn from_args(args: &Arguments) -> Result<Self> {
        match args.next_arg() {
            Some(JsUnknown::Symbol(s)) => Ok(s),
            Some(_) => Err(ArgumentsError::wrong_type("symbol", args.arg_number())),
            _ => Err(ArgumentsError::missing(args.arg_number()))
        }
    }
}

trait IntoRust {
    type Result;
    fn into_rust(&self) -> Result<Self::Result>;
}

impl IntoRust for JsString
{
    type Result = String;
    fn into_rust(&self) -> Result<Self::Result> {
        let len = self.len()?;
        let mut buffer: Vec<u8> = Vec::with_capacity(len + 1); // + '\0'
        let mut written = 0usize;
        unsafe {
            Status::result(napi_get_value_string_utf8(self.value.env.inner(), self.value.get(),
                                                      buffer.as_mut_ptr() as *mut c_char,
                                                      len + 1,
                                                      &mut written as *mut usize))?;
            buffer.set_len(written);
            // It's probably safe to assume that it's valid ut8
            Ok(String::from_utf8_unchecked(buffer))
        }
    }
}

impl IntoRust for JsNumber {
    type Result = i64;
    fn into_rust(&self) -> Result<i64> {
        let mut number = 0i64;
        unsafe {
            Status::result(napi_get_value_int64(self.value.env.inner(), self.value.get(), &mut number as *mut i64))?;
        }
        Ok(number)
    }
}

/// - Named: a simple UTF8-encoded string
/// - Integer-Indexed: an index value represented by uint32_t
/// - JavaScript value: these are represented in N-API by napi_value.
///   This can be a napi_value representing a String, Number, or Symbol.
pub trait KeyProperty {}

impl KeyProperty for JsString {}
impl KeyProperty for JsNumber {}
impl KeyProperty for JsSymbol {}
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

impl<T> IntoJs for Box<T> {
    type JsType = JsExternal;
    fn into_js(self, env: &Env) -> Result<JsExternal> {
        env.external(Box::into_raw(self))
    }
}

impl<T: 'static> IntoJs for Rc<T> {
    type JsType = JsExternal;
    fn into_js(self, env: &Env) -> Result<JsExternal> {
        env.external_rc(self)
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

impl JsString {
    /// Returns the string length
    pub fn len(&self) -> Result<usize> {
        unsafe {
            let mut length = 0;
            Status::result(napi_get_value_string_utf8(self.value.env.inner(),
                                                      self.value.get(),
                                                      std::ptr::null_mut() as *mut c_char,
                                                      0,
                                                      &mut length as *mut usize))?;
            Ok(length)
        }
    }
}

impl JsExternal {
    fn get_external<T>(&self) -> Result<*mut T> {
        let mut external: *mut T = std::ptr::null_mut();
        unsafe {
            napi_get_value_external(self.value.env.inner(),
                                    self.get_value().value,
                                    std::mem::transmute(&mut external));
        }
        if external.is_null() {
            return Err(JsExternalError.into())
        }
        Ok(external)
    }

    pub fn take_box<T: 'static>(&self) -> Result<Option<Box<T>>> {
        let mut external: *mut External<T> = std::ptr::null_mut();
        unsafe {
            napi_get_value_external(self.value.env.inner(),
                                    self.get_value().value,
                                    std::mem::transmute(&mut external));
        }
        if external.is_null() {
            return Err(JsExternalError.into())
        }
        Ok(unsafe { (*external).take_box::<T>() })
    }

    pub fn get_rc<T: 'static>(&self) -> Result<Rc<T>> {
        let mut external: *mut External<T> = std::ptr::null_mut();
        unsafe {
            napi_get_value_external(self.value.env.inner(),
                                    self.get_value().value,
                                    std::mem::transmute(&mut external));
        }
        if external.is_null() {
            return Err(JsExternalError.into())
        }
        Ok(unsafe { (*external).get_rc::<T>() })
    }

    pub fn get_arc<T: 'static>(&self) -> Result<Arc<T>> {
        let mut external: *mut External<T> = std::ptr::null_mut();
        unsafe {
            napi_get_value_external(self.value.env.inner(),
                                    self.get_value().value,
                                    std::mem::transmute(&mut external));
        }
        if external.is_null() {
            return Err(JsExternalError.into())
        }
        Ok(unsafe { (*external).get_arc::<T>() })
    }
}

impl JsObject {
    pub fn set<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: KeyProperty + AsJs,
        V: AsJs
    {
        let key = key.as_js(&self.value.env)?.get_value();
        let value = value.as_js(&self.value.env)?.get_value();
        unsafe {
            Status::result(napi_set_property(self.value.env.inner(), self.value.get(), key.get(), value.get()))?;
        };
        Ok(())
    }

    pub fn get<K>(&self, key: K) -> Result<JsUnknown>
    where
        K: KeyProperty + AsJs
    {
        let key = key.as_js(&self.value.env)?.get_value();
        let mut value = Value::new(self.value.env);
        unsafe {
            Status::result(napi_get_property(self.value.env.inner(), self.value.get(), key.get(), value.get_mut()))?;
        };
        Ok(JsUnknown::from(value)?)
    }

    pub fn get_property_names(&self) -> Result<JsArray> {
        let mut value = Value::new(self.value.env);
        unsafe {
            Status::result(napi_get_property_names(self.value.env.inner(), self.value.get(), value.get_mut()))?;
        };
        Ok(JsArray::from(value))
    }

    pub fn has_property<K>(&self, key: K) -> Result<bool>
    where
        K: KeyProperty + AsJs
    {
        let mut result = false;
        let key = key.as_js(&self.value.env)?.get_value();
        unsafe {
            Status::result(napi_has_property(self.value.env.inner(), self.value.get(), key.get(), &mut result))?;
        };
        Ok(result)
    }

    pub fn has_own_property<K>(&self, key: K) -> Result<bool>
    where
        K: KeyProperty + AsJs
    {
        let mut result = false;
        let key = key.as_js(&self.value.env)?.get_value();
        unsafe {
            Status::result(napi_has_own_property(self.value.env.inner(), self.value.get(), key.get(), &mut result))?;
        };
        Ok(result)
    }

    pub fn define_properties(&self, props: impl IntoIterator<Item = PropertyDescriptor>) -> Result<()> {
        let props: Vec<_> = props.into_iter().map(|p: PropertyDescriptor| {
            p.into()
        }).collect();

        unsafe {
            Status::result(napi_define_properties(self.value.env.inner(),
                                                  self.value.get(),
                                                  props.len(),
                                                  props.as_ptr()))?;
        }

        Ok(())
    }

    pub fn define_property(&self, prop: PropertyDescriptor) -> Result<()> {
        unsafe {
            Status::result(napi_define_properties(self.value.env.inner(),
                                                  self.value.get(),
                                                  1,
                                                  &prop.into()))?;
        }

        Ok(())
    }

    pub fn napi_unwrap<T>(&self) -> Result<*mut T> {
        let mut obj: *mut T = std::ptr::null_mut();
        unsafe {
            Status::result(napi_unwrap(self.value.env.inner(),
                                       self.get_value().value,
                                       std::mem::transmute(&mut obj)))?;
        }
        Ok(obj)
    }
}

impl JsFunction {
    pub fn call_with_this(&self, this: impl AsJs, args: impl MultiJs) -> Result<JsUnknown> {
        let args: Vec<_> = args.make_iter(&self.value.env)?
                               .into_iter()
                               .map(|v| v.value)
                               .collect();
        let mut result = Value::new(self.value.env);
        let this = this.as_js(&self.value.env)?;
        unsafe {
            Status::result(napi_call_function(self.value.env.inner(),
                                              this.get_value().value,
                                              self.value.get(),
                                              args.len(),
                                              args.as_ptr(),
                                              result.get_mut()))?;
        };
        JsUnknown::from(result)
    }

    pub fn call(&self, args: impl MultiJs) -> Result<JsUnknown> {
        let global = self.value.env.global()?;
        self.call_with_this(global, args)
    }

    pub fn new_instance(&self, args: impl MultiJs) -> Result<JsObject> {
        let args: Vec<_> = args.make_iter(&self.value.env)?
                               .into_iter()
                               .map(|v| v.value)
                               .collect();
        let mut result = Value::new(self.value.env);
        unsafe {
            Status::result(napi_new_instance(self.value.env.inner(),
                                             self.value.get(),
                                             args.len(),
                                             args.as_ptr(),
                                             result.get_mut()))?;
        }
        Ok(JsObject::from(result))
    }
}

pub struct JsArrayIterator<'e> {
    index: usize,
    len: usize,
    array: &'e JsArray
}

impl<'e> Iterator for JsArrayIterator<'e> {
    type Item = JsUnknown;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let index = self.index;
            self.index += 1;
            if index >= self.len {
                return None;
            }
            if let Ok(item) = self.array.get(index as u32) {
                return Some(item);
            }
        }
    }
}

impl JsArray {
    pub fn iter(&self) -> Result<JsArrayIterator> {
        Ok(JsArrayIterator {
            index: 0,
            array: self,
            len: self.len()?
        })
    }

    pub fn len(&self) -> Result<usize> {
        let mut len: u32 = 0;
        unsafe {
            Status::result(napi_get_array_length(self.value.env.inner(), self.value.get(), &mut len as *mut u32))?;
        }
        Ok(len as usize)
    }

    pub fn set<V>(&self, index: u32, value: V) -> Result<()>
    where
        V: AsJs
    {
        let value = value.as_js(&self.value.env)?.get_value();
        unsafe {
            Status::result(napi_set_element(self.value.env.inner(), self.value.get(), index, value.get()))?;
        };
        Ok(())
    }

    pub fn get(&self, index: u32) -> Result<JsUnknown>
    {
        let mut value = Value::new(self.value.env);
        unsafe {
            Status::result(napi_get_element(self.value.env.inner(), self.value.get(), index, value.get_mut()))?;
        }
        JsUnknown::from(value)
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

    // ClassBuilder::<SomeClass>::new()
    //     .with_method("easy", SomeClass::jsfunction)
    //     .with_method("easy2", SomeClass::jsother)
    //     .with_accessor("easy3", SomeClass::jsaccessor)
    //     .new_instance(module.env, );

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
