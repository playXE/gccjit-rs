use crate::ctx::Context;
use crate::lvalue;
use crate::lvalue::{LValue, ToLValue};
use crate::object;
use crate::object::{Object, ToObject};
use crate::rvalue;
use crate::rvalue::{RValue, ToRValue};
use gccjit_sys;
use std::fmt;
use std::marker::PhantomData;

/// Parameter represents a parameter to a function. A series of parameteres
/// can be combined to form a function signature.
#[derive(Copy, Clone)]
pub struct Parameter {
    ptr: *mut gccjit_sys::gcc_jit_param,
}

impl ToObject for Parameter {
    fn to_object(&self) -> Object {
        unsafe { object::from_ptr(gccjit_sys::gcc_jit_param_as_object(self.ptr)) }
    }
}

impl fmt::Debug for Parameter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

impl ToRValue for Parameter {
    fn to_rvalue(&self) -> RValue {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_param_as_rvalue(self.ptr);
            rvalue::from_ptr(ptr)
        }
    }
}

impl ToLValue for Parameter {
    fn to_lvalue(&self) -> LValue {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_param_as_lvalue(self.ptr);
            lvalue::from_ptr(ptr)
        }
    }
}

pub unsafe fn from_ptr(ptr: *mut gccjit_sys::gcc_jit_param) -> Parameter {
    Parameter { ptr: ptr }
}

pub unsafe fn get_ptr(loc: &Parameter) -> *mut gccjit_sys::gcc_jit_param {
    loc.ptr
}
