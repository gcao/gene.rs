extern crate ego_tree;

use std::any::Any;
use std::rc::Rc;
use std::collections::HashMap;
use std::mem;

use rand::prelude::random;

use ego_tree::{Tree, NodeRef, NodeMut};
use ordered_float::OrderedFloat;

use super::types::{Value, Gene};
use super::compiler::{Module, Block, Instruction, LiteralCheck, is_binary_op};

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

        match value {
            Value::Stream(v) => {
                for item in v {
                    self.translate(&mut tree.root_mut(), &item);
                }
            }
            _ => {
                self.translate(&mut tree.root_mut(), &value);
            }
        }

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
                    parent.append(Compilable::new(CompilableData::Array(v.clone())));
                } else {
                    let mut new_arr = Vec::new();
                    // add literal values to new_arr
                    for (i, item) in v.iter().enumerate() {
                        if item.is_literal() {
                            new_arr.insert(i, item.clone());
                        } else {
                            new_arr.insert(i, Value::Void);
                            // TODO: compile non-literal items
                        }
                    }
                    let mut node = parent.append(Compilable::new(CompilableData::Array(new_arr)));
                    // compile non-literal items
                    for (i, item) in v.iter().enumerate() {
                        if !item.is_literal() {
                            let mut node2 = node.append(Compilable::new(CompilableData::ArrayChild(i)));
                            self.translate(&mut node2, item);
                        }
                    }
                }
            }
            Value::Map(v) => {
                if v.is_literal() {
                    parent.append(Compilable::new(CompilableData::Map(v.clone())));
                } else {
                    // TODO: create map with literals then compile non-literal values and add to map
                    // let mut node = parent.append(Compilable::new(CompilableData::Map));
                    // for (key, value) in v.iter() {
                    //     let mut node2 = node.append(Compilable::new(CompilableData::MapChild(key.to_string())));
                    //     self.compile_(&mut node2, value);
                    // }
                }
            }
            Value::Gene(box v) => {
                let Gene{ kind, props, data } = v.normalize();
                match kind {
                    Value::Symbol(ref s) if is_binary_op(s) => {
                        let mut node = parent.append(Compilable::new(CompilableData::BinaryOp(s.clone())));
                        self.translate(&mut node, &data[0]);
                        self.translate(&mut node, &data[1]);
                    }
                    Value::Symbol(ref s) if s == "=" => {
                        if let Value::Symbol(name) = &data[0] {
                            let mut node = parent.append(Compilable::new(CompilableData::Assignment(name.clone())));
                            self.translate(&mut node, &data[1]);
                        } else {
                            unimplemented!();
                        }
                    }
                    Value::Symbol(ref s) if s == "var" => {
                        if let Value::Symbol(name) = &data[0] {
                            let mut node = parent.append(Compilable::new(CompilableData::Var(name.clone())));
                            let value = data[1].clone();
                            self.translate(&mut node, &value);
                        }
                    }
                    Value::Symbol(ref s) if s == "if" => {
                        let cond = &data[0];
                        let mut then_stmts = Vec::new();
                        let mut else_stmts = Vec::new();
                        let mut is_else = false;
                        for (i, item) in data.iter().enumerate() {
                            if i == 0 {
                                continue;
                            }

                            if is_else {
                                else_stmts.push(item.clone());
                            } else {
                                match item {
                                    Value::Symbol(ref s) if s == "then" => (),
                                    Value::Symbol(ref s) if s == "else" => {
                                        is_else = true;
                                    }
                                    _ => {
                                        then_stmts.push(item.clone());
                                    }
                                }
                            }
                        }
                        let mut if_node = parent.append(Compilable::new(CompilableData::If));
                        {
                            let mut if_pair = if_node.append(Compilable::new(CompilableData::IfPair));
                            let mut if_cond = if_pair.append(Compilable::new(CompilableData::IfPairCondition));
                            self.translate(&mut if_cond, cond);
                            let mut if_then = if_pair.append(Compilable::new(CompilableData::IfPairThen));
                            let mut if_then_stmts = if_then.append(Compilable::new(CompilableData::Statements));
                            for stmt in then_stmts {
                                self.translate(&mut if_then_stmts, &stmt);
                            }
                        }
                        if else_stmts.len() > 0 {
                            let mut if_else = if_node.append(Compilable::new(CompilableData::IfElse));
                            let mut if_else_stmts = if_else.append(Compilable::new(CompilableData::Statements));
                            for stmt in else_stmts {
                                self.translate(&mut if_else_stmts, &stmt);
                            }
                        }
                    }
                    _ => unimplemented!()
                }
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

        self.reg_trackers.insert(block.id.clone(), Vec::new());

        self.compile_node(&tree.root(), &mut block);

        self.module.set_default_block(Rc::new(block));
    }

    fn compile_node(&mut self, node: &NodeRef<Compilable>, block: &mut Block) {
        dbg!(node.value().data.clone());
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
            CompilableData::Bool(v) => {
                block.add_instr(Instruction::Default(Value::Boolean(v.clone())));
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
                    _ => {
                        block.add_instr(Instruction::Default(Value::Integer(v.clone())));
                    }
                }
            }
            CompilableData::String(v) => {
                let parent = node.parent().unwrap();
                let parent_value = parent.value();
                match parent_value.data {
                    CompilableData::ArrayChild(index) => {
                        let gp = parent.parent().unwrap();
                        let gp_value = gp.value();
                        let reg = gp_value.options["reg"].downcast_ref::<u16>().unwrap();
                        block.add_instr(Instruction::Default(Value::String(v.clone())));
                        block.add_instr(Instruction::SetItem(*reg, index));
                    }
                    CompilableData::Block => {
                        if node.next_sibling().is_none() {
                            // is last in the block
                            block.add_instr(Instruction::Default(Value::String(v.clone())));
                        } else {
                            // No need to generate any instruction for dead code
                        }
                    }
                    _ => {
                        block.add_instr(Instruction::Default(Value::String(v.clone())));
                    }
                }
            }
            CompilableData::Symbol(s) => {
                (*block).add_instr(Instruction::GetMember(s.to_string()));
            }
            CompilableData::Array(v) => {
                let reg = self.get_reg(block);
                (*block).add_instr(Instruction::Save(reg, Value::Array(v.clone())));
                for child in node.children() {
                    match child.value().data {
                        CompilableData::ArrayChild(i) => {
                            self.compile_node(&child.first_child().unwrap(), block);
                            (*block).add_instr(Instruction::SetItem(reg, i));
                        }
                        _ => unimplemented!()
                    }
                }
                (*block).add_instr(Instruction::CopyToDefault(reg));
            }
            CompilableData::Map(v) => {
                (*block).add_instr(Instruction::Default(Value::Map(v.clone())));
                // for child in node.children() {
                // }
            }
            CompilableData::Var(name) => {
                self.compile_node(&node.first_child().unwrap(), block);
                (*block).add_instr(Instruction::DefMember(name.clone()));
            }
            CompilableData::BinaryOp(op) => {
                let first = node.first_child().unwrap();
                self.compile_node(&first, block);
                let first_reg = self.get_reg(block);
                (*block).add_instr(Instruction::CopyFromDefault(first_reg));

                let second = first.next_sibling().unwrap();
                self.compile_node(&second, block);

                (*block).add_instr(Instruction::BinaryOp(op.clone(), first_reg));
            }
            CompilableData::Assignment(name) => {
                self.compile_node(&node.first_child().unwrap(), block);
                (*block).add_instr(Instruction::SetMember(name.clone()));
            }
            CompilableData::If => {
                let start_pos = block.len();
                let pair_node = node.first_child().unwrap();
                self.compile_node(&pair_node, block);

                let else_pos = block.len();
                if let Some(else_node) = pair_node.next_sibling() {
                    self.compile_node(&else_node, block);
                }

                let end_pos = block.len();

                for i in start_pos..end_pos {
                    let instr = &block.instructions[i];
                    match instr {
                        Instruction::JumpToElse => {
                            mem::replace(&mut (*block).instructions[i], Instruction::JumpIfFalse(else_pos as i16));
                        }
                        Instruction::JumpToNextStatement => {
                            mem::replace(&mut (*block).instructions[i], Instruction::Jump(end_pos as i16));
                        }
                        _ => ()
                    }
                }
            }
            CompilableData::IfPair => {
                let cond_node = node.first_child().unwrap();
                self.compile_node(&cond_node, block);
                (*block).add_instr(Instruction::JumpToElse);

                let then_node = cond_node.next_sibling().unwrap();
                self.compile_node(&then_node, block);
                (*block).add_instr(Instruction::JumpToNextStatement);
            }
            CompilableData::IfPairCondition | CompilableData::IfPairThen | CompilableData::IfElse => {
                let cond_node = node.first_child().unwrap();
                self.compile_node(&cond_node, block);
            }
            CompilableData::Statements => {
                for node in node.children() {
                    self.compile_node(&node, block);
                }
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

pub struct NodeWrapper<'a>(&'a mut NodeRef<'a, Compilable>);

impl<'a> NodeWrapper<'a> {
    pub fn use_member(&mut self, name: &str) -> bool {
        true
    }

    // /// @return Start position in the compiled block
    // /// Same as count of all previous code's generated instruction
    // pub fn start_pos(&mut self) -> usize {
    //     if let Some(mut prev) = self.0.prev_sibling() {
    //         let mut wrapper = NodeWrapper(&mut prev);
    //         wrapper.start_pos() + wrapper.instr_count() 
    //     } else if let Some(mut parent) = self.0.parent() {
    //         NodeWrapper(&mut parent).start_pos()
    //     } else {
    //         0
    //     }
    // }

    // pub fn end_pos(&mut self) -> usize {
    //     self.start_pos() + self.instr_count()
    // }

    // pub fn instr_count(&self) -> usize {
    //     0
    // }
}

pub struct Compilable {
    pub data: CompilableData,
    pub options: HashMap<String, Box<dyn Any>>,
    // pub start_pos: Option<usize>,
    // pub instr_count: Option<usize>,
}

impl Compilable {
    pub fn new(data: CompilableData) -> Self {
        Compilable {
            data,
            options: HashMap:: new(),
            // start_pos: None,
            // instr_count: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CompilableData {
    Block,
    Statements,
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
    Var(String),
    BinaryOp(String),
    Assignment(String),
    If,
    IfPair,
    IfPairCondition,
    IfPairThen,
    IfElse,
}

#[derive(Clone, Debug)]
pub enum GeneKind {
    Var,
    If,
    Function,
    Invocation,
}

trait Normalize {
    fn normalize(&self) -> Gene;
}

impl Normalize for Gene {
    fn normalize(&self) -> Gene {
        match self.data[0] {
            Value::Symbol(ref s) if is_binary_op(s) || s == "=" => {
                let kind = self.data[0].clone();
                let mut data = vec![self.kind.clone()];
                for (i, item) in self.data.iter().enumerate() {
                    if i > 0 {
                        data.push(item.clone());
                    }
                }
                Gene {
                    kind,
                    props: self.props.clone(),
                    data,
                }
            }
            _ => {
                Gene{
                    kind:  self.kind.clone(),
                    props: self.props.clone(),
                    data:  self.data.clone(),
                }
            }
        }
    }
}
