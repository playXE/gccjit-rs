use crate::block;
use crate::block::Block;
use crate::ctx::Context;
use crate::location;
use crate::location::Location;
use crate::lvalue;
use crate::lvalue::LValue;
use crate::object;
use crate::object::{Object, ToObject};
use crate::parameter;
use crate::parameter::Parameter;
use crate::ty as types;
use crate::ty::Type;
use gccjit_sys;
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use std::ptr;

/// FunctionType informs gccjit what sort of function a new function will be.
/// An exported function is a function that will be exported using the CompileResult
/// interface, able to be called outside of the jit. An internal function is
/// a function that cannot be called outside of jitted code. An extern function
/// is a function with external linkage, and always inline is a function that is
/// always inlined wherever it is called and cannot be accessed outside of the jit.
#[repr(C)]
pub enum FunctionType {
    /// Defines a function that is "exported" by the JIT and can be called from
    /// Rust.
    Exported = 0,
    /// Defines a function that is internal to the JIT and cannot be called
    /// from Rust, but can be called from jitted code.
    Internal = 1,
    /// Defines a function with external linkage.
    Extern = 2,
    /// Defines a function that should always be inlined whenever it is called.
    /// A function with this type cannot be called from Rust. If the optimization
    /// level is None, this function will not actually be inlined, but it still
    /// can only be called from within jitted code.
    AlwaysInline = 3,
}

/// Function is gccjit's representation of a function. Functions are constructed
/// by constructing basic blocks and connecting them together. Locals are declared
/// at the function level.
#[derive(Copy, Clone)]
pub struct Function {
    ptr: *mut gccjit_sys::gcc_jit_function,
}

impl ToObject for Function {
    fn to_object(&self) -> Object {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_function_as_object(self.ptr);
            object::from_ptr(ptr)
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

impl Function {
    pub fn get_param(&self, idx: i32) -> Parameter {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_function_get_param(self.ptr, idx);
            parameter::from_ptr(ptr)
        }
    }

    pub fn get_address(&self, loc: Option<Location>) -> crate::rvalue::RValue {
        unsafe {
            crate::rvalue::from_ptr(gccjit_sys::gcc_jit_function_get_address(
                self.ptr,
                location::get_ptr(&loc.unwrap_or(location::from_ptr(ptr::null_mut()))),
            ))
        }
    }

    pub fn dump_to_dot<S: AsRef<str>>(&self, path: S) {
        unsafe {
            let cstr = CString::new(path.as_ref()).unwrap();
            gccjit_sys::gcc_jit_function_dump_to_dot(self.ptr, cstr.as_ptr());
        }
    }

    pub fn new_block<S: AsRef<str>>(&self, name: S) -> Block {
        unsafe {
            let cstr = CString::new(name.as_ref()).unwrap();
            let ptr = gccjit_sys::gcc_jit_function_new_block(self.ptr, cstr.as_ptr());
            block::from_ptr(ptr)
        }
    }

    pub fn new_local<S: AsRef<str>>(&self, loc: Option<Location>, ty: Type, name: S) -> LValue {
        unsafe {
            let loc_ptr = match loc {
                Some(loc) => location::get_ptr(&loc),
                None => ptr::null_mut(),
            };
            let cstr = CString::new(name.as_ref()).unwrap();
            let ptr = gccjit_sys::gcc_jit_function_new_local(
                self.ptr,
                loc_ptr,
                types::get_ptr(&ty),
                cstr.as_ptr(),
            );
            lvalue::from_ptr(ptr)
        }
    }
}

pub unsafe fn from_ptr(ptr: *mut gccjit_sys::gcc_jit_function) -> Function {
    Function { ptr: ptr }
}

pub unsafe fn get_ptr(loc: &Function) -> *mut gccjit_sys::gcc_jit_function {
    loc.ptr
}
