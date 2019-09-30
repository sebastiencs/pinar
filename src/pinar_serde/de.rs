
use serde;
use serde::de::Visitor;
use serde::de::{DeserializeOwned, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, Unexpected,
                VariantAccess};
use serde::forward_to_deserialize_any;

use crate::prelude::*;
use std::fmt;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct DeserializeError {
    msg: String
}

impl DeserializeError {
    fn new<M: AsRef<str>>(msg: M) -> DeserializeError {
        let msg = msg.as_ref();
        DeserializeError { msg: msg.to_string() }
    }
}

impl From<crate::Error> for DeserializeError {
    fn from(o: crate::Error) -> DeserializeError {
        DeserializeError {
            msg: format!("{:?}", o)
        }
    }
}

impl From<Status> for DeserializeError {
    fn from(o: Status) -> DeserializeError {
        DeserializeError {
            msg: format!("{:?}", o)
        }
    }
}

type Result<T> = std::result::Result<T, DeserializeError>;

impl serde::de::Error for DeserializeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self { msg: msg.to_string() }
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for DeserializeError {
    fn description(&self) -> &str {
        self.msg.as_str()
    }
}

pub fn from_any<T>(env: Env, any: JsAny) -> Result<T>
where
    T: DeserializeOwned + ?Sized,
{
    let de: Deserializer = Deserializer::new(env, any);
    T::deserialize(de)
}

pub fn from_value<T>(env: Env, value: Value) -> Result<T>
where
    T: DeserializeOwned + ?Sized,
{
    let de: Deserializer = Deserializer::new(env, JsAny::from(value)?);
    T::deserialize(de)
}

#[doc(hidden)]
pub struct Deserializer<'e> {
    env: Env,
    input: JsAny<'e>
}

#[doc(hidden)]
impl<'e> Deserializer<'e> {
    fn new(env: Env, input: JsAny<'e>) -> Self {
        Deserializer { env, input }
    }
}

#[doc(hidden)]
impl<'e, 'de> serde::de::Deserializer<'de> for Deserializer<'e> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input {
            JsAny::String(s) => visitor.visit_string(s.to_rust()?),
            JsAny::Undefined(_) => visitor.visit_unit(),
            JsAny::Null(_) => visitor.visit_unit(),
            JsAny::Boolean(b) => visitor.visit_bool(b.to_rust()?),
            JsAny::Object(o) => {
                let mut deserializer = JsObjectAccess::new(self.env, o)?;
                visitor.visit_map(&mut deserializer)
            },
            JsAny::Array(a) => {
                let mut deserializer = JsArrayAccess::new(self.env, a);
                visitor.visit_seq(&mut deserializer)
            },
            JsAny::Number(n) => {
                visitor.visit_i64(n.to_rust()?)
            },
            JsAny::Symbol(_) => unimplemented!(),
            JsAny::External(_) => unimplemented!(),
            JsAny::Function(_) => unimplemented!(),
            JsAny::BigInt(_) => unimplemented!(),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input {
            JsAny::Undefined(_) | JsAny::Null(_) => visitor.visit_none(),
            _ => visitor.visit_some(self)
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input {
            JsAny::String(s) => {
                visitor.visit_enum(JsEnumAccess::new(self.env, s.to_rust().unwrap(), None))
            },
            JsAny::Object(o) => {
                let props = o.get_property_names()
                             .map_err(|e| DeserializeError::new(format!("{:?}", e)))?;

                if props.len() != 1 {
                    return Err(DeserializeError::new(
                        format!("object with {} properties, expected 1", props.len())
                    ));
                }
                let key = &props[0];
                let value = o.get(key.as_str())
                             .map_err(|e| DeserializeError::new(format!("{:?}", e)))?;

                visitor.visit_enum(JsEnumAccess::new(self.env, key.to_string(), Some(value)))
            },
            _ => {
                unimplemented!()
            }
        }
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        unit unit_struct seq tuple tuple_struct map struct identifier
        newtype_struct
    }
}

#[doc(hidden)]
struct JsArrayAccess<'e> {
    env: Env,
    array: JsArray<'e>,
    index: u32,
    length: u32,
}

#[doc(hidden)]
impl<'e> JsArrayAccess<'e> {
    fn new(env: Env, array: JsArray<'e>) -> Self {
        let length = array.len().unwrap() as u32;
        JsArrayAccess {
            env,
            array,
            index: 0,
            length
        }
    }
}

#[doc(hidden)]
impl<'e, 'de> SeqAccess<'de> for JsArrayAccess<'e> {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.index >= self.length {
            return Ok(None);
        }
        let value = self.array.get(self.index)?;
        self.index += 1;

        let de = Deserializer::new(self.env, value);
        seed.deserialize(de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some((self.length - self.index) as usize)
    }
}

#[doc(hidden)]
struct JsObjectAccess<'e> {
    env: Env,
    object: JsObject<'e>,
    props: VecDeque<JsAny<'e>>,
}

#[doc(hidden)]
impl<'e> JsObjectAccess<'e> {
    fn new(env: Env, object: JsObject<'e>) -> Result<Self> {
        let props = VecDeque::from(object.get_property_names_any().unwrap());

        Ok(JsObjectAccess {
            env,
            object,
            props,
        })
    }
}

#[doc(hidden)]
impl<'e, 'de> MapAccess<'de> for JsObjectAccess<'e> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.props.is_empty() {
            return Ok(None)
        }

        let prop = self.props.front().map(|v| v.clone()).unwrap();
        let de = Deserializer::new(self.env, prop);
        seed.deserialize(de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if self.props.is_empty() {
            panic!("Fetching value with empty props");
        }
        let key = self.props.pop_front().unwrap();
        let value = self.object.get(key.get_value()).unwrap();

        let de = Deserializer::new(self.env, value);
        seed.deserialize(de)
    }

    fn next_entry_seed<K, V>(&mut self, kseed: K, vseed: V) -> Result<Option<(K::Value, V::Value)>>
    where
        K: DeserializeSeed<'de>,
        V: DeserializeSeed<'de>,
    {
        if self.props.is_empty() {
            return Ok(None);
        }
        let key = self.props.pop_front().unwrap();
        let value = self.object.get(key.get_value()).unwrap();

        let de = Deserializer::new(self.env, key);
        let key = kseed.deserialize(de)?;

        let de = Deserializer::new(self.env, value);
        let value = vseed.deserialize(de)?;

        Ok(Some((key, value)))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.props.len())
    }
}

#[doc(hidden)]
struct JsEnumAccess<'e> {
    env: Env,
    variant: String,
    value: Option<JsAny<'e>>,
}

#[doc(hidden)]
impl<'e> JsEnumAccess<'e> {
    fn new(env: Env, key: String, value: Option<JsAny<'e>>) -> Self {
        JsEnumAccess {
            env,
            variant: key,
            value,
        }
    }
}

#[doc(hidden)]
impl<'e, 'de> EnumAccess<'de> for JsEnumAccess<'e> {
    type Error = DeserializeError;
    type Variant = JsVariantAccess<'e>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        use serde::de::IntoDeserializer;
        let variant = self.variant.into_deserializer();
        let variant_access = JsVariantAccess::new(self.env, self.value);
        seed.deserialize(variant).map(|v| (v, variant_access))
    }
}

#[doc(hidden)]
struct JsVariantAccess<'e> {
    env: Env,
    value: Option<JsAny<'e>>,
}

#[doc(hidden)]
impl<'e> JsVariantAccess<'e> {
    fn new(env: Env, value: Option<JsAny<'e>>) -> Self {
        JsVariantAccess { env, value }
    }
}

#[doc(hidden)]
impl<'e, 'de> VariantAccess<'de> for JsVariantAccess<'e> {
    type Error = DeserializeError;

    fn unit_variant(self) -> Result<()> {
        match self.value {
            Some(val) => {
                let deserializer = Deserializer::new(self.env, val);
                serde::de::Deserialize::deserialize(deserializer)
            }
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(val) => {
                let deserializer = Deserializer::new(self.env, val);
                seed.deserialize(deserializer)
            }
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(JsAny::Array(a)) => {
                let mut deserializer = JsArrayAccess::new(self.env, a);
                visitor.visit_seq(&mut deserializer)
            },
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V,) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(JsAny::Object(o)) => {
                let mut deserializer = JsObjectAccess::new(self.env, o)?;
                visitor.visit_map(&mut deserializer)
            },
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}
