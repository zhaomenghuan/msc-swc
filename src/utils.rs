use anyhow::{anyhow, Error};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Component, PathBuf};
use swc_core::{
  base::{config::ErrorFormat, try_with_handler},
  common::{errors::Handler, sync::Lrc, SourceMap, GLOBALS},
};
use tracing_subscriber::EnvFilter;

/// Trying to initialize default subscriber if global dispatch is not set.
/// This can be called multiple time, however subsequent calls will be ignored
/// as tracing_subscriber only allows single global dispatch.
pub fn init_default_trace_subscriber() {
  let _unused = tracing_subscriber::FmtSubscriber::builder()
    .without_time()
    .with_target(false)
    .with_writer(std::io::stderr)
    .with_ansi(true)
    .with_env_filter(EnvFilter::from_env("SWC_LOG"))
    .pretty()
    .try_init();
}

pub fn try_with<F, Ret>(
  cm: Lrc<SourceMap>,
  skip_filename: bool,
  _error_format: ErrorFormat,
  op: F,
) -> Result<Ret, Error>
where
  F: FnOnce(&Handler) -> Result<Ret, Error>,
{
  GLOBALS.set(&Default::default(), || {
    try_with_handler(
      cm,
      swc_core::base::HandlerOpts {
        skip_filename,
        ..Default::default()
      },
      |handler| {
        //
        let result = catch_unwind(AssertUnwindSafe(|| op(handler)));

        let p = match result {
          Ok(v) => return v,
          Err(v) => v,
        };

        if let Some(s) = p.downcast_ref::<String>() {
          Err(anyhow!("failed to handle: {}", s))
        } else if let Some(s) = p.downcast_ref::<&str>() {
          Err(anyhow!("failed to handle: {}", s))
        } else {
          Err(anyhow!("failed to handle with unknown panic message"))
        }
      },
    )
  })
}

///
/// 解析路径
/// 参考：https://github.com/effector/swc-plugin/blob/main/src/path.rs#L3
///
pub fn normalize_path(path: PathBuf) -> PathBuf {
  let mut components = path.components().peekable();
  let mut result = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
    components.next();
    PathBuf::from(c.as_os_str())
  } else {
    PathBuf::new()
  };

  for component in components {
    match component {
      Component::Prefix(..) => unreachable!(),
      Component::RootDir => {
        result.push(component.as_os_str());
      }
      Component::CurDir => {}
      Component::ParentDir => {
        result.pop();
      }
      Component::Normal(c) => {
        result.push(c);
      }
    }
  }
  return result;
}
