
use std::rc::Rc;
use std::ffi::c_void;
use napi_sys::*;
use std::ffi::CString;
use std::any::{TypeId};

use crate::prelude::*;
use crate::error::JsClassError;
use crate::Result;

pub trait JsClass : Sized + 'static {
    const CLASSNAME: &'static str;
    type ArgsConstructor: FromArguments;

    fn constructor(args: Self::ArgsConstructor) -> Result<Self> ;

    fn default_properties(builder: ClassBuilder<Self>) -> ClassBuilder<Self> { builder }
    fn new_instance(env: &Env, args: Self::ArgsConstructor) -> Result<JsObject> {
        ClassBuilder::<Self>::new_instance(env, args)
    }
}

trait JsClassInternal {
    unsafe extern "C" fn __pinar_class_constructor(env: napi_env, cb_info: napi_callback_info) -> napi_value;
    unsafe extern "C" fn __pinar_class_dispatch(env: napi_env, cb_info: napi_callback_info) -> napi_value;
    unsafe extern "C" fn __pinar_nop(env: napi_env, cb_info: napi_callback_info) -> napi_value;
    const CLASS_DATA: &'static str;
    const PINAR_CLASS_ID: &'static str;
}

pub unsafe extern "C" fn __pinar_drop_box<T>(_env: napi_env, data: *mut c_void, _finalize_hint: *mut c_void) {
    // println!("DROPPING BOX {:?} {:x?}", std::intrinsics::type_name::<T>(), data);
    Box::<T>::from_raw(data as *mut T);
}

pub unsafe extern "C" fn __pinar_drop_rc<T>(_env: napi_env, data: *mut c_void, _finalize_hint: *mut c_void) {
    // println!("DROPPING RC {:?} {:x?}", std::intrinsics::type_name::<T>(), data);
    Rc::<T>::from_raw(data as *mut T);
}

pub(crate) fn execute_safely<F>(env: napi_env, closure: F) -> napi_value
where
    F: Fn() -> Result<Option<napi_value>>,
    F: std::panic::UnwindSafe
{
    match std::panic::catch_unwind(closure) {
        Ok(Ok(Some(v))) => v,
        Ok(Ok(None)) => std::ptr::null_mut(),
        Ok(Err(e)) => {
            let env = Env::from(env);
            let e = e.as_js_error();
            env.throw_error(e.get_msg(), e.get_code()).ok();
            std::ptr::null_mut()
        }
        Err(e) => {
            let env = Env::from(env);
            env.throw_error(format!("Rust has panicked ! {:?}", e),
                            Some("PINAR".to_owned())).ok();
            std::ptr::null_mut()
        }
    }
}

impl<C: 'static +  JsClass> JsClassInternal for C {
    const CLASS_DATA: &'static str = "__pinar_class_data__";
    const PINAR_CLASS_ID: &'static str = "___pinar___class___id___";

    unsafe extern "C" fn __pinar_class_constructor(env: napi_env, cb_info: napi_callback_info) -> napi_value {
        execute_safely(env, || {
            let env = Env::from(env);
            let (class_data, args) = env.callback_info::<JsClassData<Self>>(cb_info)?;
            let class_data = Rc::from_raw(class_data);

            // 1 JsClassData for each JS instance, + 1 for the JS constructor
            let mut copy_class_data = Rc::clone(&class_data);
            std::mem::forget(class_data);
            let this = args.this();

            if this.has_property(C::PINAR_CLASS_ID)? == false {
                return Err(JsClassError::ThisConstructor(C::CLASSNAME).into())
            }

            let class = if copy_class_data.args_rust.is_some() {
                let args_rust = Rc::make_mut(&mut copy_class_data).args_rust.take().unwrap();
                Self::constructor(args_rust)?
            } else {
                Self::constructor(FromArguments::from_args(&args)?)?
            };

            let class = Box::new(class);

            this.define_property(PropertyDescriptor::value(
                &env,
                Self::CLASS_DATA,
                copy_class_data
            )?)?;

            Status::result(napi_wrap(
                env.env(),
                this.get_value().value,
                Box::into_raw(class) as *mut c_void,
                Some( __pinar_drop_box::<C>),
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ))?;

            Ok(Some(this.get_value().value))
        })
    }

    unsafe extern "C" fn __pinar_class_dispatch(env: napi_env, cb_info: napi_callback_info) -> napi_value {
        execute_safely(env, || {
            let env = Env::from(env);
            let (key, args) = env.callback_info::<usize>(cb_info)?;

            let this = args.this();
            let property = this.get(Self::CLASS_DATA)?;
            let this = this.napi_unwrap::<Self>()?;

            if let JsAny::External(e) = property {
                let external = e.get_rc::<JsClassData<Self>>()?;

                if external.id != TypeId::of::<Self>() {
                    return Err(JsClassError::WrongClass.into());
                }

                if let Some(handler) = external.methods.get(key as usize) {
                    return handler.handle(&mut *this, &args);
                }
                return Err(JsClassError::WrongHandler.into());
            };
            Err(JsClassError::ExternalClassData.into())
        })
    }

    unsafe extern "C" fn __pinar_nop(_env: napi_env, _cb_info: napi_callback_info) -> napi_value {
        std::ptr::null_mut()
    }
}

pub struct JsClassData<C: JsClass> {
    id: TypeId,
    args_rust: Option<C::ArgsConstructor>,
    methods: Vec<Rc<ClassMethodHandler<C>>>
}

// Implement Clone ourself because ArgsConstructor might not be clonable
impl<C: JsClass> Clone for JsClassData<C> {
    fn clone(&self) -> Self {
        JsClassData {
            id: self.id,
            args_rust: None,
            methods: self.methods.clone()
        }
    }
}

pub struct ClassBuilder<C: JsClass> {
    props: Vec<ClassProperty<C>>,
    name: String
}

pub struct ClassProperty<C: JsClass> {
    name: CString,
    method: Option<Rc<ClassMethodHandler<C>>>,
    accessor: Option<Rc<ClassMethodHandler<C>>>,
}

impl<C: JsClass + 'static> ClassProperty<C> {
    pub fn method<S, A, R, Method>(name: S, method: Method) -> ClassProperty<C>
    where
        S: AsRef<str>,
        A: FromArguments + 'static,
        R: for <'env> JsReturn<'env> + 'static,
        Method: Fn(&C, A) -> R + 'static
    {
        ClassProperty {
            name: CString::new(name.as_ref()).unwrap(),
            method: Some(Rc::new(ClassMethod::new(method))),
            accessor: None
        }
    }

    pub fn accessor<S, A, R, Accessor>(name: S, accessor: Accessor) -> ClassProperty<C>
    where
        S: AsRef<str>,
        A: FromArguments + 'static,
        R: for <'env> JsReturn<'env> + 'static,
        Accessor: Fn(&C, Option<A>) -> R + 'static
    {
        ClassProperty {
            name: CString::new(name.as_ref()).unwrap(),
            method: None,
            accessor: Some(Rc::new(ClassMethod::new(accessor)))
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
    pub fn start_build() -> Self {
        Default::default()
    }

    pub fn with_method<S, A, R, Method>(mut self, name: S, method: Method) -> Self
    where
        S: AsRef<str>,
        A: FromArguments + 'static,
        R: for <'env> JsReturn<'env> + 'static,
        Method: Fn(&C, A) -> R + 'static
    {
        self.props.push(ClassProperty::method(name, method));
        self
    }

    pub fn with_accessor<S, A, R, Accessor>(mut self, name: S, accessor: Accessor) -> Self
    where
        S: AsRef<str>,
        A: FromArguments + 'static,
        R: for <'env> JsReturn<'env> + 'static,
        Accessor: Fn(&C, Option<A>) -> R + 'static
    {
        self.props.push(ClassProperty::accessor(name, accessor));
        self
    }

    fn create_internal<'e>(&self, env: &Env, args_rust: Option<C::ArgsConstructor>) -> Result<JsFunction<'e>> {
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
            method: Some(C::__pinar_nop),
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
            args_rust
        }));

        let mut result = Value::new(*env);
        unsafe {
            Status::result(napi_define_class(
                env.env(),
                self.name.as_ptr() as *const i8,
                self.name.len(),
                Some(C::__pinar_class_constructor),
                data_ptr as *mut c_void,
                props.len(),
                props.as_ptr(),
                result.get_mut()
            ))?;
            Status::result(napi_add_finalizer(
                env.env(),
                result.get(),
                data_ptr as *mut c_void,
                Some(__pinar_drop_rc::<JsClassData<C>>),
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ))?;
        }
        Ok(JsFunction::from(result))
    }

    pub fn create<'e>(&self, env: &Env) -> Result<JsFunction<'e>> {
        self.create_internal(env, None)
    }

    pub fn new_instance<'e>(env: &Env, args: C::ArgsConstructor) -> Result<JsObject<'e>> {
        let builder = ClassBuilder::<C>::default();
        let fun = builder.create_internal(env, Some(args))?;
        fun.new_instance(())
    }
}

pub struct SomeClass {
    number: i64
}

pub struct ClassMethod<C, A, R>
where
    C: JsClass,
    A: FromArguments,
    R: for <'env> JsReturn<'env>
{
    fun: Box<Fn(&C, A) -> R>,
}

impl<C, A, R> ClassMethod<C, A, R>
where
    C: JsClass,
    A: FromArguments,
    R: for <'env> JsReturn<'env>
{
    fn new<F>(fun: F) -> Self
    where
        F: Fn(&C, A) -> R + 'static
    {
        ClassMethod {
            fun: Box::new(fun),
        }
    }
}

pub trait ClassMethodHandler<C: JsClass> {
    fn handle(&self, this: &mut C, args: &Arguments) -> Result<Option<napi_value>>;
}

impl<C, A, R> ClassMethodHandler<C> for ClassMethod<C, A, R>
where
    C: JsClass,
    A: FromArguments,
    R: for <'env> JsReturn<'env>
{
    fn handle(&self, this: &mut C, args: &Arguments) -> Result<Option<napi_value>> {
        let env = args.env();
        let args = A::from_args(args)?;

        Ok((self.fun)(this, args)
           .get_result(env)
           .map_err(Into::into)?
           .map(|res| res.get_value().value))
    }
}

impl JsClass for SomeClass {
    const CLASSNAME: &'static str = "RustClass";
    type ArgsConstructor = (String, i64);

    fn constructor(arg: Self::ArgsConstructor) -> Result<Self> {
        Ok(SomeClass{ number: arg.1 })
    }

    fn default_properties(builder: ClassBuilder<Self>) -> ClassBuilder<Self> {
        builder.with_method("easy", SomeClass::jsfunction)
               .with_method("easy2", SomeClass::jsother)
               .with_accessor("easy3", SomeClass::jsaccessor)
    }
}

impl SomeClass {
    pub fn real(&self, a: String) {
        println!("coucou {}", a);
    }
    pub fn real2(&self, a: (String, i64)) {
        println!("coucou {:?}", a);
    }
    pub fn other(&self, a: u64, b: u64) {
        println!("coucou {} {}", a, b);
    }

    pub fn jsfunction(&self, _args: (String, i64)) -> String {
        println!("FROM JSFUNCTION {} {:x?}", self.number, self as *const SomeClass);
        "weeesh".to_owned()
    }
    pub fn jsother(&self, _args: (String, i64)) -> i64 {
        println!("FROM JSOTHER", );
        93
    }
    pub fn jsaccessor(&self, _args: Option<String>) -> i64 {
        123456
    }
    pub fn jsbox(&self, _args: Option<String>) -> Box<i64> {
        Box::new(64)
    }
}

fn test(env: Env) -> Result<()> {
    //    ClassBuilder::<SomeClass>::start_build();

    SomeClass::new_instance(&env, (String::from("seb"), 2))?;

    let _class = ClassBuilder::<SomeClass>::new_instance(&env, (String::from("seb"), 2))?;
    Ok(())
}
