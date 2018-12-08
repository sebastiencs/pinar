
use crate::*;

pub struct JsString {
    pub(crate) value: Value
}

impl JsString {
    /// Returns the string length
    pub fn len(&self) -> Result<usize> {
        unsafe {
            let mut length = 0;
            Status::result(napi_get_value_string_utf8(
                self.value.env(),
                self.value.get(),
                std::ptr::null_mut() as *mut c_char,
                0,
                &mut length as *mut usize)
            )?;
            Ok(length)
        }
    }
}
