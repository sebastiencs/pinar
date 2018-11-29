
use crate::IntoHandle;
use crate::JsObject;
use crate::Value;
use crate::Env;
use crate::Result;

pub struct ModuleBuilder<'e> {
    env: &'e Env,
    module: Value<'e, JsObject>
}

impl<'e> ModuleBuilder<'e> {
    pub fn build(env: &'e Env) -> Result<ModuleBuilder> {
        Ok(ModuleBuilder {
            env,
            module: env.object()?
        })
    }

    pub fn with_function<S, F, R>(self, name: S) -> Result<()>
    where
        S: AsRef<str>,
        F: Fn() -> R,
        R: IntoHandle
    {
        Ok(())
    }
}
