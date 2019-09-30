use crate::status::Status;
use failure::{Fail, Backtrace};

#[derive(Debug)]
pub struct Error {
    cause: Box<JsError>,
    backtrace: Option<Backtrace>
}

impl Error {
    pub fn as_js_error(&self) -> &JsError {
        self.cause.as_ref()
    }

    pub fn backtrace(&self) -> &Backtrace {
        match self.cause.backtrace() {
            Some(bt) => bt,
            _ => self.backtrace.as_ref().unwrap()
        }
    }

    pub fn downcast_ref<T: Fail>(&self) -> Option<&T> {
        Fail::downcast_ref(self.cause.as_fail())
    }

    pub fn is_type<T: Fail>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Null pointer on external data")]
pub(crate) struct JsExternalError;

#[derive(Fail, Debug)]
pub(crate) enum JsClassError {
    #[fail(display = "Fail to extract external value from class. Please report on pinar repo.")]
    ExternalClassData,
    #[fail(display = "Wrong handler class. Please report on pinar repo.")]
    WrongHandler,
    #[fail(display = "A class method has been called with the wrong class. Check your JS !")]
    WrongClass,
    #[fail(display = "Wrong 'this' value. Did you call the constructor with 'new' ? (ex: 'let a = new {}()')", _0)]
    ThisConstructor(&'static str),
    #[fail(display = "Wrong 'this' value on a method call of the class {}", _0)]
    ThisMethod(&'static str),
    #[fail(display = "Constructor of the class {} is not defined", _0)]
    NoConstructor(&'static str),
    #[fail(display = "Fail to unwrap the class. Please report on pinar repo.")]
    Unwrap,
}

#[derive(Fail, Debug)]
pub(crate) enum JsFunctionError {
    #[fail(display = "Multiple overload of the function {} failed.", _0)]
    ArgumentsOverload(String),
//    #[fail(display = "{{ function {} }}: {}.", _0, _1)]
//    Arguments(String, String),
    #[fail(display = "Fail to dispatch the function, please report on pinar repo.")]
    WrongFunctionData,
}

#[derive(Fail, Debug, Clone)]
pub enum ArgumentsError {
    #[fail(display = "{}th argument is missing", _0)]
    Missing(usize),
    #[fail(display = "Wrong type, expected a {} on the {}th argument", _0, _1)]
    WrongType(String, usize),
    #[fail(display = "Deserialization error: {}", _0)]
    Deserialization(String)
}

#[derive(Fail, Debug, Clone)]
pub enum JsAnyError {
    #[fail(display = "Wrong conversion from a JsAny")]
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

impl<T: JsError> From<T> for Error {
    fn from(error: T) -> Error {
        Error {
            backtrace: match error.backtrace() {
                None => Some(Backtrace::new()),
                _ => None
            },
            cause: Box::new(error)
        }
    }
}

pub trait JsError: Fail + JsErrorAsFail {
    fn get_msg(&self) -> String {
        format!("{}", self)
    }
    fn get_code(&self) -> Option<String> {
        None
    }
}

pub trait JsErrorAsFail {
    fn as_fail(&self) -> &Fail;
}

impl<T: JsError> JsErrorAsFail for T {
    fn as_fail(&self) -> &Fail {
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
