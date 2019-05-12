extern crate gccjit_rs;

use ctx::*;
use gccjit_rs::*;

use gccjit_rs::block::BinaryOp;
use gccjit_rs::function::FunctionType;
use gccjit_rs::rvalue::ToRValue;
use std::intrinsics::transmute;

fn main() {
    let ctx = Context::default();
    ctx.set_dump_code(true);
    ctx.set_opt_level(OptimizationLevel::Aggressive);
    ctx.set_dump_gimple(true);
    let int = ctx.new_type::<i32>();
    let ptr_ty = ctx.new_type::<char>().make_pointer();
    let param = ctx.new_parameter(None, int, "n");
    let add2 = ctx.new_function(None, FunctionType::Exported, int, &[param], "add2", false);
    let printf = ctx.new_function(
        None,
        FunctionType::Extern,
        ctx.new_type::<()>(),
        &[ctx.new_parameter(None, ptr_ty, "fmt")],
        "printf",
        true,
    );
    let block = add2.new_block("entry");

    let param = add2.new_local(None, int, "n");

    block.add_assignment(
        None,
        param,
        add2.get_param(0).to_rvalue() + ctx.new_rvalue_from_int(int, 4),
    );
    block.add_eval(
        None,
        ctx.new_call(
            None,
            printf,
            &[ctx.new_string_literal("Value %i\n"), param.to_rvalue()],
        ),
    );
    let result = param.to_rvalue() + ctx.new_rvalue_from_int(int, 2);
    block.end_with_return(None, result);

    let result = ctx.compile();
    let add2_fn: fn(i32) -> i32 = unsafe { transmute(result.get_function("add2")) };

    println!("{}", add2_fn(25));
}
