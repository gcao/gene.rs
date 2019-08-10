extern crate ego_tree;

use std::any::Any;
use std::rc::Rc;
use std::collections::{BTreeMap, HashMap};

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
        let mut tree = Tree::new(Compilable::new(CompilableData::Block));

        self.compile_(&mut tree.root_mut(), &value);

        self.compile_tree(&tree)
    }

    fn compile_(&mut self, parent: &mut NodeMut<Compilable>, value: &Value) {
        match value {
            Value::Null => {
                parent.append(Compilable::new(CompilableData::Null));
            }
            Value::Boolean(v) => {
                parent.append(Compilable::new(CompilableData::Bool(*v)));
            }
            Value::Integer(v) => {
                parent.append(Compilable::new(CompilableData::Int(*v)));
            }
            Value::String(v) => {
                parent.append(Compilable::new(CompilableData::String(v.to_string())));
            }
            Value::Symbol(v) => {
                parent.append(Compilable::new(CompilableData::Symbol(v.to_string())));
            }
            Value::Array(v) => {
                let mut new_arr = Vec::new();
                // TODO: add literal values to new_arr
                let mut node = parent.append(Compilable::new(CompilableData::Array(new_arr)));
                // TODO: compile non-literal items
                // for item in v.iter() {
                //     let mut node2 = parent.append(Compilable::new(CompilableData::ArrayChild(index)));
                //     self.compile_(&mut node2, item);
                // }
            }
            Value::Map(v) => {
                // TODO: create map with literals then compile non-literal values and add to map
                // let mut node = parent.append(Compilable::new(CompilableData::Map));
                // for (key, value) in v.iter() {
                //     let mut node2 = node.append(Compilable::new(CompilableData::MapChild(key.to_string())));
                //     self.compile_(&mut node2, value);
                // }
            }
            Value::Gene(v) => {
                // TODO: create Gene with literals then compile non-literal kind/prop/data
                // let mut node = parent.append(Compilable::new(CompilableData::Gene));
                // let mut kind_node = node.append(Compilable::new(CompilableData::GeneKind(GeneKind::Other)));
                // self.compile_(&mut kind_node, v.kind);
                // for (key, value) in v.props.iter() {
                //     let mut node2 = node.append(Compilable::new(CompilableData::MapChild(key.to_string())));
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

pub struct Compilable {
    data: CompilableData,
    options: HashMap<String, Box<dyn Any>>,
}

impl Compilable {
    pub fn new(data: CompilableData) -> Self {
        Compilable {
            data,
            options: HashMap:: new(),
        }
    }
}

pub enum CompilableData {
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
    Array(Vec<Value>), // literal values are included
    ArrayChild(u32),
    Map(HashMap<String, Value>), // literal values are included
    MapChild(String),
    Gene(GeneKind, HashMap<String, Value>, Vec<Value>), // literal values are included
    GeneKind, // the gene kind may have to be compiled, this is the indicator/parent for it
    GeneProp(String),
    GeneDataChild(u32),
    Block,
}

pub enum GeneKind {
    Var,
    If,
    Function,
    Invocation,
}
