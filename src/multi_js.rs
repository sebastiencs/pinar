
use crate::*;

/// Trait to convert one or multiple value(s) to javascript.
///
/// Is it implemented for
/// - `T` where `T:`[`ToJs`]
/// - tuples where every elements implement [`ToJs`]
///
pub trait MultiJs {
    #[doc(hidden)]
    fn make_values(self, env: Env) -> JsResult<MultiValue>;
}

impl<'e, A> MultiJs for A
where
    A: ToJs<'e>,
{
    fn make_values(self, env: Env) -> JsResult<MultiValue> {
        let mut values: [napi_value; 12] = [std::ptr::null_mut(); 12];

        values[0] = self.to_js(env)?.get_value().value;

        Ok(MultiValue {
            values,
            len: 1
        })
    }
}

/// Opaque structure containing multiple javascript values
pub struct MultiValue {
    values: [napi_value; 12],
    len: usize
}

impl MultiValue {
    pub(crate) fn as_ptr(&self) -> *const napi_value {
        self.values.as_ptr()
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }
}

// Don't use this anymore to avoid a memcpy
//
// impl From<&[napi_value]> for MultiValue {
//     fn from(array: &[napi_value]) -> MultiValue {
//         let mut values: [napi_value; 12] = std::mem::zeroed();
//         let len = array.len();
//         values[..len].copy_from_slice(array);
//         MultiValue {
//             values,
//             len
//         }
//     }
// }

macro_rules! multi_js_tuples {
    (
        $( ($len:tt ; $($tuple:ident, $n:tt),* ) ),*
    ) => {
        $(
            impl<'e, $($tuple),*> MultiJs for ($($tuple,)*)
            where
                $($tuple : ToJs<'e>,)*
            {
                #[allow(unused_variables, unused_mut)]
                fn make_values(self, env: Env) -> JsResult<MultiValue> {
                    let mut values: [napi_value; 12] = [std::ptr::null_mut(); 12];

                    $(values[$n] = self.$n.to_js(env)?.get_value().value;)*

                    Ok(MultiValue {
                        values,
                        len: $len
                    })
                }
            }
        )*
    }
}

multi_js_tuples!(
    (0;),
    (1;A,0),
    (2;A,0,B,1),
    (3;A,0,B,1,C,2),
    (4;A,0,B,1,C,2,D,3),
    (5;A,0,B,1,C,2,D,3,E,4),
    (6;A,0,B,1,C,2,D,3,E,4,F,5),
    (7;A,0,B,1,C,2,D,3,E,4,F,5,G,6),
    (8;A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7),
    (9;A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8),
    (10;A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8,J,9),
    (11;A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8,J,9,K,10),
    (12;A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8,J,9,K,10,L,11)
);
