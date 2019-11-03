use crate::JsValue;
use crate::Env;
use crate::error::Error;
use crate::ToJs;
use crate::Value;
use crate::JsResult;

/// Trait that exported functions/methods to javascript returns.
///
/// There is an implementation for:
/// - `T` where `T:`[`ToJs`]
/// - `()` returns a javascript `undefined`.
/// - [`AsJsClass`]`<C>` where `C:`[`JsClass`]
///    - The type `C` is instantiated to a js class.  
/// .  
/// - `Option<T>` where `T:`[`JsReturn`],  
///    - None returns a javascript `undefined`.
/// - `JsResult<T>` where `T:`[`JsReturn`],
///    - the error is thrown in javascript.
///
/// # Example
/// 
/// ```
/// #[derive(Serialize, Deserialize, Pinar)]
/// struct MyStruct { .. }
/// 
/// // Thoses returns types implement ToJs
/// #[pinar]
/// fn my_func() -> String { .. }
/// #[pinar]
/// fn my_func() -> PathBuf { .. }
/// #[pinar]
/// fn my_func() -> MyStruct { .. }
/// 
/// // The javascript results is `undefined`
/// #[pinar]
/// fn my_func() -> Option<i64> {
///     None
/// }
///
/// // The javascript results is `undefined`
/// #[pinar]
/// fn my_func() -> () { }
/// #[pinar]
/// fn my_func() { }
/// 
/// struct MyError;
/// 
/// impl JsError for MyError { .. }
/// 
/// // The error is thrown in javascript
/// #[pinar]
/// fn my_func() -> JsResult<()> {
///     Err(MyError{}.into())
/// }
/// ```
///
/// [`JsError`]: ./trait.JsError.html
pub trait JsReturn<'e> {
    fn get_result(self, env: Env) -> Result<Option<Value>, Error>;
}

impl<'e, T> JsReturn<'e> for T
where
    T: ToJs<'e>
{
    fn get_result(self, env: Env) -> Result<Option<Value>, Error> {
        Ok(Some(self.to_js(env)?.get_value()))
    }
}

impl<'e, T> JsReturn<'e> for Option<T>
where
    T: JsReturn<'e>
{
    fn get_result(self, env: Env) -> Result<Option<Value>, Error> {
        match self {
            Some(v) => v.get_result(env),
            None => Ok(None)
        }
    }
}

impl<'e, T> JsReturn<'e> for JsResult<T>
where
    T: JsReturn<'e>
{
    #[cfg(feature = "nightly")] // Make the fn default
    default fn get_result(self, env: Env) -> Result<Option<Value>, Error> {
        self?.get_result(env)
    }

    #[cfg(not(feature = "nightly"))]
    fn get_result(self, env: Env) -> Result<Option<Value>, Error> {
        self?.get_result(env)
    }
}

#[cfg(feature = "nightly")] // Specialize the impl
impl<'e, T> JsReturn<'e> for JsResult<Option<T>>
where
    T: ToJs<'e>
{
    fn get_result(self, env: Env) -> Result<Option<Value>, Error> {
        match self? {
            Some(v) => Ok(Some(v.to_js(env)?.get_value())),
            _ => Ok(None)
        }
    }
}

impl<'e> JsReturn<'e> for () {
    fn get_result(self, _: Env) -> Result<Option<Value>, Error> {
        Ok(None)
    }
}

use crate::classes::JsClass;
use crate::classes::AsJsClass;

impl<'e, C> JsReturn<'e> for AsJsClass<C>
where
    C: JsClass
{
    fn get_result(self, env: Env) -> Result<Option<Value>, Error> {
        self.to_js_class(env).map(Some)
    }
}


/// JsReturnRef
///
/// Trait similar to [`JsReturn`] but takes a reference for self.  
/// This is used in methods of class returning a reference to themself.
///
/// # Example
///
/// ```
/// struct MyType {
///     s: String
/// };
///
/// #[pinar]
/// impl MyType {
///     fn my_method(&self) -> &str {
///         &self.s
///     }
/// }
///
/// // The impl block, with the `pinar` macro, is transformed to:
///
/// impl MyType {
///     fn my_method(&self) -> &str {
///         &self.s
///     }
///     fn __pinar_my_method(&mut self, env: Env) -> JsResult<Option<Value>> {
///         self.my_method()
///             .get_result_from_ref(env)
///     }
/// }
/// 
/// // The return type of __pinar_my_method (JsResult<Option<Value>>) implements JsReturn
/// // so it can be exported to javascript.
///
/// ```
///
/// Users of Pinar don't have to deal with this trait (thanks to the pinar macro) so
/// we hide the doc
///
#[doc(hidden)]
pub trait JsReturnRef<'e> {
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Error>;
}

impl<'e, T> JsReturnRef<'e> for T
where
    T: ToJs<'e>
{
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Error> {
        Ok(Some(self.to_js(env)?.get_value()))
    }
}

impl<'e, T> JsReturnRef<'e> for Option<&T>
where
    T: JsReturnRef<'e>
{
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Error> {
        match self {
            Some(v) => v.get_result_from_ref(env),
            None => Ok(None)
        }
    }
}

impl<'e, T> JsReturnRef<'e> for JsResult<&T>
where
    T: JsReturnRef<'e>
{
    #[cfg(feature = "nightly")] // Make the fn default
    default fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Error> {
        match self {
            Ok(v) => v.get_result_from_ref(env),
            Err(e) => Err(e.into())
        }
    }

    #[cfg(not(feature = "nightly"))]
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Error> {
        match self {
            Ok(v) => v.get_result_from_ref(env),
            Err(e) => Err(e.into())
        }
    }
}

#[cfg(feature = "nightly")] // Specialize the impl
impl<'e, T> JsReturnRef<'e> for JsResult<Option<&T>>
where
    T: ToJs<'e>
{
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Error> {
        match self {
            Ok(Some(v)) => Ok(Some(v.to_js(env)?.get_value())),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

impl<'e> JsReturnRef<'e> for ()
{
    fn get_result_from_ref(&self, _env: Env) -> Result<Option<Value>, Error> {
        Ok(None)
    }
}
