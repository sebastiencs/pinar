
use std::marker::PhantomData;
use napi_sys::*;
use crate::prelude::*;
use crate::Result;

/// A Javascript array object.
pub struct JsArray<'e> {
    pub(crate) value: Value,
    pub(crate) phantom: PhantomData<&'e ()>
}

impl<'e> JsArray<'e> {
    /// Returns true if the array has a length of 0.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Returns the number of values in the array.
    pub fn len(&self) -> Result<usize> {
        let mut len: u32 = 0;
        napi_call!(napi_get_array_length(
            self.value.env(),
            self.value.get(),
            &mut len as *mut u32
        ))?;
        Ok(len as usize)
    }
    
    /// Returns an iterator over the values in the array.
    ///
    /// # Examples
    ///
    /// ```
    /// fn my_func(array: JsArray) -> JsResult<()> {
    ///     for elem in array.iter()? {
    ///         // ...
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn iter(&self) -> Result<JsArrayIterator> {
        Ok(JsArrayIterator {
            index: 0,
            array: self,
            len: self.len()?
        })
    }

    /// Set an element on the array.
    ///
    /// # Examples
    ///
    /// ```
    /// fn my_func(array: JsArray) -> JsResult<()> {
    ///     array.set(2, "some_string")?;
    ///     array.set(3, HashMap::new())?;
    ///     array.set(10, Arc::new(2))?;
    ///     Ok(())
    /// }
    /// ```
    pub fn set<V>(&self, index: u32, value: V) -> Result<()>
    where
        V: ToJs<'e>
    {
        let value = value.to_js(self.value.env)?.get_value();
        napi_call!(napi_set_element(
            self.value.env(),
            self.value.get(),
            index,
            value.get()
        ))?;
        Ok(())
    }

    /// Similar to [`set`] but takes a reference on the JS value
    ///
    /// [`set`]: #method.set
    pub(crate) fn set_ref<V>(&self, index: u32, value: &V) -> Result<()>
    where
        V: ToJs<'e>
    {
        let value = value.to_js(self.value.env)?.get_value();
        napi_call!(napi_set_element(
            self.value.env(),
            self.value.get(),
            index,
            value.get()
        ))?;
        Ok(())
    }

    /// Returns the value at the requested index.  
    ///
    /// # Examples
    ///
    /// ```
    /// fn my_func(array: JsArray) -> JsResult<()> {
    ///     let fun = array.get(1)?.as_jsfunction()?;
    ///     fun.call("hello")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn get(&self, index: u32) -> Result<JsAny<'e>> {
        JsAny::from(self.get_value(index)?)
    }

    /// Returns the raw value at the requested index.
    fn get_value(&self, index: u32) -> Result<Value> {
        let mut value = Value::new(self.value.env);
        napi_call!(napi_get_element(
            self.value.env(),
            self.value.get(),
            index, value.get_mut()
        ))?;
        Ok(value)
    }

    /// Returns a Vec of all values in the array. 
    pub(crate) fn values(&self) -> Result<Vec<Value>> {
        let len = self.len()?;
        let mut vec = Vec::with_capacity(len);
        for i in 0..len {
            vec.push(self.get_value(i as u32)?);
        }
        Ok(vec)
    }

    /// Similar to [`values`] but transform values with `fun`
    ///
    /// [`values`]: #method.values
    pub(crate) fn with_values<T>(&self, fun: impl Fn(Value) -> Result<T>) -> Result<Vec<T>> {
        let len = self.len()?;
        let mut vec = Vec::with_capacity(len);
        for i in 0..len {
            vec.push(fun(self.get_value(i as u32)?)?);
        }
        Ok(vec)
    }
}

/// Array iterator.
///
/// This struct is created by the [`iter`] method on [`JsArray`]
///
/// [`iter`]: struct.JsArray.html#method.iter
/// [`JsArray`]: struct.JsArray.html
pub struct JsArrayIterator<'a, 'e> {
    index: usize,
    len: usize,
    array: &'a JsArray<'e>
}

impl<'a, 'e> Iterator for JsArrayIterator<'a, 'e> {
    type Item = JsAny<'e>;
    fn next(&mut self) -> Option<Self::Item> {
        // We loop on the array in case self.array.get returns None
        loop {
            let index = self.index;
            if index >= self.len {
                return None;
            }
            self.index += 1;
            if let Ok(item) = self.array.get(index as u32) {
                return Some(item);
            }
        }
    }
}
