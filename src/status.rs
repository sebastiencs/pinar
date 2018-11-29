use napi_sys::*;

#[derive(Debug)]
pub enum Status {
    Ok,
    InvalidArg,
    ObjectExpected,
    StringExpected,
    NameExpected,
    FunctionExpected,
    NumberExpected,
    BooleanExpected,
    ArrayExpected,
    GenericFailure,
    PendingException,
    Cancelled,
    EscapeCalledTwice,
    HandleCcopeMismatch,
    CallbackScopeMismatch,
    QueueFull,
    Closing,
    BigintExpected,
}

impl Status {
    pub fn result(status: napi_status) -> Result<(), Status> {
        match Status::from(status) {
            Status::Ok => Ok(()),
            s => Err(s)
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
