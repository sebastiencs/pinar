
use crate::*;

pub trait MultiJs {
    type Result: IntoIterator<Item = Value>;
    fn make_iter(self, env: &Env) -> Result<Self::Result>;
}

impl MultiJs for ()
{
    type Result = Vec<Value>;
    fn make_iter(self, _env: &Env) -> Result<Self::Result> {
        Ok(vec![])
    }
}

impl<A> MultiJs for A
where
    A: AsJs,
{
    type Result = Vec<Value>;
    fn make_iter(self, env: &Env) -> Result<Self::Result> {
        Ok(vec![
            self.as_js(env)?.get_value(),
        ])
    }
}

macro_rules! multi_js_tuples {
    (
        $( ( $($tuple:ident, $n:tt),* ) ),*
    ) => {
        $(
            impl<$($tuple),*> MultiJs for ($($tuple,)*)
            where
                $($tuple : AsJs,)*
            {
                type Result = Vec<Value>;
                fn make_iter(self, env: &Env) -> Result<Self::Result> {
                    Ok(vec![
                        $(self.$n.as_js(env)?.get_value(),)*
                    ])
                }
            }
        )*
    }
}

multi_js_tuples!(
    (A,0),
    (A,0,B,1),
    (A,0,B,1,C,2),
    (A,0,B,1,C,2,D,3),
    (A,0,B,1,C,2,D,3,E,4),
    (A,0,B,1,C,2,D,3,E,4,F,5),
    (A,0,B,1,C,2,D,3,E,4,F,5,G,6),
    (A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7),
    (A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8),
    (A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8,J,9),
    (A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8,J,9,K,10),
    (A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8,J,9,K,10,L,11),
    (A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8,J,9,K,10,L,11,M,12),
    (A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8,J,9,K,10,L,11,M,12,N,13),
    (A,0,B,1,C,2,D,3,E,4,F,5,G,6,H,7,I,8,J,9,K,10,L,11,M,12,N,13,O,14)
);
