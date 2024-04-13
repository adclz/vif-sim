#![allow(non_snake_case)]

use serde_json::{Value};
use crate::kernel::plc::operations::basics::calc::Calc;
use crate::kernel::plc::operations::basics::compare::Compare;
use crate::kernel::plc::operations::basics::assign::Assign;
use crate::kernel::plc::operations::basics::call::Call;
use crate::kernel::plc::operations::program_control::r#for::For;
use crate::kernel::plc::operations::program_control::r#while::While;
use crate::kernel::plc::operations::program_control::r#if::If;
use crate::{error, key_reader};
use crate::parser::body::path::parse_path;
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::internal::template_impl::TemplateImpl;
use crate::kernel::plc::operations::internal::counter_sm::CounterStateMachine;
use crate::kernel::plc::operations::program_control::r#return::Return;
use crate::kernel::plc::operations::internal::timer_sm::TimerStateMachine;
use crate::kernel::plc::operations::internal::f_trig::F_Trig;
use crate::kernel::plc::operations::operations::{NewJsonOperation, JsonOperation};
use crate::kernel::plc::operations::unit::block::UnitBlock;
use crate::kernel::plc::operations::unit::log::UnitLog;
use crate::kernel::plc::operations::unit::breakpoint::BreakpointJson;
use crate::kernel::plc::operations::unit::test::UnitTestJson;
use crate::container::error::error::{Stop};
use crate::kernel::plc::operations::binary::rotate_left::RotateLeft;
use crate::kernel::plc::operations::binary::rotate_right::RotateRight;
use crate::kernel::plc::operations::binary::shl::Shl;
use crate::kernel::plc::operations::binary::shr::Shr;
use crate::kernel::plc::operations::binary::swap::Swap;
use crate::kernel::plc::operations::internal::r_trig::R_Trig;
use crate::kernel::plc::operations::internal::reset::Reset;
use crate::kernel::plc::operations::math::acos::ACos;
use crate::kernel::plc::operations::math::asin::ASin;
use crate::kernel::plc::operations::math::atan::ATan;
use crate::kernel::plc::operations::math::ceil::Ceil;
use crate::kernel::plc::operations::math::cos::Cos;
use crate::kernel::plc::operations::math::exp::Exp;
use crate::kernel::plc::operations::math::floor::Floor;
use crate::kernel::plc::operations::math::fract::Fract;
use crate::kernel::plc::operations::math::ln::Ln;
use crate::kernel::plc::operations::math::sin::Sin;
use crate::kernel::plc::operations::math::sqr::Sqr;
use crate::kernel::plc::operations::math::sqrt::Sqrt;
use crate::kernel::plc::operations::math::abs::Abs;
use crate::kernel::plc::operations::math::round::Round;
use crate::kernel::plc::operations::math::tan::Tan;
use crate::kernel::plc::operations::math::trunc::Trunc;


pub fn parse_json_target(json: &Value) -> Result<JsonTarget, Stop> {
    let as_object = json.as_object()
        .ok_or_else(move || error!(format!("Data for operation is not of type object: {}", json), "Parse Abstract".to_string()))?;

    key_reader!(
            format!("Parse ast"),
            as_object {
                ty => as_str,
                src => as_object,
            }
        );

    match ty {
        // Ref / Values
        "global" => Ok(JsonTarget::Global(parse_path(&src["path"]).map_err(|e| e.add_sim_trace("Parse global reference"))?)),
        "local" => Ok(JsonTarget::Local(parse_path(&src["path"]).map_err(|e| e.add_sim_trace("Parse local reference"))?)),
        "local_out" => Ok(JsonTarget::LocalOut(parse_path(&src["path"]).map_err(|e| e.add_sim_trace("Parse local_out reference"))?)),
        "#inner" => Ok(JsonTarget::Inner(parse_path(&src["path"]).map_err(|e| e.add_sim_trace("Parse inner reference"))?)),
        // Slice access
        "access" => Ok(JsonTarget::Access(src.clone())),

        // Unit
        "unit_test" => Ok(JsonTarget::Operation(Box::new(JsonOperation::UnitTestJson(UnitTestJson::new(src)?)))),
        "unit_log" => Ok(JsonTarget::Operation(Box::new(JsonOperation::UnitLog(UnitLog::new(src)?)))),
        "unit_block" => Ok(JsonTarget::Operation(Box::new(JsonOperation::UnitBlock(UnitBlock::new(src)?)))),
        "breakpoint" => Ok(JsonTarget::Operation(Box::new(JsonOperation::BreakpointJson(BreakpointJson::new(src)?)))),

        // Return
        "return" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Return(Return::new(src)?)))),

        // Operations
        "calc" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Calc(Calc::new(src)?)))),
        "compare" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Compare(Compare::new(src)?)))),
        "if" => Ok(JsonTarget::Operation(Box::new(JsonOperation::If(If::new(src)?)))),
        "for" => Ok(JsonTarget::Operation(Box::new(JsonOperation::For(For::new(src)?)))),
        "while" => Ok(JsonTarget::Operation(Box::new(JsonOperation::While(While::new(src)?)))),
        "asg" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Assign(Assign::new(src)?)))),
        "call" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Call(Call::new(src)?)))),
        "resolve_template" => Ok(JsonTarget::Operation(Box::new(JsonOperation::TemplateImpl(TemplateImpl::new(src)?)))),

        // Math
        "cos" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Cos(Cos::new(&src)?)))),
        "sin" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Sin(Sin::new(&src)?)))),
        "tan" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Tan(Tan::new(&src)?)))),
        "acos" => Ok(JsonTarget::Operation(Box::new(JsonOperation::ACos(ACos::new(&src)?)))),
        "asin" => Ok(JsonTarget::Operation(Box::new(JsonOperation::ASin(ASin::new(&src)?)))),
        "atan" => Ok(JsonTarget::Operation(Box::new(JsonOperation::ATan(ATan::new(&src)?)))),
        "exp" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Exp(Exp::new(&src)?)))),
        "ln" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Ln(Ln::new(&src)?)))),
        "fract" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Fract(Fract::new(&src)?)))),
        "trunc" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Trunc(Trunc::new(&src)?)))),
        "sqrt" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Sqrt(Sqrt::new(&src)?)))),
        "sqr" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Sqr(Sqr::new(&src)?)))),
        "abs" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Abs(Abs::new(&src)?)))),
        "ceil" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Ceil(Ceil::new(&src)?)))),
        "floor" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Floor(Floor::new(&src)?)))),
        "round" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Round(Round::new(&src)?)))),

        // Binaries
        "shl" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Shl(Shl::new(&src)?)))),
        "shr" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Shr(Shr::new(&src)?)))),
        "rol" => Ok(JsonTarget::Operation(Box::new(JsonOperation::RotateLeft(RotateLeft::new(&src)?)))),
        "ror" => Ok(JsonTarget::Operation(Box::new(JsonOperation::RotateRight(RotateRight::new(&src)?)))),
        "swap" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Swap(Swap::new(&src)?)))),

        // Internal
        "#timer_sm" => Ok(JsonTarget::Operation(Box::new(JsonOperation::TimerStateMachine(TimerStateMachine::new(src)?)))),
        "#counter_sm" => Ok(JsonTarget::Operation(Box::new(JsonOperation::CounterStateMachine(CounterStateMachine::new(src)?)))),
        "#f_trig" => Ok(JsonTarget::Operation(Box::new(JsonOperation::F_Trig(F_Trig::new(src)?)))),
        "#r_trig" => Ok(JsonTarget::Operation(Box::new(JsonOperation::R_Trig(R_Trig::new(src)?)))),
        "#reset" => Ok(JsonTarget::Operation(Box::new(JsonOperation::Reset(Reset::new(src)?)))),

        _ => Ok(JsonTarget::Constant(as_object.clone()))
    }.map_err(|e: Stop| e.add_sim_trace(&"Parse body type".to_string()))
}