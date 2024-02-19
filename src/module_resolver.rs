use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use swc_core::ecma::ast::{FnDecl, FnExpr, Pat};
use swc_core::{
  atoms::JsWord,
  common::{errors::HANDLER, sync::Lazy, FileName, Span, DUMMY_SP},
  ecma::{
    ast::{
      CallExpr, Callee, Expr, ExprOrSpread, ImportDecl, Lit, Module, ModuleDecl, ModuleItem, Str,
    },
    visit::{VisitMut, VisitMutWith},
  },
};

use crate::utils::normalize_path;

static DEFAULT_EXTENSIONS: Lazy<Vec<&'static str>> =
  Lazy::new(|| vec!["js", "ts", "jsx", "tsx", "json"]);

pub struct TransformResult {
  pub absolute_path: Option<String>,
  pub transformed_path: Option<String>,
}

///
/// 判断默认扩展名的文件是否存在
/// eg:
/// ./utils 解析优先级: ./utils.js -> ./utils.ts -> ... -> ./utils/index.js -> ./utils/index.ts -> ...
/// ./utils.wx 解析优先级: ./utils.wx.js -> ./utils.wx.ts -> ... -> ./utils.wx/index.js -> ./utils.wx/index.ts -> ...
///
fn resolve_file_path(file_path: PathBuf) -> Option<PathBuf> {
  let file_path_string = file_path.to_str().unwrap();
  let mut file_absolute_path: Option<PathBuf> = None;
  // 处理默认扩展名且文件存在
  if is_default_extension(&file_path) && file_path.exists() {
    return Some(file_path);
  }

  // 处理缺省扩展名且文件存在的情况
  for extension in DEFAULT_EXTENSIONS.iter() {
    let new_file_path = PathBuf::from(format!("{}.{}", file_path_string, extension));
    if new_file_path.exists() {
      file_absolute_path = Some(new_file_path.clone());
      break;
    }
  }

  if file_absolute_path.is_some() {
    return file_absolute_path;
  }

  // 处理缺省扩展名且文件夹下 index.{js|ts|...} 文件存在的情况
  for extension in DEFAULT_EXTENSIONS.iter() {
    let new_file_path = PathBuf::from(format!("{}/index.{}", file_path_string, extension));
    if new_file_path.exists() {
      file_absolute_path = Some(new_file_path.clone());
      break;
    }
  }

  file_absolute_path
}

///
/// 处理 npm 包
///
fn resolve_node_modules_file(
  cwd: String,
  source_file_path: String,
  required_file_path: String,
) -> Option<PathBuf> {
  let mut file_absolute_path: Option<PathBuf> = None;
  let source_file_absolute_path = PathBuf::from(cwd.clone()).join(PathBuf::from(source_file_path));

  let mut parent_path = source_file_absolute_path.parent();
  while let Some(path) = parent_path {
    if !path.starts_with(&cwd.clone()) {
      break;
    }
    let package_file_path = path.join("node_modules").join(&required_file_path);
    let resolved_file_path = resolve_file_path(package_file_path.clone());
    if resolved_file_path.is_some() {
      file_absolute_path = resolved_file_path.clone();
      break;
    }

    // 处理 package.json main 入口逻辑
    let package_json_path = package_file_path.join("package.json");
    if package_json_path.exists() {
      let package_json = fs::read_to_string(&package_json_path).unwrap();
      let package: serde_json::Value = serde_json::from_str(&package_json).unwrap();
      // 解析 main 字段
      if let Some(main) = package.get("main") {
        let main_path = package_file_path.join(main.as_str().unwrap());
        if main_path.exists() {
          file_absolute_path = Some(main_path);
          break;
        }
      }

      // 解析 module 字段
      if let Some(module) = package.get("module") {
        let module_path = package_file_path.join(module.as_str().unwrap());
        if module_path.exists() {
          file_absolute_path = Some(module_path);
          break;
        }
      }
    }

    parent_path = path.parent();
  }

  file_absolute_path
}

fn process_transform(
  node_span: Span,
  cwd: PathBuf,
  filename: FileName,
  required_file_path: String,
  external_packages: Vec<String>,
) -> TransformResult {
  // Node 内置模块 & 外部依赖
  if is_builtin_module(&required_file_path) || external_packages.contains(&required_file_path) {
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

  // 如果 cwd 存在，则判断文件是否存在
  if is_cwd_exists {
    // 处理文件扩展名或 index 简写的模式
    let required_file_full_path =
      resolve_file_path(required_file_full_path.clone()).or_else(|| {
        resolve_node_modules_file(
          cwd.clone(),
          filename.to_string(),
          required_file_path.clone(),
        )
      });

    match required_file_full_path {
      Some(mut path) => {
        // 保存引用文件的绝对路径
        absolute_path = Some(path.to_str().unwrap().to_string());
        // 替换为 js 扩展名
        path = replace_to_js_extension(&path);
        // 替换引用文件的路径
        transformed_path = Some(
          path
            .to_str()
            .unwrap()
            .replace(cwd.as_str(), "")
            .replace('\\', "/"),
        );
      }
      None => {
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
    }
  } else {
    // 保存引用文件的绝对路径
    absolute_path = Some(required_file_full_path.to_str().unwrap().to_string());
    // 替换为 js 扩展名
    required_file_full_path = replace_to_js_extension(&required_file_full_path);
    // 替换引用文件的路径
    transformed_path = Some(required_file_full_path.to_str().unwrap().replace('\\', "/"));
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
  // 外部 NPM 包
  pub external_packages: Vec<String>,
  // 引用文件
  pub requires: &'a mut HashSet<String>,
  // require 作为函数参数时的函数深度
  pub require_as_scope_bind_depth: i32,
}

// Implement necessary visit_mut_* methods for actual custom transform.
// A comprehensive list of possible visitor methods can be found here:
// https://rustdoc.swc.rs/swc_ecma_visit/trait.VisitMut.html
impl<'a> VisitMut for ModuleResolverVisit<'a> {
  ///
  /// # require 作为具名函数参数
  ///
  /// require 作为函数参数时，不解析依赖，对此类函数层级做标记，当包含 require 为参数的函数层级为 0 时，收集依赖
  /// ```
  /// function add(require)
  /// {
  ///   require("./xxx")
  /// }
  /// ```
  fn visit_mut_fn_decl(&mut self, n: &mut FnDecl) {
    let original_require_as_scope_bind_depth = self.require_as_scope_bind_depth;
    let has_require_param = n.function.params.iter().any(|param| {
      if let Pat::Ident(ident) = &param.pat {
        ident.sym == "require"
      } else {
        false
      }
    });

    // 具名函数包含 require 为参数
    if has_require_param {
      self.require_as_scope_bind_depth += 1;
    }

    n.visit_mut_children_with(self);
    self.require_as_scope_bind_depth = original_require_as_scope_bind_depth;
  }

  ///
  /// # require 作为匿名函数参数
  ///
  /// require 作为函数参数时，不解析依赖，对此类函数层级做标记，当包含 require 为参数的函数层级为 0 时，收集依赖
  /// ```
  /// funCall(a, function(require) {
  ///   require("./xxx")
  /// })
  /// ```
  fn visit_mut_fn_expr(&mut self, n: &mut FnExpr) {
    let original_require_as_scope_bind_depth = self.require_as_scope_bind_depth;
    let has_require_param = n.function.params.iter().any(|param| {
      if let Pat::Ident(ident) = &param.pat {
        ident.sym == "require"
      } else {
        false
      }
    });

    // 匿名函数包含 require 为参数
    if has_require_param {
      self.require_as_scope_bind_depth += 1;
    }

    n.visit_mut_children_with(self);
    self.require_as_scope_bind_depth = original_require_as_scope_bind_depth;
  }

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    n.visit_mut_children_with(self);
    if let Callee::Expr(e) = &n.callee {
      if let Expr::Ident(i) = &**e {
        if self.require_as_scope_bind_depth == 0 && i.sym == *"require" && n.args.len() == 1 {
          if let Expr::Lit(Lit::Str(module_name)) = &*n.args[0].expr {
            let required_file_path = module_name.value.to_string();
            let result = process_transform(
              n.span,
              self.cwd.clone(),
              self.filename.clone(),
              required_file_path,
              self.external_packages.clone(),
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
      required_file_path.clone(),
      self.external_packages.clone(),
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
fn is_default_extension(path: &Path) -> bool {
  if let Some(ext) = path.extension() {
    return DEFAULT_EXTENSIONS.contains(&ext.to_str().unwrap());
  }

  false
}

///
/// 替换为 js 扩展名
///
fn replace_to_js_extension(path: &Path) -> PathBuf {
  let mut p = path.to_path_buf();
  let required_file_ext = p.extension().and_then(|s| s.to_str());
  if required_file_ext == Some("ts")
    || required_file_ext == Some("jsx")
    || required_file_ext == Some("tsx")
  {
    p.set_extension("js");
  }
  p
}

///
/// 判断是否是样式文件
///
fn is_style(source: &JsWord) -> bool {
  source.ends_with(".css") || source.ends_with(".scss") || source.ends_with(".sass")
}

///
/// 判断是否是 Node.js 内置的 API
///
fn is_builtin_module(module: &str) -> bool {
  let builtin_modules = vec![
    "assert",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "https",
    "net",
    "os",
    "path",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "string_decoder",
    "timers",
    "tls",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "zlib",
  ];
  let module = module.strip_prefix("node:").unwrap_or(module);
  let module = module.split('/').next().unwrap();
  builtin_modules.contains(&module)
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
    return PathBuf::from(cwd).join(required_file_path.trim_start_matches('/'));
  }

  let required_file_full_path = PathBuf::from(cwd)
    .join(PathBuf::from(source_file_path))
    .with_file_name(required_file_path);
  normalize_path(required_file_full_path)
}
