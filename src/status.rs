use napi_sys::*;

use derive_more::Display;

/// Represents different values returned from n-api functions
///
/// This implements [`JsError`]
///
/// [`JsError`]: ./trait.JsError.html
#[derive(Display, Debug)]
pub enum Status {
    #[display(fmt = "Ok.")]
    Ok,
    #[display(fmt = "Invalid argument.")]
    InvalidArg,
    #[display(fmt = "Object expected.")]
    ObjectExpected,
    #[display(fmt = "String expected.")]
    StringExpected,
    #[display(fmt = "Name expected.")]
    NameExpected,
    #[display(fmt = "Function expected.")]
    FunctionExpected,
    #[display(fmt = "Number expected.")]
    NumberExpected,
    #[display(fmt = "Boolean expected.")]
    BooleanExpected,
    #[display(fmt = "Array expected.")]
    ArrayExpected,
    #[display(fmt = "Generic failure.")]
    GenericFailure,
    #[display(fmt = "Pending exception.")]
    PendingException,
    #[display(fmt = "Cancelled.")]
    Cancelled,
    #[display(fmt = "Escape called twice.")]
    EscapeCalledTwice,
    #[display(fmt = "Handle Ccsope mismatch.")]
    HandleCcopeMismatch,
    #[display(fmt = "Callback scope mismatch.")]
    CallbackScopeMismatch,
    #[display(fmt = "Queue full.")]
    QueueFull,
    #[display(fmt = "Closing.")]
    Closing,
    #[display(fmt = "Bigint expected.")]
    BigintExpected,
}

impl Status {
    #[inline]
    pub fn result(status: napi_status) -> Result<(), Status> {
        match status {
            napi_status::napi_ok => Ok(()),
            e => Err(Status::from(e))
        }
    }
}

impl From<napi_status> for Status {
    fn from(status: napi_status) -> Status {
        match status {
            napi_status::napi_ok => Status::Ok,
            napi_status::napi_invalid_arg => Status::InvalidArg,
            napi_status::napi_object_expected => Status::ObjectExpected,
            napi_status::napi_string_expected => Status::StringExpected,
            napi_status::napi_name_expected => Status::NameExpected,
            napi_status::napi_function_expected => Status::FunctionExpected,
            napi_status::napi_number_expected => Status::NumberExpected,
            napi_status::napi_boolean_expected => Status::BooleanExpected,
            napi_status::napi_array_expected => Status::ArrayExpected,
            napi_status::napi_generic_failure => Status::GenericFailure,
            napi_status::napi_pending_exception => Status::PendingException,
            napi_status::napi_cancelled => Status::Cancelled,
            napi_status::napi_escape_called_twice => Status::EscapeCalledTwice,
            napi_status::napi_handle_scope_mismatch => Status::HandleCcopeMismatch,
            napi_status::napi_callback_scope_mismatch => Status::CallbackScopeMismatch,
            napi_status::napi_queue_full => Status::QueueFull,
            napi_status::napi_closing => Status::Closing,
            napi_status::napi_bigint_expected => Status::BigintExpected
        }
    }
}

impl Into<napi_status> for Status {
    fn into(self) -> napi_status {
        match self {
            Status::Ok => napi_status::napi_ok,
            Status::InvalidArg => napi_status::napi_invalid_arg,
            Status::ObjectExpected => napi_status::napi_object_expected,
            Status::StringExpected => napi_status::napi_string_expected,
            Status::NameExpected => napi_status::napi_name_expected,
            Status::FunctionExpected => napi_status::napi_function_expected,
            Status::NumberExpected => napi_status::napi_number_expected,
            Status::BooleanExpected => napi_status::napi_boolean_expected,
            Status::ArrayExpected => napi_status::napi_array_expected,
            Status::GenericFailure => napi_status::napi_generic_failure,
            Status::PendingException => napi_status::napi_pending_exception,
            Status::Cancelled => napi_status::napi_cancelled,
            Status::EscapeCalledTwice => napi_status::napi_escape_called_twice,
            Status::HandleCcopeMismatch => napi_status::napi_handle_scope_mismatch,
            Status::CallbackScopeMismatch => napi_status::napi_callback_scope_mismatch,
            Status::QueueFull => napi_status::napi_queue_full,
            Status::Closing => napi_status::napi_closing,
            Status::BigintExpected => napi_status::napi_bigint_expected
        }
    }
}
