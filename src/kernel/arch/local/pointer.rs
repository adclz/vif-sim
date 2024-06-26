﻿use std::borrow::Cow;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::error;
use crate::kernel::plc::types::complex::array::PlcArray;
use crate::kernel::plc::types::complex::instance::fb_instance::FbInstance;
use crate::kernel::plc::types::complex::instance::public::PublicInstanceTrait;
use crate::kernel::plc::types::complex::r#struct::PlcStruct;
use crate::kernel::plc::interface::traits::InterfaceAccessors;
use crate::kernel::plc::types::primitives::binaries::plc_binary::PlcBinary;
use crate::kernel::plc::types::primitives::boolean::plc_bool::PlcBool;
use crate::kernel::plc::types::primitives::traits::family_traits::{GetRawPointerPrimitive, WithMutFamily, WithRefFamily, IsFamily, WithTypeFamily};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic, MetaData, SetMetaData};
use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::integers::plc_integer::PlcInteger;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{RawMut, ToggleMonitor};
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive};
use crate::kernel::plc::types::primitives::string::plc_string::PlcString;
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::timers::plc_time::PlcTime;
use crate::kernel::plc::types::primitives::tod::plc_tod::PlcTod;
use crate::kernel::arch::local::r#type::{IntoLocalType, LocalType};
use camelpaste::paste;
use fixedstr::str256;

use serde::{Serialize, Serializer};
use core::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use core::fmt::{Display, Formatter};
use core::ops::{Deref, DerefMut};
use std::rc::Rc;
use crate::kernel::plc::types::primitives::string::_string::plcstr;
use crate::kernel::plc::types::primitives::string::wstring::plcwstr;
use crate::kernel::registry::Kernel;

pub struct LocalPointer {
    inner: Rc<RefCell<LocalType>>,
    read_only: bool
}

pub struct LocalPointerAndPath(pub (LocalPointer, Vec<usize>));

impl Clone for LocalPointer {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            read_only: self.read_only
        }
    }
}

impl AsRef<Rc<RefCell<LocalType>>> for LocalPointer {
    fn as_ref(&self) -> &Rc<RefCell<LocalType>> {
        &self.inner
    }
}

impl From<LocalType> for LocalPointer {
    fn from(value: LocalType) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
            read_only: false
        }
    }
}

impl MetaData for LocalPointer {
    fn name(&self) -> &'static str {
        self.inner.borrow().name()
    }

    fn get_alias_str<'a>(&'a self, kernel: &'a Kernel) -> Option<&'a String> {
        match self.inner.borrow().get_alias_id(kernel) {
            Some(a) => kernel.get_type_alias_str(a),
            None => None
        }

    }

    fn get_alias_id(&self, kernel: &Kernel) -> Option<usize> {
        self.inner.borrow().get_alias_id(kernel)
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }

    fn get_path(&self) -> String {
        self.inner.borrow().get_path()
    }
}

impl SetMetaData for LocalPointer {
    fn set_alias(&mut self, alias: &str, kernel: &Kernel) {
        self.as_ref().borrow_mut().set_alias(alias, kernel)
    }
    fn set_read_only(&mut self, value: bool) {
        self.read_only = value
    }

    fn set_name(&mut self, path: usize) {
        self.as_ref().borrow_mut().set_name(path)
    }
}

pub enum Path {
    Pointer(LocalPointer),
    Path(HashMap<usize, Path>),
}

impl InterfaceAccessors for LocalPointer {
    fn try_get_nested(&self, path: &[usize]) -> Option<LocalPointer> {
        match self.as_ref().borrow().deref() {
            LocalType::PlcStruct(a) => a.try_get_nested(path),
            LocalType::FbInstance(a) => a.try_get_nested(path),
            LocalType::PlcArray(a) => a.try_get_nested(path),
            _ => None,
        }
    }
}

#[macro_export]
macro_rules! impl_primitive_l {
    ($type: ident { $($primitive: ident),+ }) => {
        paste! {
            impl Primitive for $type {
                $(
                    fn [<is_$primitive>](&self) -> bool { self.as_ref().borrow().deref().[<is_$primitive>]() }
                    fn [<as_$primitive>](&self, channel: &Broadcast) -> Result<$primitive, Stop> {
                        self.as_ref().borrow().deref().[<as_$primitive>](channel)
                    }
                )+
            }
            impl AsMutPrimitive for $type {
                $(
                    fn [<set_$primitive>](&mut self, other: $primitive, channel: &Broadcast) -> Result<(), Stop> {
                        if self.is_read_only() { return Err(error!(format!("Attempt to change a constant value"))) }
                        self.as_ref().borrow_mut().deref_mut().[<set_$primitive>](other, channel)
                    }

                    fn [<set_default_$primitive>](&mut self, other: $primitive) -> Result<(), Stop> {
                        if self.is_read_only() { return Err(error!(format!("Attempt to change a constant value"))) }
                        self.as_ref().borrow_mut().deref_mut().[<set_default_$primitive>](other)
                    }
                )+
            }
        }
    };
}

impl_primitive_l!(LocalPointer {
    bool,
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    f32,
    f64,
    plcstr,
    char,
    plcwstr,
    wchar
});

impl LocalPointer {
    pub fn new(a_type: LocalType) -> Self {
        Self {
            inner: Rc::new(RefCell::new(a_type)),
            read_only: false
        }
    }

    pub fn set_read_only(&mut self, value: bool) {
        self.read_only = value
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn replace_pointer(&mut self, other: &LocalPointer) {
        self.inner = Rc::clone(&other.inner);
        self.read_only = other.read_only;
    }

    pub fn duplicate(&self) -> LocalPointer {
       Self::new(self.inner.borrow().deref().clone())
    }
    pub(crate) fn set_monitor(&self, kernel: &Kernel) {
        self.inner.borrow_mut().set_monitor(kernel);
    }
}
impl IntoLocalType for LocalPointer {
    fn transform(&self) -> Result<LocalType, Stop> {
        self.inner.borrow().deref().transform()
    }
}
macro_rules! impl_local_pointer {
    ($($simple_family: ident),+ + $($complex_family: ident),+) => {
        paste! {
            impl IsFamily for LocalPointer {
                $(
                    fn [<is_$simple_family:snake>](&self) -> bool {
                        match self.inner.borrow().deref() {
                            LocalType::$simple_family(..) => true,
                            _ => false
                        }
                    }
                )+
                $(
                    fn [<is_$complex_family:snake>](&self) -> bool {
                        match self.inner.borrow().deref() {
                            LocalType::$complex_family(..) => true,
                            _ => false
                        }
                    }
                )+

                fn is_complex(&self) -> bool {
                    match self.inner.borrow().deref() {
                        $(LocalType::$complex_family(..) => true,)+
                        _ => false
                    }
                }
            }

            impl WithRefFamily for LocalPointer {
                $(
                    fn [<with_$simple_family:snake>]<R>(&self, _channel: &Broadcast, f: impl Fn(&$simple_family) -> R) -> Result<R, Stop> {
                        match self.inner.borrow().deref() {
                            LocalType::$simple_family(a) => Ok(f(&*a)),
                            _ => Err(error!(format!("Could not borrow LocalPointer, expected {}, got {}", stringify!($simple_family), self)))
                        }
                    }
                )+
                $(
                    fn [<with_$complex_family:snake>]<R>(&self, _channel: &Broadcast, f: impl Fn(&$complex_family) -> R) -> Result<R, Stop> {
                        match self.inner.borrow().deref() {
                            LocalType::$complex_family(a) => Ok(f(&*a)),
                            _ => Err(error!(format!("Could not borrow LocalPointer, expected {}, got {}", stringify!($complex_family), self)))
                        }
                    }
                )+
            }

            impl WithMutFamily for LocalPointer {
                $(
                    fn [<with_mut_$simple_family:snake>]<R>(&self, _channel: &Broadcast, f: &mut impl FnMut(&mut $simple_family) -> R) -> Result<R, Stop> {
                        if self.is_read_only() { return Err(error!(format!("Attempt to change a constant value"))) }
                        match self.inner.borrow_mut().deref_mut() {
                            LocalType::$simple_family(a) => Ok(f(&mut *a)),
                            _ => Err(error!(format!("Could not borrow LocalPointer, expected {}, got {}", stringify!($simple_family), self)))
                        }
                    }
                )+
                $(
                    fn [<with_mut_$complex_family:snake>]<R>(&self, _channel: &Broadcast, f: &mut impl FnMut(&mut $complex_family) -> R) -> Result<R, Stop> {
                        if self.is_read_only() { return Err(error!(format!("Attempt to change a constant value"))) }
                        match self.inner.borrow_mut().deref_mut() {
                            LocalType::$complex_family(a) => Ok(f(&mut *a)),
                            _ => Err(error!(format!("Could not borrow LocalPointer, expected {}, got {}", stringify!($complex_family), self)))
                        }
                    }
                )+
            }

            impl WithTypeFamily for LocalPointer {
                $(
                    fn [<with_type_$simple_family:snake>]<R>(&self, f: impl Fn(&$simple_family) -> R) -> Result<R, Stop> {
                        match self.inner.borrow().deref() {
                            LocalType::$simple_family(a) => Ok(f(&*a)),
                            _ => Err(error!(format!("Could not borrow LocalPointer, expected {}, got {}", stringify!($simple_family), self)))
                        }
                    }
                )+
                $(
                    fn [<with_type_$complex_family:snake>]<R>(&self, f: impl Fn(&$complex_family) -> R) -> Result<R, Stop> {
                        match self.inner.borrow().deref() {
                            LocalType::$complex_family(a) => Ok(f(&*a)),
                            _ => Err(error!(format!("Could not borrow LocalPointer, expected {}, got {}", stringify!($complex_family), self)))
                        }
                    }
                )+
            }

            impl Display for LocalPointer {
                fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                    match self.inner.borrow().deref() {
                        $(LocalType::$simple_family(local) => write!(f, "{}", local),)+
                        $(LocalType::$complex_family(local) => write!(f, "{}", local),)+
                    }
                }
            }

            impl Serialize for LocalPointer {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
                    match self.inner.borrow().deref() {
                        $(LocalType::$simple_family(a) => a.serialize(serializer),)+
                        $(LocalType::$complex_family(a) => a.serialize(serializer),)+
                    }
                }
            }

            impl LocalPointer {
                pub fn get_raw_pointers(&self) -> Vec<*mut dyn RawMut>  {
                    match self.inner.borrow_mut().deref_mut() {
                        $(LocalType::$simple_family(a) => vec!(a.get_raw_pointer())),+,
                        $(LocalType::$complex_family(a) => a.get_raw_pointers()),+,
                    }
                }

                pub fn get_pointers_with_path(&self, full_path: &[usize], start_with: &[usize]) -> Vec<LocalPointerAndPath>  {
                    match self.inner.borrow_mut().deref_mut() {
                        $(LocalType::$simple_family(a) => vec!(LocalPointerAndPath((self.clone(), full_path.to_vec().clone())))),+,
                        $(LocalType::$complex_family(a) => a.get_pointers_with_path(full_path, start_with)),+,
                    }
                }

                $(pub fn [<as_$simple_family:snake>](&self) -> Result<Ref<$simple_family>, Stop> {
                    Ref::filter_map(self.inner.borrow(), |a| {
                        match a {
                            LocalType::$simple_family(b) => Some(b),
                            _ => None
                        }
                    }).map(|e| e).map_err(|_e| error!(format!("Expected type {}, got {}", stringify!($simple_family), self)))
                })+
                $(pub fn [<as_$complex_family:snake>](&self) -> Result<Ref<$complex_family>, Stop> {
                    Ref::filter_map(self.inner.borrow(), |a| {
                        match a {
                            LocalType::$complex_family(b) => Some(b),
                            _ => None
                        }
                    }).map(|e| e).map_err(|_e| error!(format!("Expected type {}, got {}", stringify!($complex_family), self)))
                })+

                $(pub fn [<as_mut_$simple_family:snake>](&self) -> Result<RefMut<$simple_family>, Stop> {
                    RefMut::filter_map(self.inner.borrow_mut(), |a| {
                        match a {
                            LocalType::$simple_family(b) => Some(b),
                            _ => None
                        }
                    }).map(|e| e).map_err(|_e| error!(format!("Expected type {}, got {}", stringify!($simple_family), self)))
                })+
                $(pub fn [<as_mut_$complex_family:snake>](&self) -> Result<RefMut<$complex_family>, Stop> {
                    RefMut::filter_map(self.inner.borrow_mut(), |a| {
                        match a {
                            LocalType::$complex_family(b) => Some(b),
                            _ => None
                        }
                    }).map(|e| e).map_err(|_e| error!(format!("Expected type {}, got {}", stringify!($complex_family), self)))
                })+
            }
        }
    };
}

impl_local_pointer!(
    PlcBool,
    PlcInteger,
    PlcFloat,
    PlcBinary,
    PlcTime,
    PlcString,
    PlcTod +

    PlcStruct,
    PlcArray,
    FbInstance
);
