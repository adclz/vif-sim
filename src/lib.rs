#![feature(const_trait_impl)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(unused_features)]
#[cfg(target_arch = "wasm32")]
pub mod js;
pub mod parser;
pub mod plc;
pub mod registry;
pub mod container;