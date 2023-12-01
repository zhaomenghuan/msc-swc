#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

extern crate swc_malloc;

mod minify;
mod module_resolver;
mod transform;
mod utils;
