
use std::convert::TryInto;
use std::marker::PhantomData;
use serde::de::DeserializeOwned;

use crate::prelude::*;
use crate::*;

/// A Javascript function.
pub struct JsFunction<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

impl<'e> JsFunction<'e> {
    /// Call the Javascript function and return its result.   
    /// The `this` of the function will be the `global` object.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn method(fun: JsFunction) -> JsResult<()> {
    ///     // Call the function with 1 argument:
    ///     fun.call("some_text")?;
    ///     
    ///     // You can call a function with multiple arguments
    ///     // using a tuple
    ///     // Each element has to implement the trait ToJs
    ///     fun.call((1, 2, "text", vec![10, 50], Rc::new(21)))?;
    ///     
    ///     // If no argument, you need to use an empty tuple
    ///     let res: MyCustomType = fun.call(())?
    ///                                .as_jsobject()?
    ///                                .to_rust()?;
    ///     
    ///     Ok(())
    /// }
    ///  
    /// ```
    pub fn call(&self, args: impl MultiJs) -> Result<JsAny<'e>> {
        let global = self.value.env.global()?;
        self.call_with_this(global, args)
    }

    /// Similar to [`call`] but uses a custom `this` for the function
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn method(fun: JsFunction) -> JsResult<()> {
    ///     // The `this` of the function will be a empty object
    ///     fun.call_with_this(HashMap::new(), "some_text")?;
    ///     
    ///     Ok(())
    /// }
    ///  
    /// ``` 
    /// [`call`]: #method.call
    pub fn call_with_this(&self, this: impl ToJs<'e>, args: impl MultiJs) -> Result<JsAny<'e>> {
        let args: Vec<_> = args.make_iter(self.value.env)?
                               .into_iter()
                               .map(|v| v.value)
                               .collect();
        let mut result = Value::new(self.value.env);
        let this = this.to_js(self.value.env)?;

        napi_call!(napi_call_function(
            self.value.env(),
            this.get_value().value,
            self.value.get(),
            args.len(),
            args.as_ptr(),
            result.get_mut()
        ))?;

        JsAny::from(result)
    }

    /// In case the `JsFunction` is a constructor, `new_instance` will instantiate
    /// a new Javascript object.   
    /// This is similar to the following javascript code:
    /// ```
    /// const value = new MyConstructor(1, 2, 3);
    /// ```
    pub fn new_instance(&self, args: impl MultiJs) -> Result<JsObject<'e>> {
        let args: Vec<_> = args.make_iter(self.value.env)?
                               .into_iter()
                               .map(|v| v.value)
                               .collect();
        let mut result = Value::new(self.value.env);

        napi_call!(napi_new_instance(
            self.value.env(),
            self.value.get(),
            args.len(),
            args.as_ptr(),
            result.get_mut()
        ))?;

        Ok(JsObject::from(result))
    }

    /// Creates a [`JsFunctionThreadSafe`] from the current Javascript function.  
    /// The Javascript function will be callable from other threads.
    ///
    /// The arguments and return's value types have to be specified with generic parameter.
    ///
    /// # Example
    ///
    /// ```
    /// #[pinar]
    /// fn my_func(fun: JsFunction) -> JsResult<()> {
    ///     let fun = fun.make_threadsafe::<(String, i64), PathBuf>()?;
    ///
    ///     std::thread::spawn(move || {
    ///         // The Javascript function will be called on the JS main thread
    ///         // and the return value is transfered back to this thread
    ///         let res: PathBuf = fun.call(("hello".to_string(), 124)).unwrap();
    ///     });
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// More information can be found on [`JsFunctionThreadSafe`].
    /// 
    /// [`JsFunctionThreadSafe`]: ../struct.JsFunctionThreadSafe.html
    pub fn make_threadsafe<Args, Ret>(&self) -> Result<JsFunctionThreadSafe<Args, Ret>>
    where
        Args: MultiJs + 'static,
        Ret: DeserializeOwned,
    {
        self.try_into()
    }
}
