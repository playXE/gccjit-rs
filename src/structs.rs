use gccjit_sys;

use std::fmt;
use std::marker::PhantomData;
use std::ptr;

use crate::ctx::Context;
use crate::field;
use crate::field::Field;
use crate::location;
use crate::location::Location;
use crate::object::{Object, ToObject};
use crate::ty as types;
use crate::ty::Type;

/// A Struct is gccjit's representation of a composite type. Despite the name,
/// Struct can represent either a struct, an union, or an opaque named type.
#[derive(Copy, Clone)]
pub struct Struct {
    ptr: *mut gccjit_sys::gcc_jit_struct,
}

impl Struct {
    pub fn as_type(&self) -> Type {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_struct_as_type(self.ptr);
            types::from_ptr(ptr)
        }
    }

    pub fn set_fields(&self, location: Option<Location>, fields: &[Field]) {
        let loc_ptr = match location {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        let num_fields = fields.len() as i32;
        let mut fields_ptrs: Vec<_> = fields
            .iter()
            .map(|x| unsafe { field::get_ptr(&x) })
            .collect();
        unsafe {
            gccjit_sys::gcc_jit_struct_set_fields(
                self.ptr,
                loc_ptr,
                num_fields,
                fields_ptrs.as_mut_ptr(),
            );
        }
    }
}

impl ToObject for Struct {
    fn to_object(&self) -> Object {
        let ty = self.as_type();
        ty.to_object()
    }
}

impl fmt::Debug for Struct {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.as_type();
        obj.fmt(fmt)
    }
}

pub unsafe fn from_ptr(ptr: *mut gccjit_sys::gcc_jit_struct) -> Struct {
    Struct {
        
        ptr: ptr,
    }
}
