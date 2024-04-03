use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;
use crate::plc::complex::instance::public::PublicInstanceTrait;
use crate::plc::primitives::family_traits::{IsFamily, ToggleMonitor};
use crate::plc::primitives::primitive_traits::RawMut;
use crate::registry::local::pointer::{LocalPointer, LocalPointerAndPath};
use crate::registry::local::r#type::LocalType;

pub struct StructInterface(HashMap<String, LocalPointer>);

impl Clone for StructInterface {
    fn clone(&self) -> Self {
        Self(self.0
            .iter()
            .map(|(field, pointer)| {
                (
                    field.clone(),
                    LocalPointer::new(pointer.as_ref().borrow().deref().clone())
                )
            }).collect()
        )
    }
}

impl From<HashMap<String, LocalPointer>> for StructInterface {
    fn from(value: HashMap<String, LocalPointer>) -> Self {
        Self(value)
    }
}

impl FromIterator<(String, LocalPointer)> for StructInterface {
    fn from_iter<T: IntoIterator<Item=(String, LocalPointer)>>(iter: T) -> Self {
        let mut map = StructInterface::from(HashMap::with_hasher(Default::default()));
        map.as_mut().extend(iter);
        map
    }
}

impl Display for StructInterface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_for_each(|(name, pointer)| {
            writeln!(f, "\t '{}' -> {}", name, pointer)
        })
    }
}

impl Serialize for StructInterface {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl AsRef<HashMap<String, LocalPointer>> for StructInterface {
    fn as_ref(&self) -> &HashMap<String, LocalPointer> {
        &self.0
    }
}

impl AsMut<HashMap<String, LocalPointer>> for StructInterface {
    fn as_mut(&mut self) -> &mut HashMap<String, LocalPointer> {
        &mut self.0
    }
}

impl ToggleMonitor for StructInterface {
    fn set_monitor(&mut self, activate: bool) {
        self.0.iter()
            .for_each(|x| x.1.as_ref().borrow_mut().deref_mut().set_monitor(activate))
    }
}

impl StructInterface {
    pub fn get_raw_pointers(&self) -> Vec<*mut dyn RawMut> {
        self.iter()
            .fold(vec![], |_all, p| p.1.get_raw_pointers())
    }

    pub fn get_pointers_with_path(&self, full_path: &[String], start_with: &[String]) -> Vec<LocalPointerAndPath> {
        let mut pointers = vec![];
        self.iter()
            .for_each(|a| {
                let mut expanded_path = full_path.to_vec();
                expanded_path.push(a.0.clone());
                pointers.append(&mut a.1.get_pointers_with_path(&expanded_path, start_with));
            });
        pointers
    }

    pub fn iter(&self) -> impl Iterator<Item=(&String, &LocalPointer)> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&String, &mut LocalPointer)> {
        self.0.iter_mut()
    }

    pub fn len(&self) -> usize { self.0.len() }

    pub fn share(&self) -> Self {
        Self(self.0
            .iter()
            .map(|(field, pointer)| {
                (
                    field.clone(),
                    pointer.clone()
                )
            }).collect()
        )
    }

    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, name: &str) -> Option<&LocalPointer> {
        self.0.get(name)
    }

    pub fn try_replace_pointer_nested(&mut self, path: &[String], other: &LocalPointer) -> Option<LocalPointer> {
        if path.is_empty() {
            return None;
        }

        let key = &path[0];
        let next_path = &path[1..];

        self.0.get_mut(key).and_then(|f| {
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
    }

    pub fn try_get_nested(&self, path: &[String]) -> Option<LocalPointer> {
        if path.is_empty() {
            return None;
        }

        let key = &path[0];
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