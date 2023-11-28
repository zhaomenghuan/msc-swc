#![deny(clippy::all)]

mod utils;

use std::path::Path;
use std::sync::Arc;
use anyhow::Context as _;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use swc_core::{
  base::{config::Options, TransformOutput, Compiler},
  common::{FilePathMapping, SourceMap, FileName, comments::SingleThreadedComments},
  ecma::{ast::{Program, Module, Script}, visit::Fold},
  node::{deserialize_json, get_deserialized, MapErr}
};
use crate::utils::{init_default_trace_subscriber, try_with};

pub fn noop() -> impl Fold {
  Noop
}

struct Noop;
impl Fold for Noop {
  #[inline(always)]
  fn fold_module(&mut self, m: Module) -> Module {
      m
  }

  #[inline(always)]
  fn fold_script(&mut self, s: Script) -> Script {
      s
  }
}

#[napi]
pub fn swc_transform_sync(s: String, is_module: bool, opts: Buffer) -> napi::Result<TransformOutput> {
  init_default_trace_subscriber();

  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
  let c = Compiler::new(cm.clone());

  let mut options: Options = get_deserialized(&opts)?;

  if !options.filename.is_empty() {
    options.config.adjust(Path::new(&options.filename));
  }

  let error_format = options.experimental.error_format.unwrap_or_default();

  try_with(
    c.cm.clone(),
    !options.config.error.filename.into_bool(),
    error_format,
    |handler| {
      c.run(|| {
        if is_module {
          let program: Program =
            deserialize_json(s.as_str()).context("failed to deserialize Program")?;
          c.process_js(handler, program, &options)
        } else {
          let fm = c.cm.new_source_file(
            if options.filename.is_empty() {
              FileName::Anon
            } else {
              FileName::Real(options.filename.clone().into())
            },
            s,
          );

          c.process_js_with_custom_pass(
            fm,
            None,
            handler,
            &options,
            SingleThreadedComments::default(),
            |_| {
              dbg!("custom_before_pass");
              noop()
            },
            |_| {
              dbg!("custom_after_pass");
              noop()
            }
          )
        }
      })
    },
  )
  .convert_err()
}
