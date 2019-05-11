#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
pub use gccjit_sys as sys;
pub mod block;
pub mod ctx;
pub mod field;
pub mod function;
pub mod location;
pub mod lvalue;
pub mod object;
pub mod parameter;
pub mod rvalue;
pub mod structs;
pub mod ty;
