
use napi_sys::*;
use crate::prelude::*;
use crate::Result;

pub struct JsArray {
    pub(crate) value: Value
}

impl JsArray {
    pub fn iter(&self) -> Result<JsArrayIterator> {
        Ok(JsArrayIterator {
            index: 0,
            array: self,
            len: self.len()?
        })
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
        V: ToJs
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

    pub fn get(&self, index: u32) -> Result<JsUnknown>
    {
        let mut value = Value::new(self.value.env);
        unsafe {
            Status::result(napi_get_element(
                self.value.env(),
                self.value.get(),
                index, value.get_mut())
            )?;
        }
        JsUnknown::from(value)
    }
}

pub struct JsArrayIterator<'e> {
    index: usize,
    len: usize,
    array: &'e JsArray
}

impl<'e> Iterator for JsArrayIterator<'e> {
    type Item = JsUnknown;
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
