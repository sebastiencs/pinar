
use serde::Serialize;
use crate::prelude::*;
use crate::Result;

pub mod ser;

//pub use ser::to_js;

// pub struct SerializedObject(Value);

// pub trait IntoSerializedObject {
//     fn into(self, env: &Env) -> SerializedObject;
// }

// impl<T: Serialize> IntoSerializedObject for T {
//     fn into(self, env: &Env) -> SerializedObject {
//         SerializedObject(ser::to_js(env, &self).unwrap())
//     }
// }

// impl IntoJs for SerializedObject
// {
//     type JsType = JsObject;
//     fn into_js(self, env: &Env) -> Result<JsObject> {
//         Ok(JsObject::from(self.0))
//     }
// }

// default impl<T: Serialize + 'static> IntoJs<JsObject> for T
// {
//     //type JsType = JsObject;
//     fn into_js(self, env: &Env) -> Result<JsObject> {
//         Ok(JsObject::from(ser::to_js(env, &self).unwrap()))
//     }
// }

// impl<A> MultiJs for A
// where
//     A: Serialize,
// {
//     type Result = Vec<Value>;
//     fn make_iter(self, env: &Env) -> Result<Self::Result> {
//         Ok(vec![
//             self.as_js(env)?.get_value(),
//         ])
//     }
// }

// type JsType = Self;
// fn as_js(self, _: &Env) -> Result<Self::JsType> {
//     Ok(self)
// }

// impl JsReturn for SerializedObject
// {
//     type Value = Value;
//     type Error = ();
//     fn get_result(self, env: Env) -> std::result::Result<Option<Self::Value>, Self::Error> {
//         Ok(Some(self.as_js(&env)?))
//     }
// }

// impl AsJs for SerializedObject
// {
//     type JsType = Value;
//     fn as_js(self, _: &Env) -> Result<Self::JsType> {
//         Ok(self.0)
//     }
// }

// impl<T: Serialize> From<T> for SerializedObject {
//     fn from(o: T) -> SerializedObject {
//         let env = Env {
//             env: std::ptr::null_mut()
//         };
//         SerializedObject(ser::to_js(&env, &o).unwrap())
//     }
// }
