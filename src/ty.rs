
use crate::ctx::*;
use crate::sys::*;
use std::fmt;


#[derive(Copy, Clone)]
pub struct Type {
    ptr: *mut gcc_jit_type,
}

impl Type {
    /// Given a type T, creates a type to *T, a pointer to T.
    pub fn make_pointer(self) -> Type {
        unsafe { from_ptr(gccjit_sys::gcc_jit_type_get_pointer(self.ptr)) }
    }

    /// Given a type T, creates a type of const T.
    pub fn make_const(self) -> Type {
        unsafe { from_ptr(gccjit_sys::gcc_jit_type_get_const(self.ptr)) }
    }

    pub fn from_const(ctx: &Context, u: u32) -> Type {
        unsafe { from_ptr(gcc_jit_context_get_type(context_get_ptr(ctx), u)) }
    }

    /// Given a type T, creates a new type of volatile T, which
    /// has the semantics of C's volatile.
    pub fn make_volatile(self) -> Type {
        unsafe { from_ptr(gccjit_sys::gcc_jit_type_get_volatile(self.ptr)) }
    }
}

pub trait Typeable {
    fn get_type(_: &Context) -> Type;
}

macro_rules! typeable_def {
    ($ty:ty,$expr: expr) => {
        impl Typeable for $ty {
            fn get_type(ctx: &Context) -> Type {
                unsafe {
                    let ptr = context_get_ptr(ctx);

                    let ptr = gcc_jit_context_get_type(ptr, $expr);
                    from_ptr(ptr)
                }
            }
        }
    };
    () => {};
}

use crate::object;
use crate::object::{Object, ToObject};
impl ToObject for Type {
    fn to_object(&self) -> Object {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_type_as_object(self.ptr);
            object::from_ptr(ptr)
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

typeable_def!((), gcc_jit_types_GCC_JIT_TYPE_VOID);
typeable_def!(bool, gcc_jit_types_GCC_JIT_TYPE_BOOL);
typeable_def!(char, gcc_jit_types_GCC_JIT_TYPE_CHAR);
typeable_def!(i8, gcc_jit_types_GCC_JIT_TYPE_SIGNED_CHAR);
typeable_def!(u8, gcc_jit_types_GCC_JIT_TYPE_UNSIGNED_CHAR);
typeable_def!(i16, gcc_jit_types_GCC_JIT_TYPE_SHORT);
typeable_def!(u16, gcc_jit_types_GCC_JIT_TYPE_UNSIGNED_SHORT);
typeable_def!(i32, gcc_jit_types_GCC_JIT_TYPE_INT);
typeable_def!(u32, gcc_jit_types_GCC_JIT_TYPE_UNSIGNED_INT);
typeable_def!(i64, gcc_jit_types_GCC_JIT_TYPE_LONG);
typeable_def!(u64, gcc_jit_types_GCC_JIT_TYPE_UNSIGNED_LONG);
typeable_def!(f32, gcc_jit_types_GCC_JIT_TYPE_FLOAT);
typeable_def!(f64, gcc_jit_types_GCC_JIT_TYPE_DOUBLE);
typeable_def!(usize, gcc_jit_types_GCC_JIT_TYPE_SIZE_T);

impl<T: Typeable> Typeable for *mut T {
    fn get_type(ctx: &Context) -> Type {
        unsafe {
            let ptr = gcc_jit_type_get_pointer(get_ptr(&T::get_type(ctx)));
            from_ptr(ptr)
        }
    }
}

impl<T: Typeable> Typeable for *const T {
    fn get_type(ctx: &Context) -> Type {
        unsafe {
           
            let ptr = gcc_jit_type_get_pointer(get_ptr(&T::get_type(ctx)));
            from_ptr(ptr).make_const()
        }
    }
}

pub unsafe fn from_ptr(ptr: *mut gccjit_sys::gcc_jit_type) -> Type {
    Type { ptr: ptr }
}

pub unsafe fn get_ptr(ty: &Type) -> *mut gccjit_sys::gcc_jit_type {
    ty.ptr
}
