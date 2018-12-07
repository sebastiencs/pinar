
use std::sync::Arc;
use std::rc::Rc;
use std::any::TypeId;

pub(crate) enum PtrKind<T> {
    Box(Option<Box<T>>),
    Rc(Rc<T>),
    Arc(Arc<T>)
}

pub(crate) struct External<T> {
    pub(crate) ptr: PtrKind<T>,
    pub(crate) id: TypeId
}

impl<T: 'static> External<T> {
    pub(crate) fn new_box(ptr: Box<T>) -> External<T> {
        External {
            ptr: PtrKind::Box(Some(ptr)),
            id: TypeId::of::<T>()
        }
    }
    pub(crate) fn new_rc(ptr: Rc<T>) -> External<T> {
        External {
            ptr: PtrKind::Rc(ptr),
            id: TypeId::of::<T>()
        }
    }
    pub(crate) fn new_arc(ptr: Arc<T>) -> External<T> {
        External {
            ptr: PtrKind::Arc(ptr),
            id: TypeId::of::<T>()
        }
    }

    pub(crate) fn take_box<O: 'static>(&mut self) -> Option<Box<T>> {
        if self.id != TypeId::of::<O>() {
            panic!("Trying to take a Box of a different type");
        }
        match self.ptr {
            PtrKind::Box(ref mut p) => p.take(),
            PtrKind::Rc(_) => panic!("Trying to take a Box but it's a Rc"),
            PtrKind::Arc(_) => panic!("Trying to take a Box but it's an Arc"),
        }
    }

    pub(crate) fn get_rc<O: 'static>(&mut self) -> Rc<T> {
        if self.id != TypeId::of::<O>() {
            panic!("Trying to get a Rc of a different type");
        }
        match self.ptr {
            PtrKind::Rc(ref p) => Rc::clone(p),
            PtrKind::Box(_) => panic!("Trying to take a Rc but it's a Box"),
            PtrKind::Arc(_) => panic!("Trying to take a Rc but it's an Arc"),
        }
    }

    pub(crate) fn get_arc<O: 'static>(&mut self) -> Arc<T> {
        if self.id != TypeId::of::<O>() {
            panic!("Trying to get an Arc of a different type");
        }
        match self.ptr {
            PtrKind::Arc(ref p) => Arc::clone(&p),
            PtrKind::Box(_) => panic!("Trying to take an Arc but it's a Box"),
            PtrKind::Rc(_) => panic!("Trying to take an Arc but it's a Rc"),
        }
    }
}
