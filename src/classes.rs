
use std::rc::Rc;
use std::ffi::c_void;
use napi_sys::*;
use std::ffi::CString;
use std::any::TypeId;

use crate::prelude::*;
use crate::error::JsClassError;

/// Trait to implement to create a Javascript class.
///
/// This trait is implemented with the attribute macro [`pinar`] applied on implementations.
///
/// # Example
///
/// ```
/// struct MyStruct {
///     num: i64
/// }
///
/// #[pinar]
/// impl MyStruct {
///     fn constructor(num: i64, num2: i64) -> JsResult<MyStruct> {
///         Ok(MyStruct { num: num + num2 })
///     }
///     fn my_method(&self) -> &i64 {
///         &self.num
///     }
///     fn my_other_method(&mut self, fun: JsFunction) -> JsResult<()> {
///         self.num = fun.call(())?.as_jsnumber()?.to_rust()?;
///         Ok(())
///     }
/// }
///
/// // The type MyStruct is now a class in JS, it implements the trait `JsClass`.
/// // It has a constructor and 2 other methods: my_method and my_other_method
///
/// // From javascript:
///
/// // const rust = require("./native");
/// // 
/// // const instance = new rust.MyStruct(1, 2);
/// // instance.my_method() // 3
/// ```
/// You can instantiate a JS class from Rust:
/// ```
/// #[pinar]
/// fn my_func(env: Env) -> JsResult<()> {
///     let js_instance: JsObject = MyStruct::new_instance(env, (3, 4))?;
///
///     let res: i64 = js_instance.get("my_method")?
///                               .as_jsfunction()?
///                               .call(())?
///                               .as_jsnumber()?
///                               .to_rust()?;
///     // res == 7
///     Ok(())
/// }
/// ```

/// [`pinar`]: ./attr.pinar.html
pub trait JsClass : Sized + 'static {
    /// A string naming the class in javascript.
    const CLASSNAME: &'static str;
    /// Arguments type of the class constructor.
    type ArgsConstructor: FromArguments;

    fn constructor(_args: Self::ArgsConstructor) -> JsResult<Self> {
        Err(JsClassError::NoConstructor(Self::CLASSNAME).into())
    }

    fn default_properties(builder: ClassBuilder<Self>) -> ClassBuilder<Self> {
        builder
    }

    fn new_instance<'e>(env: Env, args: Self::ArgsConstructor) -> JsResult<JsObject<'e>> {
        ClassBuilder::<Self>::new_instance(env, args)
    }
}

/// Private trait implemented on types that implement `JsClass`
trait JsClassInternal {
    const CLASS_DATA: &'static str;
    const PINAR_CLASS_ID: &'static str;

    /// Function to construct the js class
    extern "C" fn __pinar_class_constructor(env: napi_env, cb_info: napi_callback_info) -> napi_value;

    /// Function called when any method of the class is called from js.
    ///
    /// It is responsible of converting values from/to JS and call the appropriate Rust method.
    extern "C" fn __pinar_class_dispatch(env: napi_env, cb_info: napi_callback_info) -> napi_value;
}

pub(crate) unsafe extern "C" fn __pinar_drop_box<T>(_env: napi_env, data: *mut c_void, _finalize_hint: *mut c_void) {
    // println!("DROPPING BOX {:?} {:x?}", std::any::type_name::<T>(), data);
    Box::<T>::from_raw(data as *mut T);
}

pub(crate) unsafe extern "C" fn __pinar_drop_rc<T>(_env: napi_env, data: *mut c_void, _finalize_hint: *mut c_void) {
    // println!("DROPPING RC {:?} {:x?}", std::any::type_name::<T>(), data);
    Rc::<T>::from_raw(data as *mut T);
}

extern "C" fn __pinar_nop(_env: napi_env, _cb_info: napi_callback_info) -> napi_value {
    std::ptr::null_mut()
}

/// Run Rust code and handle result correctly.
#[inline(always)]
pub(crate) fn execute_safely<F>(env: napi_env, closure: F) -> napi_value
where
    F: Fn() -> JsResult<Option<Value>>,
    F: std::panic::UnwindSafe
{
    match std::panic::catch_unwind(closure) {
        Ok(Ok(Some(v))) => v.value,
        Ok(Ok(None)) => std::ptr::null_mut(),
        Ok(Err(error)) => {
            let env = Env::from(env);
            let e = error.as_js_error();
            env.throw_error(format!("{}\n{:?}", e.get_msg(), error.backtrace()), e.get_code()).ok();
            std::ptr::null_mut()
        }
        Err(_) => {
            use backtrace::Backtrace;

            let env = Env::from(env);
            let bt = crate::BACKTRACE.with(|bt| { bt.borrow_mut().take() })
                .unwrap_or_else(Backtrace::new);

            env.throw_error(format!("Rust has panicked ! {:?}", bt),
                            Some("PINAR".to_owned())).ok();

            std::ptr::null_mut()
        }
    }
}

impl<C: 'static +  JsClass> JsClassInternal for C {
    const CLASS_DATA: &'static str = "__pinar_class_data__";
    const PINAR_CLASS_ID: &'static str = "___pinar___class___id___";

    extern "C" fn __pinar_class_constructor(
        env: napi_env,
        cb_info: napi_callback_info
    ) -> napi_value
    {
        use self::JsClassError::*;

        execute_safely(env, || {
            let env = Env::from(env);
            let (class_data, args) = env.callback_info::<JsClassData<Self>>(cb_info)?;
            let mut class_data = unsafe { Rc::from_raw(class_data) };

            let class = if class_data.args_rust.is_some() {
                // Constructor has been called from Rust
                let args_rust = Rc::make_mut(&mut class_data).args_rust.take().expect("None value here");
                Self::constructor(args_rust)?
            } else if class_data.instance.is_some() {
                // Constructor has been called with the existing Rust instance
                Rc::make_mut(&mut class_data).instance.take().expect("Instance none")
            } else {
                // Constructor has been called from JS
                Self::constructor(FromArguments::from_args(&args)?)?
            };

            // 1 JsClassData for each JS instance, + 1 for the JS constructor
            let copy_class_data = Rc::clone(&class_data);
            std::mem::forget(class_data);

            let this = args.this()?
                           .as_jsobject()
                           .map_err(|_| ThisConstructor(C::CLASSNAME))?;

            if !(this.has_property(C::PINAR_CLASS_ID)?) {
                return Err(ThisConstructor(C::CLASSNAME).into())
            }

            let class = Box::new(class);

            this.define_property(PropertyDescriptor::value(
                env,
                Self::CLASS_DATA,
                copy_class_data
            )?)?;

            napi_call!(napi_wrap(
                env.env(),
                this.get_value().value,
                Box::into_raw(class) as *mut c_void,
                Some( __pinar_drop_box::<C>),
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ))?;

            Ok(Some(this.get_value()))
        })
    }

    extern "C" fn __pinar_class_dispatch(
        env: napi_env,
        cb_info: napi_callback_info
    ) -> napi_value
    {
        use self::JsClassError::*;

        execute_safely(env, || {
            let env = Env::from(env);
            let (key, args) = env.callback_info::<usize>(cb_info)?;

            let this = args.this()?
                           .as_jsobject()
                           .map_err(|_| ThisMethod(C::CLASSNAME))?;

            let external = this.get(Self::CLASS_DATA)?
                               .as_jsexternal()
                               .map_err(|_| ExternalClassData)?;

            let this = this.napi_unwrap::<Self>()?;
            let class_data = external.get_rc::<JsClassData<Self>>()?;

            if class_data.id != TypeId::of::<Self>() {
                return Err(WrongClass.into());
            }

            match class_data.methods.get(key as usize) {
                Some(method) => method.call(unsafe { &mut *this }, &args),
                _ => Err(WrongHandler.into())
            }
        })
    }
}

/// Class Data attached to the JS instance
struct JsClassData<C: JsClass> {
    id: TypeId,
    args_rust: Option<C::ArgsConstructor>,
    methods: Vec<Rc<dyn ClassMethodHandler<C>>>,
    instance: Option<C>
}

// Implement Clone ourself because ArgsConstructor might not be clonable
impl<C: JsClass> Clone for JsClassData<C> {
    fn clone(&self) -> Self {
        JsClassData {
            id: self.id,
            args_rust: None,
            methods: self.methods.clone(),
            instance: None
        }
    }
}

/// Struct holding informations (properties) of the JS class to build.
#[doc(hidden)]
pub struct ClassBuilder<C: JsClass> {
    props: Vec<ClassProperty<C>>,
    name: String
}

struct ClassProperty<C: JsClass> {
    name: CString,
    method: Option<Rc<dyn ClassMethodHandler<C>>>,
    accessor: Option<Rc<dyn ClassMethodHandler<C>>>,
}

impl<C: JsClass + 'static> ClassProperty<C> {
    pub fn method<S, A, R>(name: S, method: ClassMethod<C, A, R>) -> ClassProperty<C>
    where
        S: AsRef<str>,
        A: FromArguments + 'static,
        R: for <'env> JsReturn<'env> + 'static,
    {
        ClassProperty {
            name: CString::new(name.as_ref()).unwrap(),
            method: Some(Rc::new(method)),
            accessor: None
        }
    }

    pub fn accessor<S, A, R>(name: S, accessor: ClassMethod<C, A, R>) -> ClassProperty<C>
    where
        S: AsRef<str>,
        A: FromArguments + 'static,
        R: for <'env> JsReturn<'env> + 'static,
    {
        ClassProperty {
            name: CString::new(name.as_ref()).unwrap(),
            method: None,
            accessor: Some(Rc::new(accessor))
        }
    }
}

impl<C: JsClass> Default for ClassBuilder<C> {
    fn default() -> ClassBuilder<C> {
        let builder = ClassBuilder { name: C::CLASSNAME.to_owned(), props: vec![] };
        C::default_properties(builder)
    }
}

impl<C: JsClass + 'static> ClassBuilder<C> {
    /// Add a method to the class
    pub fn with_method<S, A, R, Method>(mut self, name: S, method: Method) -> Self
    where
        S: AsRef<str>,
        A: FromArguments + 'static,
        R: for <'env> JsReturn<'env> + 'static,
        Method: MethodFn<C, A, R> + 'static
    {
        self.props.push(ClassProperty::method(name, method.make()));
        self
    }

    /// Add an accessor to the class
    pub fn with_accessor<S, A, R, Accessor>(mut self, name: S, accessor: Accessor) -> Self
    where
        S: AsRef<str>,
        A: FromArguments + 'static,
        R: for <'env> JsReturn<'env> + 'static,
        Accessor: Fn(&mut C, Option<A>) -> R + 'static
    {
        self.props.push(ClassProperty::accessor(name, ClassMethod::new(accessor)));
        self
    }

    /// Build the class with its properties
    fn create_internal<'e>(
        &self,
        env: &Env,
        args_rust: Option<C::ArgsConstructor>,
        instance: Option<C>
    ) -> JsResult<JsFunction<'e>>
    {
        let mut props: Vec<_> = self.props.iter().enumerate().map(|(index, prop)| { napi_property_descriptor {
            utf8name: prop.name.as_ptr() as *const i8,
            name: std::ptr::null_mut(),
            method: if prop.method.is_some() { Some(C::__pinar_class_dispatch) } else { None },
            getter: if prop.accessor.is_some() { Some(C::__pinar_class_dispatch) } else { None },
            setter: if prop.accessor.is_some() { Some(C::__pinar_class_dispatch) } else { None },
            value: std::ptr::null_mut(),
            attributes: napi_property_attributes::napi_default,
            data: index as *mut std::ffi::c_void,
        }
        }).collect();

        let name = env.string(C::PINAR_CLASS_ID)?;
        props.push( napi_property_descriptor {
            utf8name: std::ptr::null_mut(),
            name: name.get_value().value,
            method: Some(__pinar_nop),
            getter: None,
            setter: None,
            value: std::ptr::null_mut(),
            attributes: napi_property_attributes::napi_default,
            data: std::ptr::null_mut(),
        });

        let data: Vec<_> = self.props.iter().map(|prop| {
            prop.method.as_ref().map(|p| Rc::clone(p)).unwrap_or_else(|| {
                prop.accessor.as_ref().map(|p| Rc::clone(&p)).unwrap()
            })
        }).collect();

        let data_ptr = Rc::into_raw(Rc::new(JsClassData {
            id: TypeId::of::<C>(),
            methods: data,
            args_rust,
            instance
        }));

        let mut result = Value::new(*env);

        napi_call!(napi_define_class(
            env.env(),
            self.name.as_ptr() as *const i8,
            self.name.len(),
            Some(C::__pinar_class_constructor),
            data_ptr as *mut c_void,
            props.len(),
            props.as_ptr(),
            result.get_mut()
        ))?;

        napi_call!(napi_add_finalizer(
            env.env(),
            result.get(),
            data_ptr as *mut c_void,
            Some(__pinar_drop_rc::<JsClassData<C>>),
            std::ptr::null_mut(),
            std::ptr::null_mut()
        ))?;

        Ok(JsFunction::from(result))
    }

    pub fn create<'e>(&self, env: &Env) -> JsResult<JsFunction<'e>> {
        self.create_internal(env, None, None)
    }

    /// Instantiate the JS class from Rust
    pub fn new_instance<'e>(env: Env, args: C::ArgsConstructor) -> JsResult<JsObject<'e>> {
        let builder = ClassBuilder::<C>::default();
        let fun = builder.create_internal(&env, Some(args), None)?;
        fun.new_instance(())
    }

    /// Instantiate the Js class from Rust with its Rust instance
    pub fn from_instance<'e>(env: Env, instance: C) -> JsResult<JsObject<'e>> {
        let builder = ClassBuilder::<C>::default();
        let fun = builder.create_internal(&env, None, Some(instance))?;
        fun.new_instance(())
    }
}

// pub struct SomeClass {
//     number: i64
// }

// TODO: Use https://github.com/rust-lang/rust/pull/55986
//       when it reaches stable

/// Trait to implement for a method of a class
pub trait MethodFn<C, A, R>
where
    C: JsClass,
    A: FromArguments,
    R: for<'env> JsReturn<'env>
{
    fn make(self) -> ClassMethod<C, A, R>;
}

macro_rules! impl_methodfn {
    (
        $( ( $($arg:ident),* ) ),*
    ) => {
        $(
            impl<$($arg,)* R, Class, Fun> MethodFn<Class, ($($arg,)*), R> for Fun
            where
                Fun: Fn(&mut Class, $($arg,)*) -> R + 'static,
                Class: JsClass,
                $($arg : FromArguments + 'static,)*
                R: for<'env> JsReturn<'env> + 'static
            {
                #[allow(non_snake_case)]
                fn make(self) -> ClassMethod<Class, ($($arg,)*), R> {
                    ClassMethod::new(move |s, ($($arg,)*)| (self)(s, $($arg,)*))
                }
            }
        )*
    }
}

impl_methodfn!(
    (),
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
    (A, B, C, D, E, F, G, H, I, J, K, L, M),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)
);

/// Struct containing the method
pub struct ClassMethod<C, A, R>
where
    C: JsClass,
    A: FromArguments,
    R: for <'env> JsReturn<'env>
{
    fun: Box<dyn Fn(&mut C, A) -> R>,
}

impl<C, A, R> ClassMethod<C, A, R>
where
    C: JsClass,
    A: FromArguments,
    R: for <'env> JsReturn<'env>
{
    fn new<F>(fun: F) -> Self
    where
        F: Fn(&mut C, A) -> R + 'static
    {
        ClassMethod {
            fun: Box::new(fun),
        }
    }
}

/// Trait to call a method of the class
trait ClassMethodHandler<C: JsClass> {
    fn call(&self, this: &mut C, args: &Arguments) -> JsResult<Option<Value>>;
}

impl<C, A, R> ClassMethodHandler<C> for ClassMethod<C, A, R>
where
    C: JsClass,
    A: FromArguments,
    R: for <'env> JsReturn<'env>
{
    fn call(&self, this: &mut C, args: &Arguments) -> JsResult<Option<Value>> {
        let env = args.env();
        let args = A::from_args(args)?;

        (self.fun)(this, args).get_result(env)
    }
}

/// Helper struct to instantiate a JS class with an existing Rust value
///
/// The type must implement [`JsClass`]
///
/// # Example
///
/// ```
/// struct MyStruct {}
///
/// #[pinar]
/// impl MyStruct { .. }
///
/// // The following functions do the same things (instantiate a JS class)
/// 
/// #[pinar]
/// fn my_func() -> AsJsClass<MyStruct> {
///     AsJsClass(MyStruct {})
/// }
/// 
/// #[pinar]
/// fn my_func(env: Env) -> JsResult<Value> {
///     MyStruct{}.to_js_class(env)
/// }
/// 
/// #[pinar]
/// fn my_func() -> JsResult<JsObject> {
///     MyStruct::new_instance(env, ())
/// }
/// ```
///
/// [`JsClass`]: ./trait.JsClass.html
pub struct AsJsClass<C: JsClass>(pub C);

impl<C> AsJsClass<C>
where
     C: JsClass
{
    pub(crate) fn to_js_class(self, env: Env) -> JsResult<Value> {
        ClassBuilder::from_instance(env, self.0).map(|c| c.get_value())
    }
}

// impl JsClass for SomeClass {
//     const CLASSNAME: &'static str = "RustClass";
//     type ArgsConstructor = (String, i64);

//     // fn constructor(arg: Self::ArgsConstructor) -> JsResult<Self> {
//     //     Ok(SomeClass {
//     //         number: arg.1
//     //     })
//     // }

//     // fn default_properties(builder: ClassBuilder<Self>) -> ClassBuilder<Self> {
//     //     builder.with_method("easy", SomeClass::jsfunction)
//     //            .with_method("easy2", SomeClass::jsother)
//     //            .with_method("real", SomeClass::real)
//     //            .with_method("real2", SomeClass::real2)
//     //            .with_method("none", SomeClass::none)
//     //            .with_method("other2", SomeClass::other2)
//     //            .with_method("obj", SomeClass::obj)
//     //            .with_accessor("easy3", SomeClass::jsaccessor)
//     // }
// }

// impl SomeClass {
//     pub fn none(&mut self) {
//         println!("coucou");
//     }
//     pub fn real(&mut self, a: String) {
//         println!("coucou {}", a);
//     }
//     pub fn real2(&mut self, a: (String, i64)) {
//         println!("coucou {:?}", a);
//     }
//     pub fn other(&mut self, a: u64, b: u64) {
//         println!("coucou {} {}", a, b);
//     }

//     pub fn other2(&mut self, a: i64, b: i64) {
//         println!("coucou {} {}", a, b);
//     }

//     pub fn obj<'e>(&mut self, env: Env) -> JsBoolean<'e> {
//         env.boolean(true).unwrap()
//     }

//     pub fn jsfunction(&mut self, _s: String, _i: i64) -> String {
//         println!("FROM JSFUNCTION {} {:x?}", self.number, self as *const SomeClass);
//         "weeesh".to_owned()
//     }
//     pub fn jsother(&mut self, _s: String, _i: i64) -> i64 {
//         println!("FROM JSOTHER", );
//         93
//     }
//     pub fn jsaccessor(&mut self, _args: Option<String>) -> i64 {
//         123456
//     }
//     pub fn jsbox(&self, _args: Option<String>) -> Box<i64> {
//         Box::new(64)
//     }
// }

// fn test(env: Env) -> JsResult<()> {
//     //    ClassBuilder::<SomeClass>::start_build();

//     SomeClass::new_instance(env, (String::from("seb"), 2))?;

//     let _class = ClassBuilder::<SomeClass>::new_instance(env, (String::from("seb"), 2))?;
//     Ok(())
// }
