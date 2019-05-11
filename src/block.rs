use crate::ctx::Context;
use crate::function::{self, Function};
use crate::location::{self, Location};
use crate::lvalue::{self, ToLValue};
use crate::object::{self, Object, ToObject};
use crate::rvalue::{self, ToRValue};
use gccjit_sys;
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use gccjit_sys::{gcc_jit_case, gcc_jit_case_as_object, gcc_jit_block_end_with_switch};

#[derive(Copy, Clone)]
pub struct Case/**/ {
    //marker: PhantomData<&'ctx Context>,
    ptr: *mut gcc_jit_case
}

impl Case {
    pub fn get_ptr(self) -> *mut gcc_jit_case {
        self.ptr
    }

    pub fn from_ptr(ptr: *mut gcc_jit_case) -> Case {
        Case {
            ptr
        }
    }
}


/// BinaryOp is a enum representing the various binary operations
/// that gccjit knows how to codegen.
#[repr(C)]
pub enum BinaryOp {
    Plus,
    Minus,
    Mult,
    Divide,
    Modulo,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
    LShift,
    RShift,
}

/// UnaryOp is an enum representing the various unary operations
/// that gccjit knows how to codegen.
#[repr(C)]
pub enum UnaryOp {
    Minus,
    BitwiseNegate,
    LogicalNegate,
    Abs,
}

/// ComparisonOp is an enum representing the various comparisons that
/// gccjit is capable of doing.
#[repr(C)]
pub enum ComparisonOp {
    Equals,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
}


impl ToObject for Case {
    fn to_object(&self) -> Object {
        unsafe {
            let ptr = gcc_jit_case_as_object(self.ptr);
            object::from_ptr(ptr)
        }
    }
}

/// Block represents a basic block in gccjit. Blocks are created by functions.
/// A basic block consists of a series of instructions terminated by a terminator
/// instruction, which can be either a jump to one block, a conditional branch to
/// two blocks (true/false branches), a return, or a void return.
#[derive(Copy, Clone)]
pub struct Block {

    pub(crate) ptr: *mut gccjit_sys::gcc_jit_block,
}

impl ToObject for Block {
    fn to_object(&self) -> Object {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_block_as_object(self.ptr);
            object::from_ptr(ptr)
        }
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

impl Block {
    pub fn get_function(&self) -> Function {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_block_get_function(self.ptr);
            function::from_ptr(ptr)
        }
    }

    /// Evaluates the rvalue parameter and discards its result. Equivalent
    /// to (void)<expr> in C.
    pub fn add_eval<T: ToRValue>(&self, loc: Option<Location>, value: T) {
        let rvalue = value.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            gccjit_sys::gcc_jit_block_add_eval(self.ptr, loc_ptr, rvalue::get_ptr(&rvalue));
        }
    }

    /// Assigns the value of an rvalue to an lvalue directly. Equivalent
    /// to <lvalue> = <rvalue> in C.
    pub fn add_assignment<L: ToLValue, R: ToRValue>(
        &self,
        loc: Option<Location>,
        assign_target: L,
        value: R,
    ) {
        let lvalue = assign_target.to_lvalue();
        let rvalue = value.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            gccjit_sys::gcc_jit_block_add_assignment(
                self.ptr,
                loc_ptr,
                lvalue::get_ptr(&lvalue),
                rvalue::get_ptr(&rvalue),
            );
        }
    }

    /// Performs a binary operation on an LValue and an RValue, assigning
    /// the result of the binary operation to the LValue upon completion.
    /// Equivalent to the *=, +=, -=, etc. operator family in C.
    pub fn add_assignment_op<L: ToLValue, R: ToRValue>(
        &self,
        loc: Option<Location>,
        assign_target: L,
        op: BinaryOp,
        value: R,
    ) {
        let lvalue = assign_target.to_lvalue();
        let rvalue = value.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            gccjit_sys::gcc_jit_block_add_assignment_op(
                self.ptr,
                loc_ptr,
                lvalue::get_ptr(&lvalue),
                mem::transmute(op),
                rvalue::get_ptr(&rvalue),
            );
        }
    }

    /// Adds a comment to a block. It's unclear from the documentation what
    /// this actually means.
    pub fn add_comment<S: AsRef<str>>(&self, loc: Option<Location>, message: S) {
        let message_ref = message.as_ref();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let cstr = CString::new(message_ref).unwrap();
            gccjit_sys::gcc_jit_block_add_comment(self.ptr, loc_ptr, cstr.as_ptr());
        }
    }

    /// Terminates a block by branching to one of two blocks, depending
    /// on the value of a conditional RValue.
    pub fn end_with_conditional<T: ToRValue>(
        &self,
        loc: Option<Location>,
        cond: T,
        on_true: Block,
        on_false: Block,
    ) {
        let cond_rvalue = cond.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            gccjit_sys::gcc_jit_block_end_with_conditional(
                self.ptr,
                loc_ptr,
                rvalue::get_ptr(&cond_rvalue),
                on_true.ptr,
                on_false.ptr,
            );
        }
    }

    /// Terminates a block by unconditionally jumping to another block.
    pub fn end_with_jump(&self, loc: Option<Location>, target: Block) {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            gccjit_sys::gcc_jit_block_end_with_jump(self.ptr, loc_ptr, target.ptr);
        }
    }

    pub fn end_with_switch(&self,loc: Option<Location>,expr: impl ToRValue,default_block: Block,cases: Vec<Case>) {
        unsafe {
            let mut cases_ = cases.iter().map(|elem| elem.get_ptr()).collect::<Vec<_>>();
            gcc_jit_block_end_with_switch(
                self.ptr,
                location::get_ptr(&loc.unwrap_or(location::from_ptr(ptr::null_mut()))),
                rvalue::get_ptr(&expr.to_rvalue()),
                default_block.ptr,
                cases_.len() as _,
                cases_.as_mut_ptr()
            );
        }
    }
    /// Terminates a block by returning from the containing function, setting
    /// the rvalue to be the return value of the function. This is equivalent
    /// to C's "return <expr>". This function can only be used to terminate
    /// a block within a function whose return type is not void.
    pub fn end_with_return<T: ToRValue>(&self, loc: Option<Location>, ret: T) {
        let ret_rvalue = ret.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            gccjit_sys::gcc_jit_block_end_with_return(
                self.ptr,
                loc_ptr,
                rvalue::get_ptr(&ret_rvalue),
            );
        }
    }

    /// Terminates a block by returning from the containing function, returning
    /// no value. This is equivalent to C's bare "return" with no expression.
    /// This function can only be used to terminate a block within a function
    /// that returns void.
    pub fn end_with_void_return(&self, loc: Option<Location>) {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            gccjit_sys::gcc_jit_block_end_with_void_return(self.ptr, loc_ptr);
        }
    }


}

pub unsafe fn from_ptr(ptr: *mut gccjit_sys::gcc_jit_block) -> Block {
    Block {
        
        ptr: ptr,
    }
}
