use crate::ctx::Context;
use crate::object;
use crate::rvalue;
use crate::ty as types;
use crate::ty::Type;
use gccjit_sys;
use object::{Object, ToObject};
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};
use std::ptr;

use crate::block::BinaryOp;
use crate::field;
use crate::field::Field;
use crate::location;
use crate::location::Location;
use crate::lvalue;
use crate::lvalue::LValue;

/// An RValue is a value that may or may not have a storage address in gccjit.
/// RValues can be dereferenced, used for field accesses, and are the parameters
/// given to a majority of the gccjit API calls.
#[derive(Copy, Clone)]
pub struct RValue {
    ptr: *mut gccjit_sys::gcc_jit_rvalue,
}

/// ToRValue is a trait implemented by types that can be converted to, or
/// treated as, an RValue.
pub trait ToRValue {
    fn to_rvalue(&self) -> RValue;
}

impl ToObject for RValue {
    fn to_object(&self) -> Object {
        unsafe { object::from_ptr(gccjit_sys::gcc_jit_rvalue_as_object(self.ptr)) }
    }
}

impl fmt::Debug for RValue {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

impl ToRValue for RValue {
    fn to_rvalue(&self) -> RValue {
        unsafe { from_ptr(self.ptr) }
    }
}

macro_rules! binary_operator_for {
    ($ty:ty, $name:ident, $op:expr) => {
        impl $ty for RValue {
            type Output = RValue;

            fn $name(self, rhs: RValue) -> RValue {
                unsafe {
                    let rhs_rvalue = rhs.to_rvalue();
                    let obj_ptr = object::get_ptr(&self.to_object());
                    let ctx_ptr = gccjit_sys::gcc_jit_object_get_context(obj_ptr);
                    let ty = rhs.get_type();
                    let ptr = gccjit_sys::gcc_jit_context_new_binary_op(
                        ctx_ptr,
                        ptr::null_mut(),
                        mem::transmute($op),
                        types::get_ptr(&ty),
                        self.ptr,
                        rhs_rvalue.ptr,
                    );
                    from_ptr(ptr)
                }
            }
        }
    };
}

// Operator overloads for ease of manipulation of rvalues
binary_operator_for!(Add, add, BinaryOp::Plus);
binary_operator_for!(Sub, sub, BinaryOp::Minus);
binary_operator_for!(Mul, mul, BinaryOp::Mult);
binary_operator_for!(Div, div, BinaryOp::Divide);
binary_operator_for!(Rem, rem, BinaryOp::Modulo);
binary_operator_for!(BitAnd, bitand, BinaryOp::BitwiseAnd);
binary_operator_for!(BitOr, bitor, BinaryOp::BitwiseOr);
binary_operator_for!(BitXor, bitxor, BinaryOp::BitwiseXor);
binary_operator_for!(Shl<RValue>, shl, BinaryOp::LShift);
binary_operator_for!(Shr<RValue>, shr, BinaryOp::RShift);

impl RValue {
    /// Gets the type of this RValue.
    pub fn get_type(&self) -> Type {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_rvalue_get_type(self.ptr);
            types::from_ptr(ptr)
        }
    }

    /// Given an RValue x and a Field f, returns an RValue representing
    /// C's x.f.
    pub fn access_field(&self, loc: Option<Location>, field: Field) -> RValue {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr =
                gccjit_sys::gcc_jit_rvalue_access_field(self.ptr, loc_ptr, field::get_ptr(&field));
            rvalue::from_ptr(ptr)
        }
    }

    /// Given an RValue x and a Field f, returns an LValue representing
    /// C's x->f.
    pub fn dereference_field(
        &self,
        loc: Option<Location>,
        field: Field,
    ) -> LValue {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_rvalue_dereference_field(
                self.ptr,
                loc_ptr,
                field::get_ptr(&field),
            );
            lvalue::from_ptr(ptr)
        }
    }

    /// Given a RValue x, returns an RValue that represents *x.
    pub fn dereference(&self, loc: Option<Location>) -> LValue {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_rvalue_dereference(self.ptr, loc_ptr);

            lvalue::from_ptr(ptr)
        }
    }
}

pub unsafe fn from_ptr(ptr: *mut gccjit_sys::gcc_jit_rvalue) -> RValue {
    RValue {
        
        ptr: ptr,
    }
}

pub unsafe fn get_ptr(rvalue: &RValue) -> *mut gccjit_sys::gcc_jit_rvalue {
    rvalue.ptr
}
