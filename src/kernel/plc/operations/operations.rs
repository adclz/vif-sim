use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::registry::Kernel;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use serde_json::{Map, Value};
use std::cell::{RefCell, RefMut};
use std::fmt::{Display, Formatter};
use std::future::{IntoFuture};

use crate::kernel::plc::operations::basics::calc::Calc;
use crate::kernel::plc::operations::basics::compare::Compare;
use crate::kernel::plc::operations::basics::assign::Assign;
use crate::kernel::plc::operations::basics::call::Call;
use crate::kernel::plc::operations::program_control::r#return::Return;
use crate::kernel::plc::operations::program_control::r#for::For;
use crate::kernel::plc::operations::program_control::r#while::While;
use crate::kernel::plc::operations::program_control::r#if::If;
use crate::kernel::plc::operations::math::cos::Cos;
use crate::kernel::plc::operations::math::sin::Sin;
use crate::kernel::plc::operations::math::tan::Tan;

use crate::kernel::plc::internal::template_impl::{TemplateImpl, TemplateMemory};
use crate::kernel::plc::operations::internal::counter_sm::CounterStateMachine;
use crate::kernel::plc::operations::internal::timer_sm::TimerStateMachine;
use crate::kernel::plc::operations::internal::f_trig::F_Trig;
use crate::kernel::plc::operations::internal::r_trig::R_Trig;

use crate::error;
use crate::kernel::plc::types::complex::array::PlcArray;
use crate::kernel::plc::types::complex::instance::fb_instance::FbInstance;
use crate::kernel::plc::types::complex::r#struct::PlcStruct;
use crate::kernel::plc::types::primitives::binaries::plc_binary::PlcBinary;
use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::boolean::plc_bool::PlcBool;
use crate::kernel::plc::types::primitives::traits::family_traits::{IsFamily, WithRefFamily, WithTypeFamily};
use crate::kernel::plc::types::primitives::integers::plc_integer::PlcInteger;
use crate::kernel::plc::types::primitives::string::plc_string::PlcString;
use crate::kernel::plc::types::primitives::timers::plc_time::PlcTime;
use crate::kernel::plc::types::primitives::tod::plc_tod::PlcTod;
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::kernel::arch::local::r#type::{LocalType, IntoLocalType};
use camelpaste::paste;
use std::ops::Deref;
use std::rc::{Rc};
use crate::kernel::plc::operations::unit::block::UnitBlock;
use crate::kernel::plc::operations::unit::log::UnitLog;
use crate::kernel::plc::operations::unit::breakpoint::BreakpointJson;
use crate::kernel::plc::operations::unit::test::UnitTestJson;
use fixedstr::str256;
use crate::parser::trace::trace::FileTrace;
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::string::wstring::wstr256;

use crate::kernel::plc::operations::binary::rotate_left::RotateLeft;
use crate::kernel::plc::operations::binary::rotate_right::RotateRight;
use crate::kernel::plc::operations::binary::shl::Shl;
use crate::kernel::plc::operations::binary::shr::Shr;
use crate::kernel::plc::operations::binary::swap::Swap;
use crate::kernel::plc::operations::internal::reset::Reset;
use crate::kernel::plc::operations::math::abs::Abs;
use crate::kernel::plc::operations::math::acos::ACos;
use crate::kernel::plc::operations::math::asin::ASin;
use crate::kernel::plc::operations::math::atan::ATan;
use crate::kernel::plc::operations::math::ceil::Ceil;
use crate::kernel::plc::operations::math::exp::Exp;
use crate::kernel::plc::operations::math::floor::Floor;
use crate::kernel::plc::operations::math::fract::Fract;
use crate::kernel::plc::operations::math::ln::Ln;
use crate::kernel::plc::operations::math::round::Round;
use crate::kernel::plc::operations::math::sqr::Sqr;
use crate::kernel::plc::operations::math::sqrt::Sqrt;
use crate::kernel::plc::operations::math::trunc::Trunc;
use crate::kernel::plc::types::primitives::traits::meta_data::{MaybeHeapOrStatic, MetaData, SetMetaData};
use crate::kernel::plc::types::primitives::traits::primitive_traits::Primitive;


#[enum_dispatch::enum_dispatch]
pub trait BuildJsonOperation {
    fn build(
        &self,
        interface: &SectionInterface,
        template_interface: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast,
    ) -> Result<RunTimeOperation, Stop>;
}

pub trait NewJsonOperation {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop>
        where
            Self: Clone;
}

#[derive(Clone)]
pub struct Operation {
    name: MaybeHeapOrStatic,
    return_early: bool,
    return_ptr: Option<LocalPointer>,
    trace: Option<FileTrace>,
    closure: Rc<RefCell<dyn FnMut(&Broadcast) -> Result<(), Stop>>>,
}

impl Operation {
    pub fn new(
        name: MaybeHeapOrStatic,
        closure: impl FnMut(&Broadcast) -> Result<(), Stop> + 'static,
        return_ptr: Option<LocalPointer>,
        return_early: bool,
        trace: &Option<FileTrace>
    ) -> Self {
        Self {
            name,
            return_early,
            return_ptr: return_ptr.clone(),
            closure: Rc::new(RefCell::new(closure)),
            trace: trace.clone()
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.return_ptr {
            None =>  write!(f, "{} -> Void", self.name),
            Some(a) =>  write!(f, "{} -> {}", self.name, a)
        }
    }
}

impl Operation {
    pub fn return_early(&self) -> bool {
        self.return_early
    }

    pub fn borrow_closure(&self) -> RefMut<dyn FnMut(&Broadcast) -> Result<(), Stop>> {
        RefMut::map(self.closure.borrow_mut(), |a| a)
    }
}

pub type RunTimeOperation = Box<Operation>;

impl MetaData for RunTimeOperation {
    fn name(&self) -> &'static str {
        &"Operation"
    }

    fn get_alias_str<'a>(&'a self, kernel: &'a Kernel) -> Option<&'a String> {
        match &self.return_ptr {
            None => None,
            Some(a) => a.get_alias_str(kernel)
        }
    }

    fn get_alias_id(&self, kernel: &Kernel) -> Option<usize> {
        match &self.return_ptr {
            None => None,
            Some(a) => a.get_alias_id(kernel)
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn get_path(&self) -> String {
        "".into()
    }
}

impl SetMetaData for RunTimeOperation {
    fn set_alias(&mut self, alias: &str, kernel: &Kernel) {
        // do nothing
    }

    fn set_read_only(&mut self, value: bool) {
        // do nothing
    }

    fn set_name(&mut self, path: usize) {
        // do nothing
    }
}

pub trait RuntimeOperationTrait {
    fn with_void(&self, channel: &Broadcast) -> Result<(), Stop>;
    fn get_return_pointer(&self) -> Option<LocalPointer>;
    fn is_void(&self) -> bool;
}

impl RuntimeOperationTrait for RunTimeOperation {
    fn with_void(&self, channel: &Broadcast) -> Result<(), Stop> {
        self.borrow_closure()(channel)
            .map_err(|e|
                match &self.name.0 {
                    None => e,
                    Some(a) => e.add_sim_trace(&format!("{}", self.name))
                }.maybe_file_trace(&self.trace)
            )?;
        match &self.return_ptr {
            Some(a) => Err(error!(format!(
                "Operation was expected to be void, but {} was returned",
                a
            ))),
            None => Ok(()),
        }
    }

    fn get_return_pointer(&self) -> Option<LocalPointer> {
        self.return_ptr.as_ref().map(|a| a.clone())
    }

    fn is_void(&self) -> bool {
        self.return_ptr.is_some()
    }
}
macro_rules! create_json_operations {
    ($($operation: ident),+) => {
        #[enum_dispatch::enum_dispatch(BuildJsonOperation)]
        #[derive(Clone)]
        pub enum JsonOperation {
            $($operation($operation)),+
        }
    };
}

create_json_operations!(
    // Unit
    UnitTestJson,
    UnitLog,
    UnitBlock,
    BreakpointJson,
    TimerStateMachine,
    CounterStateMachine,
    TemplateImpl,
    // Internal
    F_Trig,
    R_Trig,
    Reset,
    // Return
    Return,
    // Operations
    Calc,
    Compare,
    If,
    For,
    While,
    Assign,
    Call,
    // Math
    Cos,
    Sin,
    Tan,
    ACos,
    ASin,
    ATan,
    Exp,
    Ln,
    Fract,
    Trunc,
    Sqrt,
    Sqr,
    Abs,
    Ceil,
    Floor,
    Round,
    // Binaries
    Shl,
    Shr,
    RotateLeft,
    RotateRight,
    Swap
);

macro_rules! impl_family {
    ($($simple_family: ident),+ + $($complex_family: ident),+) => {
        paste! {
            impl IsFamily for RunTimeOperation {
                $(
                    fn [<is_$simple_family:snake>](&self) -> bool {
                        match &self.return_ptr {
                            Some(ptr) => ptr.[<is_$simple_family:snake>](),
                            None => false
                        }
                    }
                )+
                $(
                    fn [<is_$complex_family:snake>](&self) -> bool {
                        match &self.return_ptr {
                            Some(ptr) => ptr.[<is_$complex_family:snake>](),
                            None => false
                        }
                    }
                )+

                fn is_complex(&self) -> bool {
                    match &self.return_ptr {
                        Some(ptr) => match ptr.as_ref().borrow().deref() {
                            $(LocalType::$complex_family(_) => true,)+
                            _ => false
                        },
                        None => false
                    }
                }
            }

            impl WithRefFamily for RunTimeOperation {
                $(
                    fn [<with_$simple_family:snake>]<R>(&self, channel: &Broadcast, f: impl Fn(&[<$simple_family>]) -> R) -> Result<R, Stop> {
                        match &self.return_ptr {
                            None => Err(error!(format!("Operation return type '{}' expected, got void instead", stringify!($simple_family)))),
                            Some(ptr) => {
                                self.borrow_closure()(channel)
            .map_err(|e|
                match &self.name.0 {
                    None => e,
                    Some(a) => e.add_sim_trace(&format!("{}", self.name))
                }.maybe_file_trace(&self.trace)
            )?;
                                ptr.[<with_$simple_family:snake>](channel, |a| {
                                    f(&a)
                                })
                            }
                        }
                    }
                )+
                $(
                    fn [<with_$complex_family:snake>]<R>(&self, channel: &Broadcast, f: impl Fn(&[<$complex_family>]) -> R) -> Result<R, Stop> {
                        match &self.return_ptr {
                            None => Err(error!(format!("Operation return type '{}' expected, got void instead", stringify!($complex_family)))),
                            Some(ptr) => {
                                self.borrow_closure()(channel)
            .map_err(|e|
                match &self.name.0 {
                    None => e,
                    Some(a) => e.add_sim_trace(&format!("{}", self.name))
                }.maybe_file_trace(&self.trace)
            )?;
                                ptr.[<with_$complex_family:snake>](channel, |a| {
                                    f(&a)
                                })
                            }
                        }
                    }
                )+
            }

            impl WithTypeFamily for RunTimeOperation {
                $(
                    fn [<with_type_$simple_family:snake>]<R>(&self, f: impl Fn(&[<$simple_family>]) -> R) -> Result<R, Stop> {
                        match &self.return_ptr {
                            None => Err(error!(format!("Operation return type '{}' expected, got void instead", stringify!($simple_family)))),
                            Some(ptr) => {
                                ptr.[<with_type_$simple_family:snake>](|a| {
                                    f(&a)
                                })
                            }
                        }
                    }
                )+
                $(
                    fn [<with_type_$complex_family:snake>]<R>(&self, f: impl Fn(&[<$complex_family>]) -> R) -> Result<R, Stop> {
                        match &self.return_ptr {
                             None => Err(error!(format!("Operation return type '{}' expected, got void instead", stringify!($complex_family)))),
                             Some(ptr) => {
                                ptr.[<with_type_$complex_family:snake>](|a| {
                                    f(&a)
                                })
                            }
                        }
                    }
                )+
            }

            impl IntoLocalType for RunTimeOperation {
                fn transform(&self) -> Result<LocalType, Stop> {
                    match &self.return_ptr {
                        None => Err(error!(format!("Operation return type expected, got void instead"))),
                        Some(ptr) => match ptr.as_ref().borrow().deref() {
                                $(LocalType::$simple_family(a) => Ok(LocalType::$simple_family(a.clone()))),+,
                                $(LocalType::$complex_family(a) => Ok(LocalType::$complex_family(a.clone()))),+
                            }
                        }
                    }
            }
        }
    };
}

impl_family!(
    PlcBool,
    PlcInteger,
    PlcFloat,
    PlcBinary,
    PlcString,
    PlcTime,
    PlcTod
    +
    PlcStruct,
    PlcArray,
    FbInstance
);

macro_rules! impl_primitive_stmt {
    ($type: ident { $($primitive: ident),+ }) => {
        paste! {
            impl Primitive for $type {
                $(
                    fn [<is_$primitive>](&self) -> bool {
                        match &self.return_ptr {
                            None => false,
                            Some(a) => a.[<is_$primitive>]()
                        }
                    }
                    fn [<as_$primitive>](&self, channel: &Broadcast) -> Result<$primitive, Stop> {
                        match &self.return_ptr {
                            None => Err(error!(format!("Return type of operation is not {}, got void", stringify!($primitive)))),
                            Some(a) => {
                                self.borrow_closure()(channel).map_err(|e| e.maybe_file_trace(&self.trace))?;
                                a.[<as_$primitive>](channel)
                            }
                        }
                    }
                )+
            }
        }
    };
}

impl_primitive_stmt!(
    RunTimeOperation {
        bool,
        u8, i8,
        u16, i16,
        u32, i32,
        u64, i64,
        f32, f64,
        str256, char,
        wstr256, wchar
    }
);
