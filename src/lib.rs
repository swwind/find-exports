use std::borrow::Borrow;

use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecmascript::ast::{Callee, Decl, EsVersion, ExportDecl, Expr, Pat};
use swc_ecmascript::parser::lexer::Lexer;
use swc_ecmascript::parser::{Parser, Syntax, TsConfig};
use swc_ecmascript::visit::{Visit, VisitWith};
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

impl Visit for FindExportsVisitor {
  fn visit_export_decl(&mut self, n: &ExportDecl) {
    if let Decl::Var(var_decl) = &n.decl {
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

#[wasm_bindgen]
pub fn find_exports(source: &str, callees: Vec<String>) -> Vec<String> {
  let lexer = Lexer::new(
    Syntax::Typescript(TsConfig {
      tsx: true,
      ..Default::default()
    }),
    EsVersion::Es2022,
    StringInput::new(source, BytePos(0), BytePos(source.as_bytes().len() as u32)),
    None,
  );

  let mut parser = Parser::new_from(lexer);
  let module = parser.parse_module().unwrap();

  let mut visitor = FindExportsVisitor {
    matches: callees,
    exports: Vec::new(),
  };
  module.visit_with(&mut visitor);

  visitor.exports.into_iter().map(|x| x.to_string()).collect()
}
