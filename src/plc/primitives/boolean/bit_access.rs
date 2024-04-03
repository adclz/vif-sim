use std::any::TypeId;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::ops::DerefMut;
use std::rc::Rc;
use camelpaste::paste;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use crate::parser::body::body::parse_json_target;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::internal::template_impl::TemplateMemory;
use crate::plc::operations::operations::{NewJsonOperation};
use crate::plc::primitives::family_traits::{AsMutPrimitive, Primitive, ToggleMonitor, MetaData, SetMetaData};
use crate::plc::primitives::primitive_traits::{PrimitiveTrait, RawMut};
use crate::registry::local::pointer::LocalPointer;
use crate::registry::registry::Kernel;

#[derive(Clone)]
pub struct BitAccess {
    get_closure: Rc<RefCell<dyn Fn(&Broadcast) -> Result<bool, Stop>>>,
    set_closure: Rc<RefCell<dyn FnMut(&Broadcast) -> Result<(), Stop>>>,
    trace: Option<FileTrace>,
    of: LocalPointer,
    at: u64,
    monitor: bool
}

impl FileTraceBuilder for BitAccess {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl BitAccess {
    pub fn new_(
        json: &Map<String, Value>,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Bit Access"),
            json {
                of,
                at => as_u64,
                trace? => as_object,
            }
        );

        let trace = match trace {
            None => None,
            Some(a) => Self::build_trace(a),
        };

        let of = parse_json_target(of).map_err(|e| {
            e.add_sim_trace(&format!("Build Assign Operation [assign]"))
                .maybe_file_trace(&trace)
        })?;

        let of = of
            .solve_as_local_pointer(interface, template, registry, channel)
            .ok_or(error!(format!("Expected a valid number reference, got {}", of), "Build bit access -> source".to_string()).maybe_file_trace(&trace))?;

        Ok(Self{
            get_closure: Rc::new(RefCell::new(box_bit_get(&of, at)?)),
            set_closure: Rc::new(RefCell::new(box_bit_set(&of, at)?)),
            trace,
            of,
            at,
            monitor: false
        })
    }
}



impl RawMut for BitAccess {
    fn reset_ptr(&mut self, channel: &Broadcast) {
        panic!("A bit access cannot be referenced with a raw pointer")
    }
}

impl Display for BitAccess {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bit [{}] of {}", self.at, self.of)
    }
}

impl Serialize for BitAccess {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str("Bit access")
    }
}

impl MetaData for BitAccess {
    fn name(&self) -> &'static str {
        &"Bool"
    }
    fn get_alias_str<'a>(&'a self, kernel: &'a Kernel) -> Option<&'a String> {
        None
    }

    fn get_alias_id(&self, kernel: &Kernel) -> Option<usize> {
        None
    }

    fn is_read_only(&self) -> bool {
        true
    }
}

impl SetMetaData for BitAccess {
    fn set_alias(&mut self, alias: &str, kernel: &Kernel) {
        // do nothing
    }

    fn set_read_only(&mut self, value: bool) {
        // do nothing
    }
}

impl ToggleMonitor for BitAccess {
    fn set_monitor(&mut self, activate: bool) {
        self.monitor = false
    }
}

impl PrimitiveTrait for BitAccess {
    type Native = bool;
    type PlcPrimitive = BitAccess;

    fn new(value: &Self::Native) -> Result<Self::PlcPrimitive, Stop> {
        Err(error!(format!("A bit access can't be created manually, this should not happen")))
    }

    fn get(&self, channel: &Broadcast) -> Result<Self::Native, Stop> {
        self.get_closure.borrow_mut().deref_mut()(channel)
    }

    fn set(&mut self, value: Self::Native, channel: &Broadcast) -> Result<(), Stop> {
        self.set_closure.borrow_mut().deref_mut()(channel)
    }

    fn set_default(&mut self, value: Self::Native) -> Result<(), Stop> {
        panic!("A bit access can't have a default value, this should not happen")
    }

    fn reset(&mut self, channel: &Broadcast) {
        panic!("A bit access can't be reset, this should not happen")
    }

    fn get_id(&self) -> usize {
        panic!("A bit access does not have an id, this should not happen")
    }

    fn get_type_id(&self) -> TypeId {
        panic!("A bit access does not have an type id, this should not happen")
    }

    fn monitor(&self, channel: &Broadcast) {
        panic!("A bit access can't be monitored, this should not happen")
    }
}

macro_rules! box_bit_access_numbers {
    ($($ty: ty),+) => {
        paste! {
            pub fn box_bit_get<T: 'static + Primitive + Clone + Display>(variable: &T, index: u64) -> Result<Box<dyn Fn(&Broadcast) -> Result<bool, Stop>>, Stop> {
                $(
                    if variable.[<is_$ty>]() {
                        let variable_clone = variable.clone();

                        return Ok(Box::new(move |channel| {
                            if index < $ty::MAX as u64 {
                                Ok(variable_clone.[<as_$ty>](channel)? & (1 << index) != 0)
                            } else {
                                Err(error!(format!("Index out of bounds")))
                            }
                        }))
                    }
                )+
                Err(error!(format!("Invalid operation: Can not get bit of {}", variable)))
            }

            pub fn box_bit_set<T: 'static + Primitive + AsMutPrimitive + Clone + Display>(variable: &T, index: u64) -> Result<Box<dyn FnMut(&Broadcast) -> Result<(), Stop>>, Stop> {
                $(
                    if variable.[<is_$ty>]() {
                        let mut variable_clone = variable.clone();

                        return Ok(Box::new(move |channel| {
                            if index < $ty::MAX as u64 {
                                let other = variable_clone.[<as_$ty>](channel)?;
                                variable_clone.[<set_$ty>](other & !(1 << index) | ($ty::from(true) << index), channel)
                            } else {
                                Err(error!(format!("Index out of bounds")))
                            }
                        }))
                    }
                )+
                Err(error!(format!("Invalid operation: Can not get bit of {}", variable)))
            }
        }
    };
}

box_bit_access_numbers!(u8, u16, u32, u64, i8, i16, i32, i64);
