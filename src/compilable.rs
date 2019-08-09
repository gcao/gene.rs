extern crate ego_tree;

use std::any::Any;
use std::rc::Rc;

use ego_tree::{Tree, NodeRef, NodeMut};
use ordered_float::OrderedFloat;

use super::types::Value;
use super::compiler::{Module, Block};

pub struct Compiler {
    module: Module,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            module: Module::new(),
        }
    }

    pub fn compile(&mut self, value: Value) {
        let mut tree = Tree::new(Compilable::Block);

        self.compile_(&mut tree.root_mut(), &value);

        self.compile_tree(&tree)
    }

    fn compile_(&mut self, parent: &mut NodeMut<Compilable>, value: &Value) {
        match value {
            Value::Null => {
                parent.append(Compilable::Null);
            }
            Value::Boolean(v) => {
                parent.append(Compilable::Bool(*v));
            }
            Value::Integer(v) => {
                parent.append(Compilable::Int(*v));
            }
            Value::String(v) => {
                parent.append(Compilable::String(v.to_string()));
            }
            Value::Symbol(v) => {
                parent.append(Compilable::Symbol(v.to_string()));
            }
            Value::Array(v) => {
                let mut node = parent.append(Compilable::Array);
                for item in v.iter() {
                    self.compile_(&mut node, item);
                }
            }
            Value::Map(v) => {
                let mut node = parent.append(Compilable::Map);
                for (key, value) in v.iter() {
                    let mut node2 = node.append(Compilable::KeyValue(key.to_string()));
                    self.compile_(&mut node2, value);
                }
            }
            Value::Gene(v) => {
                // let mut node = parent.append(Compilable::Gene(GeneType::Other));
                // self.compile_(&mut node, v.kind);
                // for (key, value) in v.props.iter() {
                //     let mut node2 = node.append(Compilable::KeyValue(key.to_string()));
                //     self.compile_(&mut node2, value);
                // }
                // for item in v.data.iter() {
                //     self.compile_(&mut node, item);
                // }
            }
            _ => unimplemented!()
        }
    }

    fn compile_tree(&mut self, tree: &Tree<Compilable>) {
        let mut block = Block::new("__default__".to_string());

        self.module.set_default_block(Rc::new(block));
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
