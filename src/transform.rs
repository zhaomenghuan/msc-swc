use napi::bindgen_prelude::Buffer;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use swc_core::{
  base::{config::Options, Compiler},
  common::{comments::SingleThreadedComments, FileName, FilePathMapping, SourceMap},
  ecma::{transforms::base::pass::noop, visit::as_folder},
  node::{get_deserialized, MapErr},
};

use crate::module_resolver::ModuleResolverVisit;
use crate::utils::{init_default_trace_subscriber, try_with};

#[napi_derive::napi(object)]
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TransformerCustomOptions {
  #[serde(default)]
  pub external_packages: Vec<String>,
}

#[napi_derive::napi(object)]
#[derive(Debug, Serialize)]
pub struct Metadata {
  pub requires: Vec<String>,
}

#[napi_derive::napi(object)]
#[derive(Debug, Serialize)]
pub struct TransformOutput {
  pub code: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub map: Option<String>,
  pub metadata: Metadata,
}

#[napi]
pub fn transform_sync(
  s: String,
  opts: Buffer,
  custom_opts: Option<Buffer>,
) -> napi::Result<TransformOutput> {
  init_default_trace_subscriber();

  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
  let c = Compiler::new(cm.clone());

  let mut options: Options = get_deserialized(&opts)?;
  let custom_options: TransformerCustomOptions = if custom_opts.is_some() {
    get_deserialized(&custom_opts.unwrap())?
  } else {
    TransformerCustomOptions::default()
  };

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
        let filename = if options.filename.is_empty() {
          FileName::Anon
        } else {
          FileName::Real(options.filename.clone().into())
        };
        let fm = c.cm.new_source_file(filename.clone(), s);

        let mut requires: HashSet<String> = HashSet::new();
        let result = c
          .process_js_with_custom_pass(
            fm,
            None,
            handler,
            &options,
            SingleThreadedComments::default(),
            |_| {
              as_folder(ModuleResolverVisit {
                cwd: options.cwd.clone(),
                filename: filename.clone(),
                external_packages: custom_options.external_packages.clone(),
                requires: &mut requires,
              })
            },
            |_| noop(),
          )
          .unwrap();

        let metadata = Metadata {
          requires: requires.clone().into_iter().collect(),
        };
        Ok(TransformOutput {
          code: result.code,
          map: result.map.clone(),
          metadata,
        })
      })
    },
  )
  .convert_err()
}
