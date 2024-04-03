﻿use crate::parser::body::json_target::JsonTarget;
use crate::plc::complex::instance::private::PrivateInstanceAccessors;
use crate::plc::interface::section_interface::SectionInterface;
use crate::registry::local::pointer::LocalPointer;

pub trait PublicInstanceAccessors {
    fn get_interface(&self) -> &SectionInterface;
    //fn get_mut_interface(&mut self) -> &mut SectionInterface;
    fn get_body(&self) -> &Vec<JsonTarget>;
}

pub trait PublicInstanceTrait {
    fn try_get_nested(&self, path: &[String]) -> Option<LocalPointer>;
    fn try_replace_pointer_nested(&mut self, path: &[String], other: &LocalPointer) -> Option<LocalPointer>;
}


impl <T: PrivateInstanceAccessors + PublicInstanceAccessors> PublicInstanceTrait for T {
    fn try_replace_pointer_nested(&mut self, path: &[String], other: &LocalPointer)-> Option<LocalPointer> {
        self.get_mut_interface().try_replace_pointer_nested(path, other)
    }

    fn try_get_nested(&self, path: &[String]) -> Option<LocalPointer> {
        self.get_interface().try_get_nested(path)
    }
}