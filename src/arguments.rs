use std::sync::Arc;
use std::rc::Rc;
use napi_sys::*;
use std::cell::Cell;

use crate::error::ArgumentsError;
use crate::Result;
use crate::JsValue;
use crate::prelude::*;

pub struct Arguments {
    env: Env,
    args: Vec<Value>,
    this: Value,
    current_arg: Cell<usize>
}

impl Arguments {
    pub(crate) fn new(env: Env, this: Value, args: &[napi_value]) -> Result<Arguments> {
        Ok(Arguments {
            env,
            this,
            args: args.iter()
                      .map(|a| Value::from(env, *a))
                      .collect(),
            current_arg: Cell::new(0)
        })
    }

    pub fn this<'e>(&self) -> JsObject<'e> {
        JsObject::from(self.this)
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn arg_number(&self) -> usize {
        self.current_arg.get()
    }

    pub fn next_arg<'e>(&self) -> Option<JsUnknown<'e>> {
        let current = self.current_arg.get();
        self.current_arg.set(current + 1);
        self.args.get(current).and_then(|v| JsUnknown::from(*v).ok())
    }
}

pub trait FromArguments: Sized {
    fn from_args(args: &Arguments) -> Result<Self>;
}

macro_rules! from_args_tuples {
    (
        $( ( $($tuple:ident),* ) ),*
    ) => {
        $(
            impl<$($tuple),*> FromArguments for ($($tuple,)*)
            where
                $($tuple : FromArguments,)*
            {
                #[allow(non_snake_case, unused_variables)]
                fn from_args(args: &Arguments) -> Result<Self> {
                    // FromArguments::from_args needs to be called in order
                    $(let $tuple = $tuple::from_args(args)?;)*
                    Ok(($($tuple,)*))
                }
            }
        )*
    }
}

from_args_tuples!(
    (),
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
    (A, B, C, D, E, F, G, H, I, J, K, L, M),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)
);

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

impl<A> FromArguments for Vec<A>
where
    A: FromArguments
{
    fn from_args(args: &Arguments) -> Result<Self> {
        match args.next_arg() {
            Some(JsUnknown::Array(array)) => {
                let args = Arguments {
                    args: array.values()?,
                    current_arg: Cell::new(0),
                    env: *args.env(),
                    this: args.this().get_value()
                };

                (0..array.len()?).map(|_| A::from_args(&args))
                                 .collect()
            }
            Some(_) => Err(ArgumentsError::wrong_type("array", args.arg_number())),
            _ => Err(ArgumentsError::missing(args.arg_number()))
        }
    }
}

impl FromArguments for Env {
    fn from_args(args: &Arguments) -> Result<Self> {
        Ok(args.env)
    }
}

macro_rules! from_args_js {
    (
        JS_TYPES:
        $( ( $jstype:ident, $utype:ident, $str:expr ) ),*,
        RUST_TYPES:
        $( ( $rtype:ident, $rutype:ident, $rstr:expr $(,$gen:ident),* ) ),*
    ) => {
        $(
            impl<'e> FromArguments for $jstype<'e>
            {
                fn from_args(args: &Arguments) -> Result<Self> {
                    match args.next_arg() {
                        Some(JsUnknown::$utype(value)) => Ok(value),
                        Some(_) => Err(ArgumentsError::wrong_type($str, args.arg_number())),
                        _ => Err(ArgumentsError::missing(args.arg_number()))
                    }
                }
            }
        )*
        $(
            impl<$($gen: 'static)*> FromArguments for $rtype<$($gen)*>
            {
                fn from_args(args: &Arguments) -> Result<Self> {
                    match args.next_arg() {
                        Some(JsUnknown::$rutype(value)) => value.to_rust(),
                        Some(_) => Err(ArgumentsError::wrong_type($rstr, args.arg_number())),
                        _ => Err(ArgumentsError::missing(args.arg_number()))
                    }
                }
            }
        )*
    }
}

from_args_js!(
    JS_TYPES:
    (JsSymbol, Symbol, "symbol"),
    (JsString, String, "string"),
    (JsObject, Object, "object"),
    (JsArray, Array, "array"),
    (JsNumber, Number, "number"),
    (JsUndefined, Undefined, "undefined"),
    (JsFunction, Function, "function"),
    (JsExternal, External, "external"),
    (JsNull, Null, "null"),
    (JsBoolean, Boolean, "boolean"),
    (JsBigInt, BigInt, "bigint"),
    RUST_TYPES:
    (i32, Number, "number (integer)"),
    (i64, Number, "number (integer)"),
    (f64, Number, "number (double)"),
    (String, String, "string"),
    (bool, Boolean, "string"),
    (Box, External, "external (box)", T),
    (Rc, External, "external (rc)", T),
    (Arc, External, "external (arc)", T)
);
