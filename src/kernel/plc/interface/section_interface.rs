use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use serde::{Serialize, Serializer};
use serde::ser::{SerializeMap};
use crate::kernel::plc::types::complex::instance::public::PublicInstanceTrait;
use crate::kernel::plc::interface::section::Section;
use crate::kernel::plc::interface::struct_interface::StructInterface;
use crate::kernel::plc::types::primitives::traits::family_traits::{IsFamily};
use crate::kernel::plc::types::primitives::traits::primitive_traits::ToggleMonitor;
use crate::kernel::plc::types::primitives::traits::primitive_traits::RawMut;
use crate::kernel::arch::local::pointer::{LocalPointer, LocalPointerAndPath};
use crate::kernel::arch::local::r#type::LocalType;
use crate::kernel::registry::{get_string, Kernel};

pub struct SectionInterface((HashMap<Section, StructInterface>, Option<LocalPointer>));

impl Clone for SectionInterface {
    fn clone(&self) -> Self {
        Self((self.0.0
                  .iter()
                  .map(|(section, field)| {
                      (*section, field.as_ref()
                          .iter()
                          .map(|(field, pointer)| {
                              (
                                  field.clone(),
                                  LocalPointer::new(pointer.as_ref().borrow().deref().clone()),
                              )
                          })
                          .collect())
                  })
                  .collect(), self.0.1.as_ref()
                  .map(|pointer| LocalPointer::new(pointer.as_ref().borrow().deref().clone()))))
    }
}

impl From<(HashMap<Section, StructInterface>, Option<LocalPointer>)> for SectionInterface {
    fn from(value: (HashMap<Section, StructInterface>, Option<LocalPointer>)) -> Self {
        Self(value)
    }
}

impl Display for SectionInterface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.0.is_empty() && self.0.1.is_none() {
            writeln!(f, "Empty Interface")
        } else {
            self.0.0.iter().try_for_each(|(section, interface)| {
                writeln!(f, "{}: {}", section, interface.len())?;
                write!(f, "{}", interface)
            })?;
            match &self.0.1 {
                Some(p) => writeln!(f, "Return -> {}", p),
                None => writeln!(f, "Return -> Void")
            }
        }
    }
}

impl Serialize for SectionInterface {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut map = serializer.serialize_map(None)?;
        self.iter()
            .try_for_each(|a| {
                a.1.iter().try_for_each(|b| {
                    map.serialize_entry(&b.0, &b.1)
                })?;
                Ok(())
            })?;
        map.end()
    }
}

impl ToggleMonitor for SectionInterface {
    fn set_monitor(&self, kernel: &Kernel) {
        self
            .iter()
            .for_each(|x| x.1.set_monitor(kernel))
    }
}

impl SectionInterface {
    pub fn get_raw_pointers(&self) -> Vec<*mut dyn RawMut> {
        self.iter()
            .fold(vec![], |_all, p| p.1.get_raw_pointers())
    }

    pub fn get_pointers_with_path(&self, full_path: &[usize], start_with: &[usize]) -> Vec<LocalPointerAndPath> {
        let mut pointers = vec![];
        self.iter()
            .for_each(|a| {
                pointers.append(&mut a.1.get_pointers_with_path(full_path, start_with));
            });
        pointers
    }
    
    pub fn iter(&self) -> impl Iterator<Item=(&Section, &StructInterface)> {
        self.0.0.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&Section, &mut StructInterface)> {
        self.0.0.iter_mut()
    }

    pub fn share(&self) -> Self {
        Self((self.0.0
                  .iter()
                  .map(|(section, field)| {
                      (*section, field.as_ref()
                          .iter()
                          .map(|(field, pointer)| {
                              (
                                  field.clone(),
                                  pointer.clone(),
                              )
                          })
                          .collect())
                  })
                  .collect(),
              self.0.1.as_ref().cloned()))
    }

    pub fn new() -> Self {
        Self((HashMap::new(), None))
    }

    pub fn entry(&mut self, section: Section) -> Entry<Section, StructInterface> {
        self.0.0.entry(section)
    }

    /// Change the return pointer with the one provided in param
    pub fn swap_return(&mut self, _return: LocalPointer) {
        self.0.1 = Some(_return.clone());
    }

    pub fn get(&self, section: &Section) -> Option<&StructInterface> {
        self.0.0.get(section)
    }

    pub fn get_mut(&mut self, section: &Section) -> Option<&mut StructInterface> {
        self.0.0.get_mut(section)
    }

    pub fn get_return(&self) -> &Option<LocalPointer> {
        &self.0.1
    }

    pub fn get_mut_return(&mut self) -> &mut Option<LocalPointer> {
        &mut self.0.1
    }

    pub fn try_replace_pointer_nested(&mut self, path: &[usize], other: &LocalPointer) -> Option<LocalPointer> {
        if path.is_empty() {
            return None;
        }

        let key = &path[0];
        let next_path = &path[1..];

        if get_string(*key) == "return" {
            return self.0.1.clone();
        }

        self.0.0.iter_mut().find_map(|(_section, members)| {
            members.as_mut().get_mut(key)
                .and_then(|f| {
                    if next_path.is_empty() || f.is_primitive() {
                        f.replace_pointer(other);
                        Some(f.clone())
                    } else {
                        match f.as_ref().borrow_mut().deref_mut() {
                            LocalType::PlcStruct(_struct) => _struct.try_replace_pointer_nested(next_path, other),
                            LocalType::PlcArray(array) => array.try_replace_pointer_nested(next_path, other),
                            LocalType::FbInstance(instance) => instance.try_replace_pointer_nested(next_path, other),
                            _ => None
                        }
                    }
                })
        })
    }

    pub fn try_get_nested(&self, path: &[usize]) -> Option<LocalPointer> {
        if path.is_empty() {
            return None;
        }

        let key = &path[0];
        let next_path = &path[1..];

        if get_string(*key) == "return" {
            return self.0.1.clone();
        }

        self.0.0.iter().find_map(|(_section, members)| {
            members.as_ref().get(key)
                .and_then(|f| {
                    if next_path.is_empty() || f.is_primitive() {
                        Some(f.clone())
                    } else {
                        match f.as_ref().borrow().deref() {
                            LocalType::PlcStruct(_struct) => _struct.try_get_nested(next_path),
                            LocalType::PlcArray(array) => array.try_get_nested(next_path),
                            LocalType::FbInstance(instance) => instance.try_get_nested(next_path),
                            _ => None
                        }
                    }
                })
        })
    }
}
