
use crate::*;

pub struct JsFunction {
    pub(crate) value: Value
}

impl JsFunction {
    pub fn call_with_this(&self, this: impl AsJs, args: impl MultiJs) -> Result<JsUnknown> {
        let args: Vec<_> = args.make_iter(&self.value.env)?
                               .into_iter()
                               .map(|v| v.value)
                               .collect();
        let mut result = Value::new(self.value.env);
        let this = this.as_js(&self.value.env)?;
        unsafe {
            Status::result(napi_call_function(self.value.env(),
                                              this.get_value().value,
                                              self.value.get(),
                                              args.len(),
                                              args.as_ptr(),
                                              result.get_mut()))?;
        };
        JsUnknown::from(result)
    }

    pub fn call(&self, args: impl MultiJs) -> Result<JsUnknown> {
        let global = self.value.env.global()?;
        self.call_with_this(global, args)
    }

    pub fn new_instance(&self, args: impl MultiJs) -> Result<JsObject> {
        let args: Vec<_> = args.make_iter(&self.value.env)?
                               .into_iter()
                               .map(|v| v.value)
                               .collect();
        let mut result = Value::new(self.value.env);
        unsafe {
            Status::result(napi_new_instance(self.value.env(),
                                             self.value.get(),
                                             args.len(),
                                             args.as_ptr(),
                                             result.get_mut()))?;
        }
        Ok(JsObject::from(result))
    }
}
