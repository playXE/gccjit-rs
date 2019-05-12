use crate::ctx::Context;
use crate::field;
use crate::field::Field;
use crate::location;
use crate::location::Location;
use crate::object;
use crate::object::{Object, ToObject};
use crate::rvalue;
use crate::rvalue::{RValue, ToRValue};
use gccjit_sys;
use std::fmt;
use std::marker::PhantomData;
use std::ptr;

/// An LValue in gccjit represents a value that has a concrete
/// location in memory. A LValue can be converted into an RValue
/// through the ToRValue trait.
/// It is also possible to get the dress of an LValue.
#[derive(Copy, Clone)]
pub struct LValue {
    ptr: *mut gccjit_sys::gcc_jit_lvalue,
}

/// ToLValue is a trait implemented by types that can be converted (or treated
/// as) LValues.
pub trait ToLValue {
    fn to_lvalue(&self) -> LValue;
}

impl ToObject for LValue {
    fn to_object(&self) -> Object {
        unsafe { object::from_ptr(gccjit_sys::gcc_jit_lvalue_as_object(self.ptr)) }
    }
}

impl fmt::Debug for LValue {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

impl ToLValue for LValue {
    fn to_lvalue(&self) -> LValue {
        unsafe { from_ptr(self.ptr) }
    }
}

impl ToRValue for LValue {
    fn to_rvalue(&self) -> RValue {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_lvalue_as_rvalue(self.ptr);
            rvalue::from_ptr(ptr)
        }
    }
}

impl LValue {
    /// Given an LValue x and a Field f, gets an LValue for the field
    /// access x.f.
    pub fn access_field(&self, loc: Option<Location>, field: Field) -> LValue {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr =
                gccjit_sys::gcc_jit_lvalue_access_field(self.ptr, loc_ptr, field::get_ptr(&field));
            from_ptr(ptr)
        }
    }

    /// Given an LValue x, returns the RValue address of x, akin to C's &x.
    pub fn get_address(&self, loc: Option<Location>) -> RValue {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_lvalue_get_address(self.ptr, loc_ptr);
            rvalue::from_ptr(ptr)
        }
    }
}

pub unsafe fn from_ptr(ptr: *mut gccjit_sys::gcc_jit_lvalue) -> LValue {
    LValue { ptr: ptr }
}

pub unsafe fn get_ptr(lvalue: &LValue) -> *mut gccjit_sys::gcc_jit_lvalue {
    lvalue.ptr
}
