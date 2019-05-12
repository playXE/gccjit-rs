use crate::ctx::Context;
use crate::object;
use gccjit_sys;
use object::{Object, ToObject};
use std::fmt;
use std::marker::PhantomData;

/// A Location represents a location used when debugging jitted code.
#[derive(Copy, Clone)]
pub struct Location {
    ptr: *mut gccjit_sys::gcc_jit_location,
}

impl ToObject for Location {
    fn to_object(&self) -> Object {
        unsafe { object::from_ptr(gccjit_sys::gcc_jit_location_as_object(self.ptr)) }
    }
}

impl fmt::Debug for Location {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

pub unsafe fn from_ptr(ptr: *mut gccjit_sys::gcc_jit_location) -> Location {
    Location { ptr: ptr }
}

pub unsafe fn get_ptr(loc: &Location) -> *mut gccjit_sys::gcc_jit_location {
    loc.ptr
}
