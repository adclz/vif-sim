﻿#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(unused_features)]
#![allow(unused_imports)]
#[cfg(target_arch = "wasm32")]
pub mod js;
pub mod parser;
pub mod kernel;
pub mod container;
#[cfg(test)]
mod tests;