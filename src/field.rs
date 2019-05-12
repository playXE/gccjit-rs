use std::fmt;
use std::marker::PhantomData;

use crate::ctx::Context;
use crate::object;
use crate::object::{Object, ToObject};

/// Field represents a field that composes structs or unions. A number of fields
/// can be combined to create either a struct or a union.
#[derive(Copy, Clone)]
pub struct Field {
    ptr: *mut gccjit_sys::gcc_jit_field,
}

impl ToObject for Field {
    fn to_object(&self) -> Object {
        unsafe { object::from_ptr(gccjit_sys::gcc_jit_field_as_object(self.ptr)) }
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

pub unsafe fn from_ptr(ptr: *mut gccjit_sys::gcc_jit_field) -> Field {
    Field { ptr: ptr }
}

pub unsafe fn get_ptr(f: &Field) -> *mut gccjit_sys::gcc_jit_field {
    f.ptr
}
