
use std::cell::Cell;
use std::fmt;
use crate::prelude::*;
use crate::*;
//use super::SerializedObject;

use serde::ser::{self, Serialize};

#[derive(Debug)]
pub struct SerializeError {
    msg: String
}

impl From<crate::Error> for SerializeError {
    fn from(o: crate::Error) -> SerializeError {
        SerializeError {
            msg: format!("{:?}", o)
        }
    }
}

impl From<Status> for SerializeError {
    fn from(o: Status) -> SerializeError {
        SerializeError {
            msg: format!("{:?}", o)
        }
    }
}

type Result<T> = std::result::Result<T, SerializeError>;

impl ser::Error for SerializeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self { msg: msg.to_string() }
        //Status::QueueFull
    }
}

impl fmt::Display for SerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for SerializeError {
    fn description(&self) -> &str {
        "status error"
        //self.message.as_str()
        //self.message.as_str()
    }
}

#[inline]
pub fn serialize_to_js<V>(env: &Env, value: &V) -> Result<Value>
where
    V: Serialize + ?Sized,
{
    let serializer = PinarSerializer {
        env,
    };
    value.serialize(serializer)
}

#[doc(hidden)]
pub struct PinarSerializer<'e>
{
    env: &'e Env,
}

#[doc(hidden)]
pub struct PinarArraySer<'e> {
    env: &'e Env,
    current_index: Cell<usize>,
    array: JsArray,
    name_obj: Option<&'static str>
}

impl<'e> PinarArraySer<'e> {
    fn new(env: &'e Env, length: usize, name: Option<&'static str>) -> Result<PinarArraySer<'e>> {
        Ok(PinarArraySer {
            env,
            current_index: Cell::new(0),
            array: env.array_with_capacity(length)?,
            name_obj: name
        })
    }
}

pub struct PinarMapSer<'e> {
    env: &'e Env,
    obj: JsObject,
    key: Option<Value>,
    name_obj: Option<&'static str>
}

impl<'e> PinarMapSer<'e> {
    fn new(env: &'e Env, name: Option<&'static str>) -> Result<PinarMapSer<'e>> {
        Ok(PinarMapSer {
            env,
            obj: env.object()?,
            key: None,
            name_obj: name
        })
    }
}

// impl<'e> PinarSerializer<'e> {
//     fn new_array() -> PinarSerializer<'e> {

//     }
// }

impl<'e> ser::Serializer for PinarSerializer<'e> {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = Value;

    // The error type when some error occurs during serialization.
    type Error = SerializeError;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = PinarArraySer<'e>;
    type SerializeTuple = PinarArraySer<'e>;
    type SerializeTupleStruct = PinarArraySer<'e>;
    type SerializeTupleVariant = PinarArraySer<'e>;
    type SerializeMap = PinarMapSer<'e>;
    type SerializeStruct = PinarMapSer<'e>;
    type SerializeStructVariant = PinarMapSer<'e>;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(self.env.boolean(v)?.get_value())
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(self.env.number(v as i64)?.get_value())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(self.env.double(v)?.get_value())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(self.env.string(v)?.get_value())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(self.env.null()?.get_value())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        let obj = self.env.object()?;
        obj.set(variant, serialize_to_js(&self.env, value)?)?;
        Ok(obj.get_value())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        PinarArraySer::new(self.env, len.unwrap_or(0), None)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        PinarArraySer::new(self.env, len, None)
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        PinarArraySer::new(self.env, len, None)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        PinarArraySer::new(self.env, len, Some(variant))
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        PinarMapSer::new(self.env, None)
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        PinarMapSer::new(self.env, None)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        PinarMapSer::new(self.env, Some(variant))
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'e> ser::SerializeSeq for PinarArraySer<'e> {
    // Must match the `Ok` type of the serializer.
    type Ok = Value;
    // Must match the `Error` type of the serializer.
    type Error = SerializeError;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let index = self.current_index.get();
        self.array.set(index as u32, serialize_to_js(self.env, value)?)?;
        self.current_index.set(index + 1);
        Ok(())
    }

    // Close the sequence.
    fn end(self) -> Result<Self::Ok> {
        Ok(self.array.get_value())
    }
}
// Same thing but for tuples.
impl<'e> ser::SerializeTuple for PinarArraySer<'e> {
    type Ok = Value;
    type Error = SerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

// Same thing but for tuple structs.
impl<'e> ser::SerializeTupleStruct for PinarArraySer<'e> {
    type Ok = Value;
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'e> ser::SerializeTupleVariant for PinarArraySer<'e> {
    type Ok = Value;
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        let name = self.name_obj.unwrap();
        let obj = self.env.object()?;
        obj.set(name, self.array)?;
        Ok(obj.get_value())
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'e> ser::SerializeMap for PinarMapSer<'e> {
    type Ok = Value;
    type Error = SerializeError;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This cen be done by using a different PinarSerializer to serialkey<'e
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.key = Some(serialize_to_js(self.env, key)?);
        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.obj.set(self.key.unwrap(), serialize_to_js(self.env, value)?)?;
        Ok(())
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        self.obj.set(serialize_to_js(self.env, key)?, serialize_to_js(self.env, value)?)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.obj.get_value())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'e> ser::SerializeStruct for PinarMapSer<'e> {
    type Ok = Value;
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeMap::end(self)
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'e> ser::SerializeStructVariant for PinarMapSer<'e> {
    type Ok = Value;
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        let name = self.name_obj.unwrap();
        let obj = self.env.object()?;
        obj.set(name, self.obj)?;
        Ok(obj.get_value())
    }
}
