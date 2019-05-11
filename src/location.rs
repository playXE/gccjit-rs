use crate::ctx::Context;
use crate::object;
use gccjit_sys;
use object::{Object, ToObject};
use std::fmt;
use std::marker::PhantomData;

/// A Location represents a location used when debugging jitted code.
#[derive(Copy, Clone)]
pub struct Location<'ctx> {
    marker: PhantomData<&'ctx Context<'ctx>>,
    ptr: *mut gccjit_sys::gcc_jit_location,
}

impl<'ctx> ToObject<'ctx> for Location<'ctx> {
    fn to_object(&self) -> Object<'ctx> {
        unsafe { object::from_ptr(gccjit_sys::gcc_jit_location_as_object(self.ptr)) }
    }
}

impl<'ctx> fmt::Debug for Location<'ctx> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

pub unsafe fn from_ptr<'ctx>(ptr: *mut gccjit_sys::gcc_jit_location) -> Location<'ctx> {
    Location {
        marker: PhantomData,
        ptr: ptr,
    }
}

pub unsafe fn get_ptr<'ctx>(loc: &Location<'ctx>) -> *mut gccjit_sys::gcc_jit_location {
    loc.ptr
}
