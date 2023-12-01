use crate::utils::normalize_path;
use std::collections::HashSet;
use std::path::PathBuf;
use swc_core::common::Span;
use swc_core::{
  atoms::JsWord,
  common::{errors::HANDLER, sync::Lazy, FileName, DUMMY_SP},
  ecma::{
    ast::{
      CallExpr, Callee, Expr, ExprOrSpread, ImportDecl, Lit, Module, ModuleDecl, ModuleItem, Str,
    },
    visit::{VisitMut, VisitMutWith},
  },
};

static DEFAULT_EXTENSIONS: Lazy<Vec<&'static str>> =
  Lazy::new(|| vec!["js", "ts", "jsx", "tsx", "json"]);

static EXTERNALS_PACKAGE: Lazy<HashSet<String>> = Lazy::new(|| {
  let mut set = HashSet::new();
  set.insert("react".to_string());
  set.insert("@mtfe/msc-rlist".to_string());
  set
});

pub struct TransformResult {
  pub absolute_path: Option<String>,
  pub transformed_path: Option<String>,
}

fn process_transform(
  node_span: Span,
  cwd: PathBuf,
  filename: FileName,
  required_file_path: String,
) -> TransformResult {
  // 内置的依赖 react、@mtfe/msc-rlist 由基础库解析
  if EXTERNALS_PACKAGE.contains(&*required_file_path) {
    return TransformResult {
      absolute_path: None,
      transformed_path: None,
    };
  }

  let absolute_path: Option<String>;
  let transformed_path: Option<String>;
  let is_cwd_exists = cwd.exists();

  // 获取文件完整路径
  let cwd = if is_cwd_exists {
    cwd.display().to_string()
  } else {
    "/".to_string()
  };
  let mut required_file_full_path = resolve_required_file_path(
    cwd.clone(),
    filename.to_string(),
    required_file_path.clone(),
  );

  // 是否是默认扩展名称, 否则默认兜底为 .js 后缀
  if !is_default_extension(&required_file_full_path) {
    required_file_full_path = PathBuf::from(format!(
      "{}.js",
      required_file_full_path.to_str().unwrap().to_string()
    ));
  }

  // 如果 cwd 存在，则判断文件是否存在
  if is_cwd_exists {
    let mut is_exists: bool = false;
    // 判断默认扩展名的文件是否存在
    for extension in DEFAULT_EXTENSIONS.iter() {
      let mut path = required_file_full_path.clone();
      if path.extension().and_then(|s| s.to_str()) != Some(extension) {
        path.set_extension(extension);
      }
      is_exists = path.exists();
      if is_exists {
        required_file_full_path = path.clone();
        break;
      }
    }

    // TODO: 文件不存在时判断 npm 包是否存在
    if is_exists {
      // 保存引用文件的绝对路径
      absolute_path = Some(required_file_full_path.to_str().unwrap().to_string());
      // 替换为 js 扩展名
      required_file_full_path = replace_to_js_extension(&required_file_full_path);
      // 替换引用文件的路径
      transformed_path = Some(
        required_file_full_path
          .to_str()
          .unwrap()
          .replace(&cwd.as_str(), "")
          .replace("\\", "/"),
      );
    } else {
      absolute_path = None;
      transformed_path = None;
      HANDLER.with(|handler| {
        handler
          .struct_span_err(
            node_span,
            format!("{filename} 文件中引入的 {required_file_path} 文件不存在").as_str(),
          )
          .emit();
      });
    }
  } else {
    // 保存引用文件的绝对路径
    absolute_path = Some(required_file_full_path.to_str().unwrap().to_string());
    // 替换为 js 扩展名
    required_file_full_path = replace_to_js_extension(&required_file_full_path);
    // 替换引用文件的路径
    transformed_path = Some(required_file_full_path.to_str().unwrap().replace("\\", "/"));
  }
  TransformResult {
    absolute_path,
    transformed_path,
  }
}

pub struct ModuleResolverVisit<'a> {
  // 源码根目录
  pub cwd: PathBuf,
  // 文件名称
  pub filename: FileName,
  // 引用文件
  pub requires: &'a mut HashSet<String>,
}

// Implement necessary visit_mut_* methods for actual custom transform.
// A comprehensive list of possible visitor methods can be found here:
// https://rustdoc.swc.rs/swc_ecma_visit/trait.VisitMut.html
impl<'a> VisitMut for ModuleResolverVisit<'a> {
  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if let Callee::Expr(e) = &n.callee {
      if let Expr::Ident(i) = &**e {
        if i.sym == *"require" && n.args.len() == 1 {
          if let Expr::Lit(Lit::Str(module_name)) = &*n.args[0].expr {
            let required_file_path = module_name.value.to_string();
            let result = process_transform(
              n.span,
              self.cwd.clone(),
              self.filename.clone(),
              required_file_path,
            );
            if result.absolute_path.is_some() {
              self.requires.insert(result.absolute_path.unwrap());
            }

            if result.transformed_path.is_some() {
              *n = CallExpr {
                span: DUMMY_SP,
                callee: n.callee.clone(),
                args: vec![ExprOrSpread {
                  expr: result.transformed_path.clone().unwrap().into(),
                  spread: None,
                }],
                type_args: None,
              };
            }
          }
        }
      }
    }
  }

  fn visit_mut_import_decl(&mut self, n: &mut ImportDecl) {
    // 忽略样式文件
    if is_style(&n.src.value) {
      return;
    }

    let required_file_path = n.src.value.to_string();
    let result = process_transform(
      n.span,
      self.cwd.clone(),
      self.filename.clone(),
      required_file_path,
    );
    if result.absolute_path.is_some() {
      self.requires.insert(result.absolute_path.unwrap());
    }

    if result.transformed_path.is_some() {
      n.src = Box::new(Str::from(result.transformed_path.unwrap()));
    }
  }

  fn visit_mut_module(&mut self, n: &mut Module) {
    n.visit_mut_children_with(self);
    // 移除 JS 文件引入的样式文件
    n.body.retain(|item| {
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(i)) = item {
        return !is_style(&i.src.value);
      }
      true
    })
  }
}

///
/// 判断文件扩展名是否是默认支持的扩展名
///
fn is_default_extension(path: &PathBuf) -> bool {
  if let Some(ext) = path.extension() {
    return DEFAULT_EXTENSIONS.contains(&ext.to_str().unwrap());
  }

  return false;
}

///
/// 替换为 js 扩展名
///
fn replace_to_js_extension(path: &PathBuf) -> PathBuf {
  let mut p = path.clone();
  let required_file_ext = p.extension().and_then(|s| s.to_str());
  if required_file_ext == Some("ts")
    || required_file_ext == Some("jsx")
    || required_file_ext == Some("tsx")
  {
    p.set_extension("js");
  }
  return p;
}

///
/// 判断是否是样式文件
///
fn is_style(source: &JsWord) -> bool {
  return source.ends_with(".css") || source.ends_with(".scss") || source.ends_with(".sass");
}

///
/// 解析引入文件的路径
///
/// source_file_path: 源文件路径
/// required_file_path: 引入文件的路径
///
pub fn resolve_required_file_path(
  cwd: String,
  source_file_path: String,
  required_file_path: String,
) -> PathBuf {
  if required_file_path.starts_with('/') {
    return PathBuf::from(required_file_path);
  }

  let required_file_full_path = PathBuf::from(cwd)
    .join(PathBuf::from(source_file_path))
    .with_file_name(required_file_path);
  return normalize_path(required_file_full_path);
}