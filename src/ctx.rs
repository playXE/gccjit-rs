use std::default::Default;
use std::ffi::CString;
use std::marker::PhantomData;
use std::mem;
use std::ops::Drop;
use std::ptr;

use crate::block::{BinaryOp, ComparisonOp, UnaryOp, Block, Case};
use crate::field::{self, Field};
use crate::function::{self, Function, FunctionType};
use crate::location::{self, Location};
use crate::lvalue::{self, LValue};
use crate::parameter::{self, Parameter};
use crate::rvalue::{self, RValue, ToRValue};
use crate::structs::{self, Struct};
use crate::ty as types;
use gccjit_sys::*;

use crate::sys::*;

/// Represents an optimization level that the JIT compiler
/// will use when compiling your code.
#[repr(C)]
pub enum OptimizationLevel {
    /// No optimizations are applied.
    None,
    /// Optimizies for both speed and code size, but doesn't apply
    /// any optimizations that take extended periods of time.
    Limited,
    /// Performs all optimizations that do not involve a tradeoff
    /// of code size for speed.
    Standard,
    /// Performs all optimizations at the Standard level, as well
    /// as function inlining, loop vectorization, some loop unrolling,
    /// and various other optimizations.
    Aggressive,
}

/// This enum indicates to gccjit the format of the output
/// code that is written out by compile_to_file.
#[repr(C)]
pub enum OutputKind {
    /// Outputs an assembly file (.S)
    Assembler,
    /// Outputs an object file (.o)
    ObjectFile,
    /// Outputs a dynamic library (.so)
    DynamicLibrary,
    /// Outputs an executable
    Executable,
}

pub struct Context<'a> {
    marker: PhantomData<&'a Context<'a>>,
    ptr: *mut crate::sys::gcc_jit_context,
}

impl Default for Context<'static> {
    fn default() -> Context<'static> {
        unsafe {
            Context {
                marker: PhantomData,
                ptr: crate::sys::gcc_jit_context_acquire(),
            }
        }
    }
}

impl<'a> Context<'a> {
    pub fn add_command_line_option(&self,name: impl AsRef<str>) {
        let name_ref = name.as_ref();
        let cstr = CString::new(name_ref).unwrap();
        unsafe {
            gcc_jit_context_add_command_line_option(self.ptr,cstr.as_ptr())
        }
    }
    pub fn add_driver_option(&self,name: impl AsRef<str>) {
        let name_ref = name.as_ref();
        let cstr = CString::new(name_ref).unwrap();
        unsafe {
            gcc_jit_context_add_driver_option(self.ptr, cstr.as_ptr());
        }
    }
    pub fn set_name(&self, name: impl AsRef<str>) {
        let name_ref = name.as_ref();
        let cstr = CString::new(name_ref).unwrap();
        unsafe {
            crate::sys::gcc_jit_context_set_str_option(
                self.ptr,
                gcc_jit_str_option_GCC_JIT_STR_OPTION_PROGNAME,
                cstr.as_ptr(),
            )
        }
    }

    /// Compiles the context and returns a CompileResult that contains
    /// the means to access functions and globals that have currently
    /// been JIT compiled.
    pub fn compile(&self) -> CompileResult {
        unsafe {
            CompileResult {
                ptr: gccjit_sys::gcc_jit_context_compile(self.ptr)
            }
        }
    }

    /// Compiles the context and saves the result to a file. The
    /// type of the file is controlled by the OutputKind parameter.
    pub fn compile_to_file<S: AsRef<str>>(&self, kind: OutputKind, file: S) {
        unsafe {
            let file_ref = file.as_ref();
            let cstr = CString::new(file_ref).unwrap();
            gccjit_sys::gcc_jit_context_compile_to_file(self.ptr,
                                                        mem::transmute(kind),
                                                        cstr.as_ptr());
        }
    }

    pub fn set_opt_level(&self, opt: OptimizationLevel) {
        unsafe {
            gcc_jit_context_set_int_option(
                self.ptr,
                gcc_jit_int_option_GCC_JIT_INT_OPTION_OPTIMIZATION_LEVEL,
                opt as i32,
            );
        }
    }

    pub fn set_dump_gimple(&self,value: bool) {
        unsafe {
            gcc_jit_context_set_bool_option(self.ptr,gcc_jit_bool_option_GCC_JIT_BOOL_OPTION_DUMP_INITIAL_GIMPLE,value as _);
        }
    }

    pub fn set_dump_code(&self, value: bool) {
        unsafe {
            gcc_jit_context_set_bool_option(
                self.ptr,
                gcc_jit_bool_option_GCC_JIT_BOOL_OPTION_DUMP_GENERATED_CODE,
                value as _,
            );
        }
    }

    /// Creates a new child context from this context. The child context
    /// is a fully-featured context, but it has a lifetime that is strictly
    /// less than the lifetime that spawned it.
    pub fn new_child_context<'b>(&'b self) -> Context<'b> {
        unsafe {
            Context {
                marker: PhantomData,
                ptr: gccjit_sys::gcc_jit_context_new_child_context(self.ptr),
            }
        }
    }

    /// Creates a new location for use by gdb when debugging a JIT compiled
    /// program. The filename, line, and col are used by gdb to "show" your
    /// source when in a debugger.
    pub fn new_location<'b, S: AsRef<str>>(
        &'b self,
        filename: S,
        line: i32,
        col: i32,
    ) -> Location<'b> {
        unsafe {
            let filename_ref = filename.as_ref();
            let cstr = CString::new(filename_ref).unwrap();
            let ptr = gccjit_sys::gcc_jit_context_new_location(self.ptr, cstr.as_ptr(), line, col);
            location::from_ptr(ptr)
        }
    }

    /// Constructs a new type for any type that implements the Typeable trait.
    /// This library only provides a handful of implementations of Typeable
    /// for some primitive types - utilizers of this library are encouraged
    /// to provide their own types that implement Typeable for ease of type
    /// creation.
    pub fn new_type<'b, T: types::Typeable>(&'b self) -> types::Type<'b> {
        <T as types::Typeable>::get_type(self)
    }

    pub fn new_vector_type<'b,T: types::Typeable>(&'b self,units: usize) -> types::Type<'b> {
        let ty = unsafe {gcc_jit_type_get_vector(types::get_ptr(&<T as types::Typeable>::get_type(self)),units)};
        unsafe {types::from_ptr(ty)}
    }

    /// Constructs a new array type with a given base element type and a
    /// size.
    pub fn new_array_type<'b>(
        &'b self,
        loc: Option<Location<'b>>,
        ty: types::Type<'b>,
        num_elements: i32,
    ) -> types::Type<'b> {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_array_type(
                self.ptr,
                loc_ptr,
                types::get_ptr(&ty),
                num_elements,
            );
            types::from_ptr(ptr)
        }
    }

    /// Constructs a new struct type with the given name, optional source location,
    /// and a list of fields. The returned struct is concrete and new fields cannot
    /// be added to it.
    pub fn new_struct_type<'b, S: AsRef<str>>(
        &'b self,
        loc: Option<Location<'b>>,
        name: S,
        fields: &[Field<'b>],
    ) -> Struct<'b> {
        let name_ref = name.as_ref();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        let num_fields = fields.len() as i32;
        let mut fields_ptrs: Vec<_> = fields
            .iter()
            .map(|x| unsafe { field::get_ptr(&x) })
            .collect();
        unsafe {
            let cname = CString::new(name_ref).unwrap();
            let ptr = gccjit_sys::gcc_jit_context_new_struct_type(
                self.ptr,
                loc_ptr,
                cname.as_ptr(),
                num_fields,
                fields_ptrs.as_mut_ptr(),
            );
            structs::from_ptr(ptr)
        }
    }

    /// Constructs a new struct type whose fields are not known. Fields can
    /// be added to this struct later, but only once.
    pub fn new_opaque_struct_type<'b, S: AsRef<str>>(
        &'b self,
        loc: Option<Location<'b>>,
        name: S,
    ) -> Struct<'b> {
        let name_ref = name.as_ref();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let cstr = CString::new(name_ref).unwrap();
            let ptr =
                gccjit_sys::gcc_jit_context_new_opaque_struct(self.ptr, loc_ptr, cstr.as_ptr());
            structs::from_ptr(ptr)
        }
    }

    /// Creates a new union type from a set of fields.
    pub fn new_union_type<'b, S: AsRef<str>>(
        &'b self,
        loc: Option<Location<'b>>,
        name: S,
        fields: &[Field<'b>],
    ) -> types::Type<'b> {
        let name_ref = name.as_ref();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        let num_fields = fields.len() as i32;
        let mut fields_ptrs: Vec<_> = fields
            .iter()
            .map(|x| unsafe { field::get_ptr(&x) })
            .collect();
        unsafe {
            let cname = CString::new(name_ref).unwrap();
            let ptr = gccjit_sys::gcc_jit_context_new_union_type(
                self.ptr,
                loc_ptr,
                cname.as_ptr(),
                num_fields,
                fields_ptrs.as_mut_ptr(),
            );
            types::from_ptr(ptr)
        }
    }

    pub fn new_case<'b>(&self,min_value: impl ToRValue<'b>,max_value: impl ToRValue<'b>,dest_block: Block<'b>) -> Case<'a> {
        unsafe {
            Case::from_ptr(gcc_jit_context_new_case(self.ptr,rvalue::get_ptr(&min_value.to_rvalue()),rvalue::get_ptr(&max_value.to_rvalue()),dest_block.ptr))
        }
    }

    pub fn new_field<'b>(&self,loc: Option<Location<'b>>,ty: types::Type<'b>,name: impl AsRef<str>) -> Field<'b> {
        unsafe {
            field::from_ptr(
                gcc_jit_context_new_field(
                    self.ptr,
                    location::get_ptr(&loc.unwrap_or(location::from_ptr(ptr::null_mut()))),
                    types::get_ptr(&ty),
                    CString::new(name.as_ref()).unwrap().as_ptr()
                )
            )
        }
    }

    /// Creates a new function pointer type with the given return type
    /// parameter types, and an optional location. The last flag can
    /// make the function variadic, although Rust can't really handle
    /// the varargs calling convention.
    pub fn new_function_pointer_type<'b>(
        &'a self,
        loc: Option<Location<'b>>,
        return_type: types::Type<'b>,
        param_types: &[types::Type<'b>],
        is_variadic: bool,
    ) -> types::Type<'b> {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        let num_types = param_types.len() as i32;
        let mut types_ptrs: Vec<_> = param_types
            .iter()
            .map(|x| unsafe { types::get_ptr(&x) })
            .collect();
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_function_ptr_type(
                self.ptr,
                loc_ptr,
                types::get_ptr(&return_type),
                num_types,
                types_ptrs.as_mut_ptr(),
                is_variadic as i32,
            );
            types::from_ptr(ptr)
        }
    }

    /// Creates a new function with the given function kind, return type, parameters, name,
    /// and whether or not the function is variadic.
    pub fn new_function<'b, S: AsRef<str>>(
        &'b self,
        loc: Option<Location<'b>>,
        kind: FunctionType,
        return_ty: types::Type<'b>,
        params: &[Parameter<'b>],
        name: S,
        is_variadic: bool,
    ) -> Function<'a> {
        let name_ref = name.as_ref();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        let num_params = params.len() as i32;
        let mut params_ptrs: Vec<_> = params
            .iter()
            .map(|x| unsafe { parameter::get_ptr(&x) })
            .collect();
        unsafe {
            let cstr = CString::new(name_ref).unwrap();
            let ptr = gccjit_sys::gcc_jit_context_new_function(
                self.ptr,
                loc_ptr,
                mem::transmute(kind),
                types::get_ptr(&return_ty),
                cstr.as_ptr(),
                num_params,
                params_ptrs.as_mut_ptr(),
                is_variadic as i32,
            );
            function::from_ptr(ptr)
        }
    }

    /// Creates a new binary operation between two RValues and produces a new RValue.
    pub fn new_binary_op<'b, L: ToRValue<'a>, R: ToRValue<'b>>(
        &'b self,
        loc: Option<Location<'b>>,
        op: BinaryOp,
        ty: types::Type<'b>,
        left: L,
        right: R,
    ) -> RValue<'b> {
        let left_rvalue = left.to_rvalue();
        let right_rvalue = right.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_binary_op(
                self.ptr,
                loc_ptr,
                mem::transmute(op),
                types::get_ptr(&ty),
                rvalue::get_ptr(&left_rvalue),
                rvalue::get_ptr(&right_rvalue),
            );
            rvalue::from_ptr(ptr)
        }
    }

    /// Creates a unary operation on one RValue and produces a result RValue.
    pub fn new_unary_op<'b, T: ToRValue<'b>>(
        &'b self,
        loc: Option<Location<'b>>,
        op: UnaryOp,
        ty: types::Type<'b>,
        target: T,
    ) -> RValue<'b> {
        let rvalue = target.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_unary_op(
                self.ptr,
                loc_ptr,
                mem::transmute(op),
                types::get_ptr(&ty),
                rvalue::get_ptr(&rvalue),
            );
            rvalue::from_ptr(ptr)
        }
    }

    pub fn new_comparison<'b, L: ToRValue<'b>, R: ToRValue<'b>>(
        &'b self,
        loc: Option<Location<'b>>,
        op: ComparisonOp,
        left: L,
        right: R,
    ) -> RValue<'b> {
        let left_rvalue = left.to_rvalue();
        let right_rvalue = right.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_comparison(
                self.ptr,
                loc_ptr,
                mem::transmute(op),
                rvalue::get_ptr(&left_rvalue),
                rvalue::get_ptr(&right_rvalue),
            );
            rvalue::from_ptr(ptr)
        }
    }

    /// Creates a function call to a function object with a given number of parameters.
    /// The RValue that is returned is the result of the function call.
    /// Note that due to the way that Rust's generics work, it is currently
    /// not possible to be generic over different types of arguments (RValues
    /// together with LValues and Parameters, for example), so in order to
    /// mix the types of the arguments it may be necessary to call to_rvalue()
    /// before calling this function.
    pub fn new_call<'b>(
        &'b self,
        loc: Option<Location<'b>>,
        func: Function<'b>,
        args: &[RValue<'b>],
    ) -> RValue<'b> {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        let num_params = args.len() as i32;
        let mut params_ptrs: Vec<_> = args
            .iter()
            .map(|x| unsafe { rvalue::get_ptr(&x) })
            .collect();
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_call(
                self.ptr,
                loc_ptr,
                function::get_ptr(&func),
                num_params,
                params_ptrs.as_mut_ptr(),
            );
            rvalue::from_ptr(ptr)
        }
    }

    pub fn new_rvalue_from_vector<'b>(
        &'b self,
        loc: Option<Location<'b>>,
        elements: Vec<RValue<'b>>,
        ty: types::Type<'b>,
    ) -> RValue<'b> {
        let mut elems = vec![];
        for elem in elements.iter() {
            unsafe { elems.push(rvalue::get_ptr(elem)) };
        }
        let elems_ptr = elems.as_mut_ptr();
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_rvalue_from_vector(
                self.ptr,
                 location::get_ptr(&loc.unwrap_or(location::from_ptr(ptr::null_mut()))),
                types::get_ptr(&ty),
                elems.len() as _,
                elems_ptr,
            );

            rvalue::from_ptr(ptr)
        }
    }

    /// Creates an indirect function call that dereferences a function pointer and
    /// attempts to invoke it with the given arguments. The RValue that is returned
    /// is the result of the function call.
    pub fn new_call_through_ptr<'b, F: ToRValue<'a>>(
        &'b self,
        loc: Option<Location<'b>>,
        fun_ptr: F,
        args: &[RValue<'b>],
    ) -> RValue<'b> {
        let fun_ptr_rvalue = fun_ptr.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        let num_params = args.len() as i32;
        let mut params_ptrs: Vec<_> = args
            .iter()
            .map(|x| x.to_rvalue())
            .map(|x| unsafe { rvalue::get_ptr(&x) })
            .collect();
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_call_through_ptr(
                self.ptr,
                loc_ptr,
                rvalue::get_ptr(&fun_ptr_rvalue),
                num_params,
                params_ptrs.as_mut_ptr(),
            );
            rvalue::from_ptr(ptr)
        }
    }

    /// Cast an RValue to a specific type. I don't know what happens when the cast fails yet.
    pub fn new_cast<'b, T: ToRValue<'a>>(
        &'b self,
        loc: Option<Location<'b>>,
        value: T,
        dest_type: types::Type<'b>,
    ) -> RValue<'b> {
        let rvalue = value.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_cast(
                self.ptr,
                loc_ptr,
                rvalue::get_ptr(&rvalue),
                types::get_ptr(&dest_type),
            );
            rvalue::from_ptr(ptr)
        }
    }

    /// Creates an LValue from an array pointer and an offset. The LValue can be the target
    /// of an assignment, or it can be converted into an RValue (i.e. loaded).
    pub fn new_array_access<'b, A: ToRValue<'b>, I: ToRValue<'b>>(
        &'b self,
        loc: Option<Location<'b>>,
        array_ptr: A,
        index: I,
    ) -> LValue<'b> {
        let array_rvalue = array_ptr.to_rvalue();
        let idx_rvalue = index.to_rvalue();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_array_access(
                self.ptr,
                loc_ptr,
                rvalue::get_ptr(&array_rvalue),
                rvalue::get_ptr(&idx_rvalue),
            );
            lvalue::from_ptr(ptr)
        }
    }

    /// Creates a new RValue from a given long value.
    pub fn new_rvalue_from_long<'b>(&'b self, ty: types::Type<'b>, value: i64) -> RValue<'b> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_rvalue_from_long(
                self.ptr,
                types::get_ptr(&ty),
                value,
            );
            rvalue::from_ptr(ptr)
        }
    }

    /// Creates a new RValue from a given int value.
    pub fn new_rvalue_from_int<'b>(&'b self, ty: types::Type<'b>, value: i32) -> RValue<'b> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_rvalue_from_int(
                self.ptr,
                types::get_ptr(&ty),
                value,
            );
            rvalue::from_ptr(ptr)
        }
    }

    /// Creates a new RValue from a given double value.
    pub fn new_rvalue_from_double<'b>(&'b self, ty: types::Type<'b>, value: f64) -> RValue<'b> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_rvalue_from_double(
                self.ptr,
                types::get_ptr(&ty),
                value,
            );
            rvalue::from_ptr(ptr)
        }
    }

    /// Creates a zero element for a given type.
    pub fn new_rvalue_zero<'b>(&'b self, ty: types::Type<'b>) -> RValue<'b> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_zero(self.ptr, types::get_ptr(&ty));
            rvalue::from_ptr(ptr)
        }
    }

    /// Creates a one element for a given type.
    pub fn new_rvalue_one<'b>(&'b self, ty: types::Type<'b>) -> RValue<'b> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_one(self.ptr, types::get_ptr(&ty));
            rvalue::from_ptr(ptr)
        }
    }

    /// Creates an RValue for a raw pointer. This function
    /// requires that the lifetime of the pointer be greater
    /// than that of the jitted program.
    pub fn new_rvalue_from_ptr<'b>(&'b self, ty: types::Type<'b>, value: *mut ()) -> RValue<'b> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_new_rvalue_from_ptr(
                self.ptr,
                types::get_ptr(&ty),
                mem::transmute(value),
            );
            rvalue::from_ptr(ptr)
        }
    }

    /// Creates a null RValue.
    pub fn new_null<'b>(&'b self, ty: types::Type<'b>) -> RValue<'b> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_context_null(self.ptr, types::get_ptr(&ty));
            rvalue::from_ptr(ptr)
        }
    }

    /// Creates a string literal RValue.
    pub fn new_string_literal<'b, S: AsRef<str>>(&'b self, value: S) -> RValue<'b> {
        unsafe {
            let cstr = CString::new(value.as_ref()).unwrap();
            let ptr = gccjit_sys::gcc_jit_context_new_string_literal(self.ptr, cstr.as_ptr());
            rvalue::from_ptr(ptr)
        }
    }

    /// Dumps a small C file to the path that can be used to reproduce a series
    /// of API calls. You should only ever need to call this if you are debugging
    /// an issue in gccjit itself or this library.
    pub fn dump_reproducer_to_file<S: AsRef<str>>(&self, path: S) {
        unsafe {
            let path_ref = path.as_ref();
            let cstr = CString::new(path_ref).unwrap();
            gccjit_sys::gcc_jit_context_dump_reproducer_to_file(self.ptr, cstr.as_ptr());
        }
    }

    /// Creates a new parameter with a given type, name, and location.
    pub fn new_parameter<'b, S: AsRef<str>>(
        &'b self,
        loc: Option<Location<'b>>,
        ty: types::Type<'b>,
        name: S,
    ) -> Parameter<'b> {
        let name_ref = name.as_ref();
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut(),
        };
        unsafe {
            let cstr = CString::new(name_ref).unwrap();
            let ptr = gccjit_sys::gcc_jit_context_new_param(
                self.ptr,
                loc_ptr,
                types::get_ptr(&ty),
                cstr.as_ptr(),
            );
            parameter::from_ptr(ptr)
        }
    }

    /// Get a builtin function from gcc. It's not clear what functions are
    /// builtin and you'll likely need to consult the GCC internal docs
    /// for a full list.
    pub fn get_builtin_function<'b, S: AsRef<str>>(&'a self, name: S) -> Function<'b> {
        let name_ref = name.as_ref();
        unsafe {
            let cstr = CString::new(name_ref).unwrap();
            let ptr = gccjit_sys::gcc_jit_context_get_builtin_function(self.ptr, cstr.as_ptr());
            function::from_ptr(ptr)
        }
    }
}

pub fn context_get_ptr<'a>(ctx: &'a Context<'a>) -> *mut gcc_jit_context {
    ctx.ptr
}

pub struct CompileResult {
    ptr: *mut gccjit_sys::gcc_jit_result
}

impl CompileResult {
    /// Gets a function pointer to a JIT compiled function. If the function
    /// does not exist (wasn't compiled by the Context that produced this
    /// CompileResult), this function returns a null pointer.
    ///
    /// It is the caller's responsibility to ensure that this pointer is not used
    /// past the lifetime of the CompileResult object. Second, it is
    /// the caller's responsibility to check whether or not the pointer
    /// is null. It is also expected that the caller of this function
    /// will transmute this pointer to a function pointer type.
    pub fn get_function<S: AsRef<str>>(&self, name: S) -> *mut () {
        let c_str = CString::new(name.as_ref()).unwrap();
        unsafe {
            let func = gccjit_sys::gcc_jit_result_get_code(self.ptr,
                                                           c_str.as_ptr());
            mem::transmute(func)
        }
    }

    /// Gets a pointer to a global variable that lives on the JIT heap.
    ///
    /// It is the caller's responsibility
    /// to ensure that the pointer is not used past the lifetime of the
    /// CompileResult object. It is also the caller's responsibility to
    /// check whether or not the returned pointer is null.
    pub fn get_global<S: AsRef<str>>(&self, name: S) -> *mut () {
        let c_str = CString::new(name.as_ref()).unwrap();
        unsafe {
            let ptr = gccjit_sys::gcc_jit_result_get_global(self.ptr, c_str.as_ptr());
            mem::transmute(ptr)
        }
    }
}

impl Drop for CompileResult {
    fn drop(&mut self) {
        unsafe {
            gccjit_sys::gcc_jit_result_release(self.ptr);
        }
    }
}
