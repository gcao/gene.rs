extern crate ego_tree;

use std::any::Any;
use std::rc::Rc;
use std::collections::HashMap;

use rand::prelude::random;

use ego_tree::{Tree, NodeRef, NodeMut};
use ordered_float::OrderedFloat;

use super::types::Value;
use super::compiler::{Module, Block, Instruction, LiteralCheck};

pub struct Compiler {
    pub module: Module,
    reg_trackers: HashMap<String, Vec<u16>>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            module: Module::new(),
            reg_trackers: HashMap::new(),
        }
    }

    pub fn compile(&mut self, value: Value) {
        let mut tree = Tree::new(Compilable::new(CompilableData::Block));

        self.translate(&mut tree.root_mut(), &value);

        self.compile_tree(&tree)
    }

    fn translate(&mut self, parent: &mut NodeMut<Compilable>, value: &Value) {
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
                if v.is_literal() {
                    let mut node = parent.append(Compilable::new(CompilableData::Array(v.clone())));
                } else {
                    let mut new_arr = Vec::new();
                    // TODO: add literal values to new_arr
                    let mut node = parent.append(Compilable::new(CompilableData::Array(new_arr)));
                    // TODO: compile non-literal items
                    // for item in v.iter() {
                    //     let mut node2 = parent.append(Compilable::new(CompilableData::ArrayChild(index)));
                    //     self.compile_(&mut node2, item);
                    // }
                }
            }
            Value::Map(v) => {
                if v.is_literal() {
                    let mut node = parent.append(Compilable::new(CompilableData::Map(v.clone())));
                } else {
                    // TODO: create map with literals then compile non-literal values and add to map
                    // let mut node = parent.append(Compilable::new(CompilableData::Map));
                    // for (key, value) in v.iter() {
                    //     let mut node2 = node.append(Compilable::new(CompilableData::MapChild(key.to_string())));
                    //     self.compile_(&mut node2, value);
                    // }
                }
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

        self.compile_node(&tree.root(), &mut block);

        self.module.set_default_block(Rc::new(block));
    }

    fn compile_node(&mut self, node: &NodeRef<Compilable>, block: &mut Block) {
        match &node.value().data {
            CompilableData::Block => {
                for child in node.children() {
                    self.compile_node(&child, block);
                }
            }

            CompilableData::Null => {
                let parent = node.parent().unwrap();
                let parent_value = parent.value();
                match parent_value.data {
                    CompilableData::ArrayChild(index) => {
                        let gp = parent.parent().unwrap();
                        let gp_value = gp.value();
                        let reg = gp_value.options["reg"].downcast_ref::<u16>().unwrap();
                        block.add_instr(Instruction::Default(Value::Null));
                        block.add_instr(Instruction::SetItem(*reg, index));
                    }
                    CompilableData::Block => {
                        if node.next_sibling().is_none() {
                            // is last in the block
                            block.add_instr(Instruction::Default(Value::Null));
                        } else {
                            // No need to generate any instruction for dead code
                        }
                    }
                    _ => unimplemented!()
                }
            }
            CompilableData::Int(v) => {
                let parent = node.parent().unwrap();
                let parent_value = parent.value();
                match parent_value.data {
                    CompilableData::ArrayChild(index) => {
                        let gp = parent.parent().unwrap();
                        let gp_value = gp.value();
                        let reg = gp_value.options["reg"].downcast_ref::<u16>().unwrap();
                        block.add_instr(Instruction::Default(Value::Integer(v.clone())));
                        block.add_instr(Instruction::SetItem(*reg, index));
                    }
                    CompilableData::Block => {
                        if node.next_sibling().is_none() {
                            // is last in the block
                            block.add_instr(Instruction::Default(Value::Integer(v.clone())));
                        } else {
                            // No need to generate any instruction for dead code
                        }
                    }
                    _ => unimplemented!()
                }
            }
            CompilableData::Array(v) => {
                (*block).add_instr(Instruction::Default(Value::Array(v.clone())));
                // for child in node.children() {
                // }
            }
            CompilableData::Map(v) => {
                (*block).add_instr(Instruction::Default(Value::Map(v.clone())));
                // for child in node.children() {
                // }
            }
            _ => unimplemented!()
        }
    }

    /// 1. find and return available register
    /// 2. if all registers are occupied
    fn get_reg(&mut self, block: &mut Block) -> u16 {
        let trackers = self.reg_trackers.get_mut(&block.id).unwrap();
        for i in 2..16 {
            let mut available = true;
            for tracker in trackers.iter() {
                if *tracker == i as u16 {
                    available = false;
                }
            }
            if available {
                trackers.push(i);
                return i as u16;
            }
        }
        16 + random::<u16>()
    }

    fn free_reg(&mut self, block: &mut Block, reg: u16) {
        if reg < 16 {
            let trackers = self.reg_trackers.get_mut(&block.id).unwrap();
            trackers.retain(|&tracker| tracker != reg)
        }
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
    Block,
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
    ArrayChild(usize),
    Map(HashMap<String, Value>), // literal values are included
    MapChild(String),
    Gene(GeneKind, HashMap<String, Value>, Vec<Value>), // literal values are included
    GeneKind, // the gene kind may have to be compiled, this is the indicator/parent for it
    GeneProp(String),
    GeneDataChild(usize),
}

pub enum GeneKind {
    Var,
    If,
    Function,
    Invocation,
}
