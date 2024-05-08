use std::cell::RefCell;
use std::ops::{Deref};
use std::rc::Rc;
use serde::{Serialize, Serializer};
use serde::ser::{SerializeSeq, SerializeStruct};
use tsify::Tsify;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::kernel::registry::get_string;

#[derive(Default, Tsify)]
struct VecSectionOrLog(Vec<SectionOrLog>);

impl Serialize for VecSectionOrLog {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_seq(Some(self.0.len()))?;
        for a in &self.0 {
            state.serialize_element(a)?;
        }
        state.end()
    }
}

impl Clone for VecSectionOrLog {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Tsify)]
pub enum SectionOrLog {
    Log(String),
    Section(Rc<RefCell<Section>>),
}

impl Clone for SectionOrLog {
    fn clone(&self) -> Self {
        match &self {
            SectionOrLog::Log(a) => Self::Log(a.clone()),
            SectionOrLog::Section(a) => Self::Section(Rc::new(RefCell::new(a.borrow().clone())))
        }
    }
}

impl Serialize for SectionOrLog {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            SectionOrLog::Log(log) => serializer.serialize_str(&log),
            SectionOrLog::Section(section) => section.borrow().deref().serialize(serializer)
        }
    }
}

#[derive(Default, Tsify)]
pub struct Section {
    name: usize,
    ty: String,
    content: VecSectionOrLog,
}

impl Serialize for Section {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("section", 4)?;
        state.serialize_field("name", &get_string(self.name))?;
        state.serialize_field("ty", &self.ty)?;
        state.serialize_field("content", &self.content)?;
        state.end()
    }
}

impl Section {
    fn new(name: usize, ty: &str) -> Self {
        Self {
            name,
            ty: ty.into(),
            content: VecSectionOrLog(vec![]),
        }
    }

    pub fn insert_log(&mut self, log: &str) {
        //println!("INSERT Log -> '{}' to {}", &log, self.name);
        self.content.0.push(SectionOrLog::Log(log.into()));
    }

    pub fn insert_section(&mut self, section: &Rc<RefCell<Section>>) {
        //println!("INSERT SECTION -> {} to {}", &section.borrow().name, self.name);
        self.content.0.push(SectionOrLog::Section(section.clone()));
    }
}

impl Clone for Section {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            ty: self.ty.clone(),
            content: self.content.clone(),
        }
    }
}

#[wasm_bindgen(skip_typescript)]
#[derive(Default, Tsify)]
pub struct Stack {
    current: Option<Rc<RefCell<Section>>>,
    stack: Vec<Rc<RefCell<Section>>>
}

impl Clone for Stack {
    fn clone(&self) -> Self {
        Self {
            current: self.current.as_ref().map(|a| Rc::new(RefCell::new(a.borrow().clone()))),
            stack: self.stack.clone(),
        }
    }
}

impl Stack {
    pub fn new() -> Stack {
        Self {
            current: None,
            stack: vec!(),
        }
    }

    pub fn clear(&mut self) {
        self.current = None;
        self.stack.clear();
    }

    pub fn add_section(&mut self, name: usize, ty: &str) -> usize {
        let new = Rc::new(RefCell::new(Section::new(name, ty)));
        let mut index = 0;
        match &self.current {
            None => {}
            Some(a) => {
                index = self.stack.iter().position(|x| x.borrow().name == a.borrow().name).unwrap();
                a.borrow_mut().insert_section(&new)
            }
        }
        self.stack.push(new.clone());
        self.current = Some(new.clone());
        //println!("Open section {}", &self.current.as_ref().unwrap().borrow().name);
        index + 1
    }

    pub fn get_current_section(&mut self) -> Option<Rc<RefCell<Section>>> {
        //println!("Current -> {}", self.current.as_ref().unwrap().borrow().name);
        self.current.as_ref().cloned()
    }

    pub fn go_back_to_section(&mut self, index: usize) {
        match self.stack.is_empty() {
            true => {},
            false => {
                self.current = Some(self.stack[index - 1].clone())
            },
        }
        //println!("Back to section {}",  self.current.as_ref().unwrap().borrow().name);

    }

    pub fn insert_log(&mut self, log: &str) {
        match &self.current {
            None => {}
            Some(cur) => cur.borrow_mut().insert_log(log)
        }
    }

    pub fn serialize(&self) -> JsValue {
        match &self.stack.is_empty() {
            true => JsValue::null(),
            false => serde_wasm_bindgen::to_value(&self.stack[0].borrow().deref()).unwrap()
        }
    }
}
