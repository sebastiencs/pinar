
//use crate::IntoHandle;
use crate::classes::execute_safely;
use crate::error::ArgumentsError;
use crate::error::JsFunctionError;
use napi_sys::*;
use crate::arguments::Arguments;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use crate::prelude::*;
use crate::Result;

pub struct ModuleBuilder<'e>
{
    env: Env,
    export: JsObject<'e>,
    functions: HashMap<String, ModuleFunction>,
    classes: Vec<(String, JsFunction<'e>)>
}

pub(crate) struct ModuleFunction {
    pub(crate) functions: Vec<Box<CallbackHandler>>,
    name: String
}

impl ModuleFunction {
    pub(crate) fn new<N, Fun, Args, R>(name: N, fun: Fun) -> ModuleFunction
    where
        N: Into<String>,
        Fun: Fn(Args) -> R + 'static,
        Args: FromArguments + 'static,
        R: for<'env> JsReturn<'env> + 'static
    {
        ModuleFunction {
            name: name.into(),
            functions: vec![Box::new(Callback::new(fun))]
        }
    }
}

impl<'e> ModuleBuilder<'e> {
    pub fn new(env: napi_env, export: napi_value) -> ModuleBuilder<'e> {
        let env = Env::from(env);
        let export = Value::from(env, export);
        ModuleBuilder {
            env,
            export: JsObject::from(export),
            functions: HashMap::new(),
            classes: vec![]
        }
    }

    pub fn with_function<S, Fun, Args, R>(mut self, name: S, fun: Fun) -> Self
    where
        S: Into<String>,
        Fun: Fn(Args) -> R + 'static,
        Args: FromArguments + 'static,
        R: for<'env> JsReturn<'env> + 'static
    {
        let name = name.into();
        match self.functions.entry(name.clone()) {
            Entry::Occupied(mut funs) => {
                funs.get_mut().functions.push(Box::new(Callback::new(fun)));
            }
            Entry::Vacant(funs) => {
                funs.insert(ModuleFunction::new(name, fun));
            }
        };
        self
    }

    pub fn with_class<C: 'static +  JsClass>(mut self, name: impl Into<String>, fun: impl Fn() -> ClassBuilder<C>) -> Self {
        let fun = fun().create(&self.env).unwrap();
        self.classes.push((name.into(), fun));
        self
    }

    pub fn build(self) -> Result<napi_value> {
        for (name, functions) in self.functions.into_iter() {
            let fun = Rc::new(functions);
            let jsfunction = self.env.function_internal(&name, fun)?;

            self.export.set(name.as_str(), jsfunction)?;
        }
        for (name, class) in self.classes {
            self.export.set(name, class)?;
        }
        Ok(self.export.get_value().value)
    }
}

pub(crate) extern "C" fn __pinar_dispatch_function(env: napi_env, info: napi_callback_info) -> napi_value {
    execute_safely(env, || {
        let env = Env::from(env);
        let (fun, args) = env.callback_info::<ModuleFunction>(info)?;

        if fun.is_null() {
            return Err(JsFunctionError::WrongFunctionData.into());
        }

        let function = unsafe { &mut *fun };
        let mut last_error = None;

        for function in &function.functions {
            let result = function.as_ref().handle(&args);
            if let Err(ref e) = result {
                if let Some(e) = e.downcast_ref::<ArgumentsError>() {
                    last_error = Some(e.clone());
                    continue;
                };
            };
            return result;
        }

        return match function.functions.len() {
            0 => Err(JsFunctionError::WrongFunctionData.into()),
            1 if last_error.is_some() => Err(last_error.unwrap().into()),
            _ => Err(JsFunctionError::ArgumentsOverload(function.name.clone()).into())
        };
    })
}

struct Callback<A, R>
where
    A: FromArguments,
    R: for<'env> JsReturn<'env>
{
    fun: Box<Fn(A) -> R>
}

impl<A, R> Callback<A, R>
where
    A: FromArguments,
    R: for<'env> JsReturn<'env>
{
    fn new<F>(fun: F) -> Self
    where
        F: Fn(A) -> R + 'static
    {
        Callback {
            fun: Box::new(fun)
        }
    }
}

pub(crate) trait CallbackHandler {
    fn handle(&self, args: &Arguments) -> Result<Option<napi_value>>;
}

impl<A, R> CallbackHandler for Callback<A, R>
where
    A: FromArguments,
    R: for<'env> JsReturn<'env>
{
    fn handle(&self, args: &Arguments) -> Result<Option<napi_value>> {
        let env = args.env();
        let args = A::from_args(args)?;

        Ok((self.fun)(args)
           .get_result(env)
           .map_err(|e| e.into())?
           .map(|res| res.get_value().value))
    }
}
