extern crate ego_tree;

use std::any::Any;
use std::rc::Rc;

use ego_tree::{Tree, NodeRef, NodeMut};
use ordered_float::OrderedFloat;

use super::types::Value;
use super::compiler::{Module, Block};

pub struct Compiler {
  // module: Module,
}

impl Compiler {
  pub fn new() -> Self {
    Compiler {
      // module: Module::new(),
    }
  }

  pub fn compile(&mut self, value: Value) {
    self.compile_(None, value);
  }

  fn compile_(&mut self, node: Option<&NodeMut<Compilable>>, value: Value) {
    match value {
      Value::Null => {
        //
      }
      _ => unimplemented!()
    }
    if (node.is_none()) {
    } else {
    }
  }
}

pub enum Compilable {
  /// literal
  Void,
  /// literal
  Null,
  /// literal
  Bool(bool),
  /// literal
  Int(i64),
  /// literal
  Float(OrderedFloat<f64>),
  /// literal
  String(String),
  Symbol(String),
  Array,
  Map,
  KeyValue(String),
  Gene(GeneType),
  Block,
}

impl Compilable {
  pub fn set(&mut self, name: String, value: Box<dyn Any>) {
    match self {
      _ => unimplemented!()
    }
  }
}

pub enum GeneType {
  Function,
  If,
  Other,
}

struct CompilableNode<'a>(NodeRef<'a, Compilable>);

impl<'a> CompilableNode<'a> {
  pub fn new(node: NodeRef<'a, Compilable>) -> CompilableNode {
    CompilableNode(node)
  }
}