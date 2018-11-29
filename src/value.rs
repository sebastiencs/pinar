use napi_sys::*;

#[derive(Debug)]
pub enum ValueType {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Symbol,
    Object,
    Function,
    External,
    Bigint,
}

impl From<napi_valuetype> for ValueType {
    fn from(status: napi_valuetype) -> ValueType {
        match status {
            napi_valuetype::napi_undefined => ValueType::Undefined,
            napi_valuetype::napi_null => ValueType::Null,
            napi_valuetype::napi_boolean => ValueType::Boolean,
            napi_valuetype::napi_number => ValueType::Number,
            napi_valuetype::napi_string => ValueType::String,
            napi_valuetype::napi_symbol => ValueType::Symbol,
            napi_valuetype::napi_object => ValueType::Object,
            napi_valuetype::napi_function => ValueType::Function,
            napi_valuetype::napi_external => ValueType::External,
            napi_valuetype::napi_bigint => ValueType::Bigint,
        }
    }
}

impl Into<napi_valuetype> for ValueType {
    fn into(self) -> napi_valuetype {
        match self {
            ValueType::Undefined => napi_valuetype::napi_undefined,
            ValueType::Null => napi_valuetype::napi_null,
            ValueType::Boolean => napi_valuetype::napi_boolean,
            ValueType::Number => napi_valuetype::napi_number,
            ValueType::String => napi_valuetype::napi_string,
            ValueType::Symbol => napi_valuetype::napi_symbol,
            ValueType::Object => napi_valuetype::napi_object,
            ValueType::Function => napi_valuetype::napi_function,
            ValueType::External => napi_valuetype::napi_external,
            ValueType::Bigint => napi_valuetype::napi_bigint,
        }
    }
}
