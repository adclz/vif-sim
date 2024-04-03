#![allow(non_snake_case)]

use crate::registry::global::r#type::GlobalType;
use crate::container::error::error::Stop;
use crate::plc::pou::udt::Udt;
use crate::plc::pou::ob::Ob;
use crate::plc::pou::fb::Fb;
use crate::plc::pou::fc::Fc;
use crate::plc::pou::db::db::Db;
use crate::error;
use camelpaste::paste;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Display, Formatter};
use std::ops::{Deref};
use std::rc::Rc;
use crate::plc::interface::traits::InterfaceAccessors;
use crate::registry::local::pointer::LocalPointer;

pub struct GlobalPointer(Rc<RefCell<GlobalType>>);

impl Clone for GlobalPointer {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl AsRef<Rc<RefCell<GlobalType>>> for GlobalPointer {
    fn as_ref(&self) -> &Rc<RefCell<GlobalType>> {
        &self.0
    }
}

impl InterfaceAccessors for GlobalPointer {
    fn try_get_nested(&self, path: &[String]) -> Option<LocalPointer> {
       match self.as_ref().borrow().deref() {
           GlobalType::Db(a) => match a {
               Db::Global(b) => b.try_get_nested(path),
               Db::Instance(b) => b.try_get_nested(path),
           },
           _ => None
       }
    }
}

impl GlobalPointer {
    pub fn new(a_type: GlobalType) -> Self {
        GlobalPointer(Rc::new(RefCell::new(a_type)))
    }
}

impl Display for GlobalPointer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.as_ref().borrow().deref() {
            GlobalType::Ob(_) => write!(f, "Ob"),
            GlobalType::Fb(_) => write!(f, "Fb"),
            GlobalType::Fc(_) => write!(f, "Fc"),
            GlobalType::Db(db) => {
                match db {
                    Db::Global(_) => write!(f, "Global Db"),
                    Db::Instance(_) => write!(f, "Instance Db"),
                }
            }
            GlobalType::Udt(_) => write!(f, "Udt"),
        }
    }
}

macro_rules! impl_global_pointer {
    ($($global_type:ident),+) => {
        paste! {
            impl GlobalPointer {
                $(
                    pub fn [<is_$global_type:snake>](&self) -> bool {
                        match self.as_ref().borrow().deref() {
                            GlobalType::$global_type(_) => true,
                            _ => false
                        }
                    }

                    pub fn [<as_ref_$global_type:snake>](&self) -> Result<Ref<$global_type>, Stop> {
                        Ref::filter_map(self.as_ref().borrow(), |e| match e {
                            GlobalType::$global_type(ref $global_type) => Some($global_type),
                            _ => None,
                        })
                        .map_or(Err(error!(format!("Could not deref {} as {}", stringify!($global_type), stringify!($global_type)))), |f| Ok(f))
                    }

                    pub fn [<as_mut_$global_type:snake>](&self) -> Result<RefMut<$global_type>, Stop> {
                        RefMut::filter_map(self.as_ref().borrow_mut(), |e| match e {
                            GlobalType::$global_type($global_type) => Some($global_type),
                            _ => None,
                        })
                        .map_or(Err(error!(format!("Could not deref {} as {}", stringify!($global_type), stringify!($global_type)))), |f| Ok(f))
                    }
                )+
            }
        }
    };
}

impl_global_pointer!(
    Udt,
    Db,
    Fc,
    Fb,
    Ob
);