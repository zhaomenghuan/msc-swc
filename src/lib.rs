#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

extern crate swc_malloc;

pub mod minify;
pub mod module_resolver;
pub mod transform;
pub mod utils;
