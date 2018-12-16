
use std::marker::PhantomData;
use napi_sys::*;
use crate::prelude::*;
use crate::Result;

pub struct JsArray<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

impl<'e> JsArray<'e> {
    pub fn iter(&self) -> Result<JsArrayIterator> {
        Ok(JsArrayIterator {
            index: 0,
            array: self,
            len: self.len()?
        })
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn len(&self) -> Result<usize> {
        let mut len: u32 = 0;
        unsafe {
            Status::result(napi_get_array_length(
                self.value.env(),
                self.value.get(),
                &mut len as *mut u32)
            )?;
        }
        Ok(len as usize)
    }

    pub fn set<V>(&self, index: u32, value: V) -> Result<()>
    where
        V: ToJs<'e>
    {
        let value = value.to_js(&self.value.env)?.get_value();
        unsafe {
            Status::result(napi_set_element(
                self.value.env(),
                self.value.get(),
                index,
                value.get())
            )?;
        };
        Ok(())
    }

    pub fn get(&self, index: u32) -> Result<JsAny<'e>> {
        let mut value = Value::new(self.value.env);
        unsafe {
            Status::result(napi_get_element(
                self.value.env(),
                self.value.get(),
                index, value.get_mut())
            )?;
        }
        JsAny::from(value)
    }

    fn get_value(&self, index: u32) -> Result<Value> {
        let mut value = Value::new(self.value.env);
        unsafe {
            Status::result(napi_get_element(
                self.value.env(),
                self.value.get(),
                index, value.get_mut())
            )?;
        }
        Ok(value)
    }

    pub(crate) fn values(&self) -> Result<Vec<Value>> {
        let len = self.len()?;
        let mut vec = Vec::with_capacity(len);
        for i in 0..len {
            vec.push(self.get_value(i as u32)?);
        }
        Ok(vec)
    }
}

pub struct JsArrayIterator<'a, 'e> {
    index: usize,
    len: usize,
    array: &'a JsArray<'e>
}

impl<'a, 'e> Iterator for JsArrayIterator<'a, 'e> {
    type Item = JsAny<'e>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let index = self.index;
            self.index += 1;
            if index >= self.len {
                return None;
            }
            if let Ok(item) = self.array.get(index as u32) {
                return Some(item);
            }
        }
    }
}
