use std::borrow::Borrow;
use std::rc::Rc;

use swc_common::comments::SingleThreadedComments;
use swc_common::input::SourceFileInput;
use swc_common::{FileName, SourceMap};
use swc_ecmascript::ast::{Callee, Decl, EsVersion, Expr, Module, ModuleDecl, ModuleItem, Pat};
use swc_ecmascript::parser::lexer::Lexer;
use swc_ecmascript::parser::{Parser, Syntax, TsConfig};
use wasm_bindgen::prelude::*;

pub struct Found {
  callee: String,
  export: String,
}

struct FindExportsVisitor {
  matches: Vec<String>,
  exports: Vec<Found>,
}

impl Found {
  fn to_string(self) -> String {
    format!("{}/{}", self.callee, self.export)
  }
}

impl FindExportsVisitor {
  fn find_module(&mut self, module: &Module) {
    for item in &module.body {
      if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(decl)) = item {
        if let Decl::Var(var_decl) = &decl.decl {
          if let Some(decl) = var_decl.decls.first() {
            if let Pat::Ident(ident_export) = &decl.name {
              if let Some(init) = &decl.init {
                if let Expr::Call(call) = init.borrow() {
                  if let Callee::Expr(expr) = &call.callee {
                    if let Expr::Ident(ident_callee) = expr.borrow() {
                      let callee_name = ident_callee.sym.to_string();
                      let export_name = ident_export.sym.to_string();

                      if self.matches.contains(&callee_name) {
                        self.exports.push(Found {
                          callee: callee_name,
                          export: export_name,
                        })
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}

#[wasm_bindgen]
pub fn find_exports(source: &str, callees: Vec<String>) -> Vec<String> {
  let cm = Rc::new(SourceMap::default());
  let fm = cm.new_source_file(FileName::Custom("input.js".to_string()), source.to_string());

  let comments = SingleThreadedComments::default();
  let lexer = Lexer::new(
    Syntax::Typescript(TsConfig {
      tsx: true,
      ..Default::default()
    }),
    EsVersion::EsNext,
    SourceFileInput::from(&*fm),
    Some(&comments),
  );

  let mut parser = Parser::new_from(lexer);
  let module = parser.parse_module().unwrap();

  let mut visitor = FindExportsVisitor {
    matches: callees,
    exports: Vec::new(),
  };
  visitor.find_module(&module);

  visitor.exports.into_iter().map(|x| x.to_string()).collect()
}

#[cfg(test)]
mod test {
  use crate::find_exports;

  fn run_test(source: &str, callees: &[&str], expected: &[&str]) {
    let result = find_exports(source, callees.iter().map(|x| x.to_string()).collect());
    assert_eq!(result, expected);
  }

  #[test]
  fn support_pure_javascript() {
    run_test(
      r#"
      export const one = loader$(() => { return true; });
      export let two = action$(() => { return false; });
      export var the = loader$(() => { return false; });
    
      // below may works, but will not match
      const none = loader$(() => {}); export { none };
      export const none = (0, loader$)(() => {});
      export const [none = loader$(() => {})] = [];
      "#,
      &["loader$", "action$"],
      &["loader$/one", "action$/two", "loader$/the"],
    );
  }

  #[test]
  fn support_typescript() {
    run_test(
      r#"
      export const one: Loader = loader$<T>(() => { return true; });
      export const two: Action = action$<{ name: string }>(() => { return false; });
      export const the: {} | null = loader$(() => { return false; });
      "#,
      &["loader$", "action$"],
      &["loader$/one", "action$/two", "loader$/the"],
    );
  }

  #[test]
  fn support_jsx() {
    run_test(
      r#"
      export const one: Loader = loader$<div>(() => { return <div> hello world </div>; });
      export const two: Action = action$(() => { return <span> export const v = loader$() </span>; });
      export const the: {} | null = loader$(() => { return null; });
      "#,
      &["loader$", "action$"],
      &["loader$/one", "action$/two", "loader$/the"],
    );
  }

  #[test]
  fn support_esnext() {
    run_test(
      r#"
      class A { static #foo = 233 };
      class B { static { console.log("block"); } }
      await getMethod(undefined ?? "233");
      "#,
      &["loader$", "action$"],
      &[],
    );
  }

  #[test]
  #[should_panic]
  fn invalid_code() {
    run_test(
      r#"
      a7268t612t*^T@&^(!%&^T@^!!R&@^(TR^!@T(*R!TR%)!@&%^(*@&!%(@!)()()())
      "#,
      &["loader$", "action$"],
      &[],
    );
  }
}
