use gccjit_rs::*;

use ctx::*;
use function::*;
use block::*;
use std::intrinsics::transmute;

fn main() {
    let ctx = Context::default();
    ctx.set_dump_code(true);
    ctx.set_opt_level(OptimizationLevel::Standart);

    let char_ptr = ctx.new_type::<char>().make_pointer(); // char*
    let int = ctx.new_type::<i32>(); // int
    let argv_ty = char_ptr.make_pointer(); // char**
    let printf = ctx.new_function(
        None,
        FunctionType::Extern,
        ctx.new_type::<()>(),
        &[ctx.new_parameter(None,char_ptr,"fmt")],
        "printf",
        true
    );


    let main = ctx.new_function(
        None,
        FunctionType::Exported,
        int,
        &[ctx.new_parameter(None,int,"argc"),ctx.new_parameter(None,argv_ty,"argv")],
        "main",
        false
    );

    let string = ctx.new_string_literal("Hello,world!\n");

    let block = main.new_block("entry");
    block.add_eval(
        None,
        ctx.new_call(
            None,
            printf,
            &[string]
        )
    );

    block.end_with_return(None,ctx.new_rvalue_from_int(int,0));

    let result = ctx.compile();

    let main_fn: fn() -> i32 = unsafe {transmute(result.get_function("main"))};

    main_fn();

}