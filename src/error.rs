use crate::status::Status;

use backtrace::Backtrace;

use derive_more::Display;

#[derive(Debug)]
pub struct Error {
    cause: Box<dyn JsError>,
    backtrace: Backtrace,
}

impl Error {
    pub fn as_js_error(&self) -> &dyn JsError {
        self.cause.as_ref()
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    pub fn is_type<T: 'static>(&self) -> bool {
         self.cause.type_id() == TypeId::of::<T>()
    }

    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.cause.as_any().downcast_ref::<T>()
    }

    #[allow(mutable_transmutes)]
    fn take(&self) -> Self {
        let self_mut: &mut Self = unsafe { std::mem::transmute(self) };
        
        let cause: Box<dyn JsError> = std::mem::replace(
            &mut self_mut.cause,
            Box::new(JsReturnRefError)
        );
        
        Error {
            cause,
            backtrace: self.backtrace.clone()
        }
    }
}

#[derive(Display, Debug)]
#[display(fmt = "Error on JsReturnRef Error, this should never happen.")]
pub(crate) struct JsReturnRefError;

#[derive(Display, Debug)]
#[display(fmt = "Null pointer on external data")]
pub(crate) struct JsExternalError;

#[derive(Display, Debug)]
pub(crate) enum JsClassError {
    #[display(fmt = "Fail to extract external value from class. Please report on pinar repo.")]
    ExternalClassData,
    #[display(fmt = "Wrong handler class. Please report on pinar repo.")]
    WrongHandler,
    #[display(fmt = "A class method has been called with the wrong class. Check your JS !")]
    WrongClass,
    #[display(fmt = "Wrong 'this' value. Did you call the constructor with 'new' ? (ex: 'let a = new {}()')", _0)]
    ThisConstructor(&'static str),
    #[display(fmt = "Wrong 'this' value on a method call of the class {}", _0)]
    ThisMethod(&'static str),
    #[display(fmt = "Constructor of the class {} is not defined", _0)]
    NoConstructor(&'static str),
    #[display(fmt = "Fail to unwrap the class. Please report on pinar repo.")]
    Unwrap,
}

#[derive(Display, Debug)]
pub(crate) enum JsFunctionError {
    #[display(fmt = "Multiple overload of the function {} failed.", _0)]
    ArgumentsOverload(String),
//    #[display(fmt = "{{ function {} }}: {}.", _0, _1)]
//    Arguments(String, String),
    #[display(fmt = "Fail to dispatch the function, please report on pinar repo.")]
    WrongFunctionData,
}

#[derive(Display, Debug, Clone)]
pub enum ArgumentsError {
    #[display(fmt = "{}th argument is missing", _0)]
    Missing(usize),
    #[display(fmt = "Wrong type, expected a {} on the {}th argument", _0, _1)]
    WrongType(String, usize),
    #[display(fmt = "Deserialization error: {}", _0)]
    Deserialization(String)
}

#[derive(Display, Debug, Clone)]
pub enum JsAnyError {
    #[display(fmt = "Wrong conversion from a JsAny")]
    WrongAny,
}

impl ArgumentsError {
    pub fn wrong_type(s: &str, n: usize) -> Error {
        ArgumentsError::WrongType(s.to_owned(), n).into()
    }

    pub fn missing(n: usize) -> Error {
        ArgumentsError::Missing(n).into()
    }
}

impl<T: JsError + 'static> From<T> for Error {
    fn from(error: T) -> Error {
        Error {
            backtrace: Backtrace::new(),
            cause: Box::new(error),
        }
    }
}

impl From<&Error> for Error {
    fn from(error: &Error) -> Error {
        error.take()
    }
}

use std::any::TypeId;
use std::any::Any;

pub trait JsError: JsErrorAsAny + std::fmt::Display + std::fmt::Debug + 'static {
    fn get_msg(&self) -> String {
        format!("{}", self)
    }
    fn get_code(&self) -> Option<String> {
        None
    }
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

pub trait JsErrorAsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: JsError> JsErrorAsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl JsError for Status {
    fn get_code(&self) -> Option<String> {
        Some("N-API".to_owned())
    }
}

impl JsError for ArgumentsError {
    fn get_code(&self) -> Option<String> {
        Some("PINAR".to_owned())
    }
}

impl JsError for JsExternalError {
    fn get_code(&self) -> Option<String> {
        Some("PINAR".to_owned())
    }
}

impl JsError for JsReturnRefError {
    fn get_code(&self) -> Option<String> {
        Some("PINAR".to_owned())
    }
}

impl JsError for JsClassError {
    fn get_code(&self) -> Option<String> {
        Some("PINAR".to_owned())
    }
}

impl JsError for JsFunctionError {
    fn get_code(&self) -> Option<String> {
        Some("PINAR".to_owned())
    }
}

impl JsError for JsAnyError {
    fn get_code(&self) -> Option<String> {
        Some("PINAR".to_owned())
    }
}

impl JsError for std::io::Error {
    fn get_code(&self) -> Option<String> {
        Some("IO".to_owned())
    }
}
