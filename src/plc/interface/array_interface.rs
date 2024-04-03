use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;
use crate::plc::complex::instance::public::PublicInstanceTrait;
use crate::plc::primitives::family_traits::{IsFamily, ToggleMonitor};
use crate::plc::primitives::primitive_traits::RawMut;
use crate::registry::local::pointer::{LocalPointer, LocalPointerAndPath};
use crate::registry::local::r#type::LocalType;

pub struct ArrayInterface(Vec<LocalPointer>);

impl Display for ArrayInterface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut i = -1;
        self.0.iter().try_for_each(|member| {
            i += 1;
            writeln!(f, "\t [{}]: {}", i, member)
        })
    }
}

impl Clone for ArrayInterface {
    fn clone(&self) -> Self {
        Self(self.0.iter()
            .map(|pointer| {
                LocalPointer::new(pointer.as_ref().borrow().deref().clone())
            })
            .collect())
    }
}

impl From<Vec<LocalPointer>> for ArrayInterface {
    fn from(value: Vec<LocalPointer>) -> Self {
        Self(value)
    }
}

impl ToggleMonitor for ArrayInterface {
    fn set_monitor(&mut self, activate: bool) {
        self
            .iter()
            .for_each(|x| x.as_ref().borrow_mut().deref_mut().set_monitor(activate))
    }
}

impl Serialize for ArrayInterface {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut array = serializer.serialize_seq(Some(self.0.len()))?;
        for element in &self.0 {
            array.serialize_element(element)?;
        }
        array.end()
    }
}

impl ArrayInterface {
    pub fn get_raw_pointers(&self) -> Vec<*mut dyn RawMut> {
        self.iter()
            .fold(vec![], |_all, p| p.get_raw_pointers())
    }

    pub fn get_pointers_with_path(&self, full_path: &[String], start_with: &[String]) -> Vec<LocalPointerAndPath> {
        let mut pointers = vec![];
        self.iter()
            .enumerate()
            .for_each(|a| {
                let mut expanded_path = full_path.to_vec();
                expanded_path.push(format!("[{}]", a.0.to_string()));
                pointers.append(&mut a.1.get_pointers_with_path(&expanded_path, start_with));
            });
        pointers
    }

    pub fn iter(&self) -> impl Iterator<Item=&LocalPointer> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut LocalPointer> {
        self.0.iter_mut()
    }
    
    pub fn share(&self) -> Self {
        Self(self.0.iter().cloned()
            .collect())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn new() -> Self {
        Self(Vec::new())
    }
    
    pub fn get(&self, index: usize) -> Option<&LocalPointer> {
        self.0.get(index)
    }

    pub fn try_replace_pointer_nested(&mut self, path: &[String], other: &LocalPointer) -> Option<LocalPointer> {
        if path.is_empty() {
            return None;
        }

        let key = &path[0];
        let next_path = &path[1..];

        let key = path[0]
            .replace(['[', ']'], "")
            .parse::<u64>().unwrap() as usize;
        let next_path = &path[1..];

        self.0.get_mut(key).and_then(|f| {
            if next_path.is_empty() || f.is_primitive() {
                f.replace_pointer(other);
                Some(f.clone())
            } else {
                match f.as_ref().borrow_mut().deref_mut() {
                    LocalType::PlcStruct(_struct) => _struct.try_replace_pointer_nested(path, other),
                    LocalType::PlcArray(array) => array.try_replace_pointer_nested(path, other),
                    LocalType::FbInstance(instance) => instance.try_replace_pointer_nested(path, other),
                    _ => None
                }
            }
        })
    }

    pub fn try_get_nested(&self, path: &[String]) -> Option<LocalPointer> {
        if path.is_empty() {
            return None;
        }

        let key = path[0]
            .replace(['[', ']'], "")
            .parse::<u64>().unwrap() as usize;
        let next_path = &path[1..];

        self.0.get(key).and_then(|f| {
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
    }
}
