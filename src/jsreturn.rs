use crate::JsUndefined;
use crate::JsValue;
use crate::Env;
use crate::error::Error;
use crate::ToJs;
use crate::Value;

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
/// - `Result<T, E>` where `T:`[`JsReturn`] and `E:`[`JsError`],
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
    type Value: JsValue;
    type Error: Into<Error>;
    fn get_result(self, env: Env) -> Result<Option<Value>, Self::Error>;
}

impl<'e, T> JsReturn<'e> for T
where
    T: ToJs<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Value>, Self::Error> {
        Ok(Some(self.to_js(env)?.get_value()))
    }
}

impl<'e, T> JsReturn<'e> for Option<T>
where
    T: JsReturn<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Value>, Self::Error> {
        match self {
            Some(v) => v.get_result(env).map_err(Into::into),
            None => Ok(None)
        }
    }
}

impl<'e, T> JsReturn<'e> for crate::Result<T>
where
    T: JsReturn<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Value>, Self::Error> {
        self?.get_result(env).map_err(Into::into)
    }
}

impl<'e> JsReturn<'e> for () {
    type Value = JsUndefined<'e>;
    type Error = Error;
    fn get_result(self, _: Env) -> Result<Option<Value>, Self::Error> {
        Ok(None)
    }
}

use crate::classes::JsClass;
use crate::classes::AsJsClass;

impl<'e, C> JsReturn<'e> for AsJsClass<C>
where
    C: JsClass
{
    type Value = Value;
    type Error = Error;
    fn get_result(self, env: Env) -> Result<Option<Value>, Self::Error> {
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
    type Value: JsValue;
    type Error: Into<Error>;
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Self::Error>;
}

impl<'e, T> JsReturnRef<'e> for T
where
    T: ToJs<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Self::Error> {
        Ok(Some(self.to_js(env)?.get_value()))
    }
}

impl<'e, T> JsReturnRef<'e> for Option<&T>
where
    T: JsReturnRef<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Self::Error> {
        match self {
            Some(v) => v.get_result_from_ref(env).map_err(Into::into),
            None => Ok(None)
        }
    }
}

impl<'e, T> JsReturnRef<'e> for crate::Result<&T>
where
    T: JsReturnRef<'e>
{
    type Value = T::Value;
    type Error = Error;
    fn get_result_from_ref(&self, env: Env) -> Result<Option<Value>, Self::Error> {
        match self {
            Ok(v) => v.get_result_from_ref(env).map_err(Into::into),
            Err(e) => Err(e.into())
        }
    }
}

impl<'e> JsReturnRef<'e> for ()
{
    type Value = Value;
    type Error = Error;
    fn get_result_from_ref(&self, _env: Env) -> Result<Option<Value>, Self::Error> {
        Ok(None)
    }
}
