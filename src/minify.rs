use napi::bindgen_prelude::Buffer;
use std::sync::Arc;

use crate::utils::{init_default_trace_subscriber, try_with};
use serde::Deserialize;
use swc_core::{
  base::{config::ErrorFormat, Compiler, TransformOutput},
  common::{collections::AHashMap, sync::Lrc, FileName, FilePathMapping, SourceFile, SourceMap},
  node::{get_deserialized, MapErr},
};

#[derive(Deserialize)]
#[serde(untagged)]
enum MinifyTarget {
  /// Code to minify.
  Single(String),
  /// `{ filename: code }`
  Map(AHashMap<String, String>),
}

impl MinifyTarget {
  fn to_file(&self, cm: Lrc<SourceMap>) -> Lrc<SourceFile> {
    match self {
      MinifyTarget::Single(code) => cm.new_source_file(FileName::Anon, code.clone()),
      MinifyTarget::Map(codes) => {
        assert_eq!(
          codes.len(),
          1,
          "swc.minify does not support concatting multiple files yet"
        );

        let (filename, code) = codes.iter().next().unwrap();
        cm.new_source_file(FileName::Real(filename.clone().into()), code.clone())
      }
    }
  }
}

#[napi]
pub fn minify_sync(code: Buffer, opts: Buffer) -> napi::Result<TransformOutput> {
  init_default_trace_subscriber();
  let code: MinifyTarget = get_deserialized(code)?;
  let opts = get_deserialized(opts)?;

  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
  let c = Compiler::new(cm.clone());
  let fm = code.to_file(c.cm.clone());

  try_with(
    c.cm.clone(),
    false,
    // TODO(kdy1): Maybe make this configurable?
    ErrorFormat::Normal,
    |handler| c.minify(fm, handler, &opts),
  )
  .convert_err()
}
