extern crate rand;
extern crate ego_tree;

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::mem;

use rand::prelude::random;

use ego_tree::{Tree, NodeRef, NodeMut};
use ordered_float::OrderedFloat;

use super::types::Value;
use super::types::Gene;
use super::vm::types::Function;
use super::vm::types::Matcher;

use super::utils::new_uuidv4;

pub trait LiteralCheck {
    fn is_literal(&self) -> bool;
}

impl LiteralCheck for Value {
    fn is_literal(&self) -> bool {
        match self {
            Value::Symbol(_s) => false,
            Value::Array(v) => v.is_literal(),
            Value::Gene(g) => g.is_literal(),
            _ => true
        }
    }
}

impl LiteralCheck for Vec<Value> {
    fn is_literal(&self) -> bool {
        self.iter().all(LiteralCheck::is_literal)
    }
}

impl<S: ::std::hash::BuildHasher> LiteralCheck for HashMap<String, Value, S> {
    fn is_literal(&self) -> bool {
        self.values().all(LiteralCheck::is_literal)
    }
}
impl LiteralCheck for Gene {
    fn is_literal(&self) -> bool {
        false
    }
}

pub struct Statements(Vec<Value>);

#[derive(Debug)]
pub struct Module {
    pub id: String,
    pub blocks: HashMap<String, Rc<Block>>,
    default_block_id: String,
}

impl Module {
    pub fn new() -> Self {
        Module {
            id: new_uuidv4(),
            blocks: HashMap::new(),
            default_block_id: "".to_string(),
        }
    }

    pub fn set_default_block(&mut self, block: Block) {
        self.default_block_id = block.id.clone();
        self.blocks.insert(block.id.clone(), Rc::new(block));
    }

    pub fn get_default_block(&self) -> Rc<Block> {
        let block = &self.blocks[&self.default_block_id];
        block.clone()
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.insert(block.id.clone(), Rc::new(block));
    }
}

#[derive(Debug)]
pub struct Block {
    pub id: String,
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub registers_in_use: HashSet<u16>,
    pub name_managers: HashMap<String, NameManager>,
}

impl Block {
    pub fn new(name: String) -> Self {
        Block {
            id: new_uuidv4(),
            name,
            instructions: Vec::new(),
            registers_in_use: HashSet::new(),
            name_managers: HashMap::new(),
        }
    }

    pub fn add_instr(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_reg(&mut self) -> u16 {
        let mut i: u16 = 0;
        while self.registers_in_use.contains(&i) {
            i += 1;
        }
        self.registers_in_use.insert(i);
        i
    }

    pub fn free_reg(&mut self, reg: &u16) {
        self.registers_in_use.remove(reg);
    }

    pub fn add_name(&mut self, name: &str, total_usage: usize) {
        let mut name_manager = NameManager::new(name);
        name_manager.total_usage = total_usage;
        self.name_managers.insert(name.to_string(), name_manager);
    }

    pub fn use_name(&mut self, name: &str) {
        let mut name_manager = self.name_managers.get_mut(name).unwrap();
        name_manager.usage += 1;
    }

    pub fn get_name_manager(&self, name: &str) -> &NameManager {
        &self.name_managers[name]
    }

    /// Assign a register for member with <name> if not already assigned and return it.
    pub fn get_reg_for(&mut self, name: &str) -> u16 {
        if let Some(register) = self.get_name_manager(name).register {
            register
        } else {
            let register = self.get_reg();
            let mut name_manager = self.name_managers.get_mut(name).unwrap();
            name_manager.register = Some(register);
            register
        }
    }

    /// return (false, reg) if last instruction is CopyFromDefault or CopyToDefault
    /// else (true, reg)
    pub fn save_default_to_reg(&mut self) -> (bool, u16) {
        if let Some(instr) = self.instructions.last() {
            match instr {
                Instruction::CopyFromDefault(reg) => {
                    return (false, *reg);
                }
                Instruction::CopyToDefault(reg) => {
                    return (false, *reg);
                }
                _ => {}
            }
        }
        let reg = self.get_reg();
        self.add_instr(Instruction::CopyFromDefault(reg));
        (true, reg)
    }

    pub fn set_default(&mut self, value: Value) {
        if self.len() > 0 {
            let last_instr = &self.instructions[self.len() - 1];
            if let Instruction::CopyToDefault(_) = last_instr {
                self.instructions.remove(self.len() - 1);
            }
        }
        self.instructions.push(Instruction::Default(value));
    }
}

impl fmt::Display for Block {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("(Block ")?;
        fmt.write_str(&self.name)?;
        fmt.write_str("\n")?;
        for (i, instr) in self.instructions.iter().enumerate() {
            fmt.write_str(&*format!("{: >5} ", i))?;
            fmt.write_str(&instr.to_string())?;
            fmt.write_str("\n")?;
        }
        fmt.write_str(")")?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum Instruction {
    Dummy,
    Init,

    /// Save Value to default register
    Default(Value),
    /// Save Value to named register
    Save(u16, Value),
    CopyFromDefault(u16),
    CopyToDefault(u16),

    /// index in scope, register
    ScopeDefMemberByIndex(usize, usize),
    /// name, register
    ScopeDefMemberByName(String, usize),
    ScopeSetMemberByIndex(usize, usize),
    ScopeSetMemberByName(String, usize),
    ScopeGetMemberByIndex(usize),
    ScopeGetMemberByName(String),

    DefMember(String),
    DefMemberInScope(String),
    DefMemberInNS(String),
    GetMember(String),
    GetMemberInScope(String),
    GetMemberInNS(String),
    SetMember(String),
    SetMemberInScope(String),
    SetMemberInNS(String),

    /// GetItem(target reg, index)
    GetItem(u16, usize),
    // /// GetItemDynamic(target reg, index reg)
    // GetItemDynamic(String, String),
    /// SetItem(target reg, index, value reg)
    SetItem(u16, usize),
    // /// SetItemDynamic(target reg, index reg, value reg)
    // SetItemDynamic(String, String, String),

    // /// GetProp(target reg, name)
    // GetProp(String, String),
    // /// GetPropDynamic(target reg, name reg)
    // GetPropDynamic(String, String),
    /// SetProp(target reg, name, value reg)
    SetProp(u16, String),
    // /// SetPropDynamic(target reg, name reg, value reg)
    // SetPropDynamic(String, String, String),

    Jump(i16),
    JumpIfFalse(i16),
    /// Below are pseudo instructions that should be replaced with other jump instructions
    /// before sent to the VM to execute.
    JumpToElse,
    JumpToNextStatement,

    Break,
    LoopStart,
    LoopEnd,

    /// reg + default
    Add(u16),
    /// reg - default
    Sub(u16),
    Eq(u16),
    Lt(u16),
    Gt(u16),

    /// Function(name, args reg, block id)
    Function(String, Matcher, String),
    /// Create an argument object and store in a register
    CreateArguments(u16),

    /// Call(options)
    Call(u16, Option<u16>, HashMap<String, Rc<dyn Any>>),
    CallEnd,
}

impl fmt::Display for Instruction {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("(")?;
        match &self {
            Instruction::Dummy => {
                fmt.write_str("Dummy           Dummy")?;
            }
            Instruction::Init => {
                fmt.write_str("Init            Init")?;
            }
            Instruction::Default(v) => {
                fmt.write_str("Default         Default = ")?;
                fmt.write_str(&v.to_string())?;
            }
            Instruction::Save(reg, v) => {
                fmt.write_str("Save            R")?;
                fmt.write_str(&reg.to_string())?;
                fmt.write_str(" = ")?;
                fmt.write_str(&v.to_string())?;
            }
            Instruction::CopyFromDefault(reg) => {
                fmt.write_str("CopyFromDefault R")?;
                fmt.write_str(&reg.to_string())?;
                fmt.write_str(" = Default")?;
            }
            Instruction::CopyToDefault(reg) => {
                fmt.write_str("CopyToDefault   Default = R")?;
                fmt.write_str(&reg.to_string())?;
            }
            Instruction::DefMember(name) => {
                fmt.write_str("DefMember       <")?;
                fmt.write_str(name)?;
                fmt.write_str("> = Default")?;
            }
            Instruction::DefMemberInScope(name) => {
                fmt.write_str("DefMemberInScope <")?;
                fmt.write_str(name)?;
                fmt.write_str("> = Default")?;
            }
            Instruction::DefMemberInNS(name) => {
                fmt.write_str("DefMemberInNS   <")?;
                fmt.write_str(name)?;
                fmt.write_str("> = Default")?;
            }
            Instruction::GetMember(name) => {
                fmt.write_str("GetMember       Default = <")?;
                fmt.write_str(name)?;
                fmt.write_str(">")?;
            }
            Instruction::GetMemberInScope(name) => {
                fmt.write_str("GetMemberInScope Default = <")?;
                fmt.write_str(name)?;
                fmt.write_str(">")?;
            }
            Instruction::GetMemberInNS(name) => {
                fmt.write_str("GetMemberInNS   Default = <")?;
                fmt.write_str(name)?;
                fmt.write_str(">")?;
            }
            Instruction::SetMember(name) => {
                fmt.write_str("SetMember       <")?;
                fmt.write_str(name)?;
                fmt.write_str("> = Default")?;
            }
            Instruction::SetMemberInScope(name) => {
                fmt.write_str("SetMemberInScope <")?;
                fmt.write_str(name)?;
                fmt.write_str("> = Default")?;
            }
            Instruction::SetMemberInNS(name) => {
                fmt.write_str("SetMemberInNS   <")?;
                fmt.write_str(name)?;
                fmt.write_str("> = Default")?;
            }
            Instruction::GetItem(reg, index) => {
                fmt.write_str("GetItem         Default = R")?;
                fmt.write_str(&reg.to_string())?;
                fmt.write_str("[")?;
                fmt.write_str(&index.to_string())?;
                fmt.write_str("]")?;
            }
            Instruction::SetItem(reg, index) => {
                fmt.write_str("SetItem         R")?;
                fmt.write_str(&reg.to_string())?;
                fmt.write_str("[")?;
                fmt.write_str(&index.to_string())?;
                fmt.write_str("] = Default")?;
            }
            Instruction::SetProp(reg, key) => {
                fmt.write_str("SetProp         R")?;
                fmt.write_str(&reg.to_string())?;
                fmt.write_str("[")?;
                fmt.write_str(key)?;
                fmt.write_str("] = Default")?;
            }
            Instruction::Jump(pos) => {
                fmt.write_str("Jump            Jump to ")?;
                fmt.write_str(&pos.to_string())?;
            }
            Instruction::JumpIfFalse(pos) => {
                fmt.write_str("JumpIfFalse     Jump to ")?;
                fmt.write_str(&pos.to_string())?;
                fmt.write_str(" if Default is falsy")?;
            }
            Instruction::Break => {
                fmt.write_str("Break           Break")?;
            }
            Instruction::LoopStart => {
                fmt.write_str("LoopStart       LoopStart")?;
            }
            Instruction::LoopEnd => {
                fmt.write_str("LoopEnd         LoopEnd")?;
            }
            Instruction::Add(first) => {
                fmt.write_str("Add             Default = R")?;
                fmt.write_str(&first.to_string())?;
                fmt.write_str(" + Default")?;
            }
            Instruction::Sub(first) => {
                fmt.write_str("Sub             Default = R")?;
                fmt.write_str(&first.to_string())?;
                fmt.write_str(" - Default")?;
            }
            Instruction::Eq(first) => {
                fmt.write_str("Lt              Default = R")?;
                fmt.write_str(&first.to_string())?;
                fmt.write_str(" == Default")?;
            }
            Instruction::Lt(first) => {
                fmt.write_str("Lt              Default = R")?;
                fmt.write_str(&first.to_string())?;
                fmt.write_str(" < Default")?;
            }
            Instruction::Gt(first) => {
                fmt.write_str("Gt              Default = R")?;
                fmt.write_str(&first.to_string())?;
                fmt.write_str(" > Default")?;
            }
            Instruction::Function(name, _matcher, body_id) => {
                fmt.write_str("Function        Default = Function <")?;
                fmt.write_str(name)?;
                fmt.write_str("> ")?;
                // TODO: matcher to string
                // fmt.write_str(&matcher)?;
                // fmt.write_str(" ")?;
                fmt.write_str(body_id)?;
            }
            Instruction::Call(target_reg, args_reg, _options) => {
                fmt.write_str("Call            Default = Call R")?;
                fmt.write_str(&target_reg.to_string())?;
                fmt.write_str(" R")?;
                if let Some(reg) = args_reg {
                    fmt.write_str(&reg.to_string())?;
                }
            }
            Instruction::CallEnd => {
                fmt.write_str("CallEnd         CallEnd")?;
            }
            Instruction::CreateArguments(reg) => {
                fmt.write_str("CreateArguments R")?;
                fmt.write_str(&reg.to_string())?;
                fmt.write_str(" = CreateArguments")?;
            }
            _ => {
                fmt.write_str("???             ???")?;
            }
        }
        fmt.write_str(")")?;
        Ok(())
    }
}

pub fn is_binary_op(op: &str) -> bool {
    let binary_ops = vec!["+", "-", "*", "/", "<", "<=", ">", ">=", "=="];
    binary_ops.contains(&op)
}

// fn normalize(gene: Gene) -> Gene {
//     if gene.data.is_empty() {
//         gene
//     } else {
//         let first = gene.data[0].clone();
//         match first {
//             Value::Symbol(ref s) if is_binary_op(s) || s == "=" => {
//                 let Gene {
//                     kind,
//                     props,
//                     mut data,
//                 } = gene;
//                 let new_kind = data.remove(0);
//                 data.insert(0, kind.clone());
//                 Gene {
//                     kind: new_kind,
//                     props,
//                     data,
//                 }
//             }
//             _ => gene,
//         }
//     }
// }

#[derive(Debug)]
pub struct NameManager {
    pub name: String,
    pub total_usage: usize,
    pub usage: usize,
    pub register: Option<u16>,
}

impl NameManager {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            total_usage: 0,
            usage: 0,
            register: None,
        }
    }

    pub fn used_first_time(&self) -> bool {
        self.usage == 0
    }
}

pub struct Compiler {
    pub module: Module,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            module: Module::new(),
        }
    }

    pub fn compile(&mut self, value: Value) {
        let mut tree = Tree::new(Compilable::new(CompilableData::Block));
        self.translate(&mut tree.root_mut(), &value);
        let block = self.compile_tree(&tree, "__default__".to_string(), true);
        self.module.set_default_block(block);
    }

    fn translate(&mut self, parent: &mut NodeMut<Compilable>, value: &Value) {
        match value {
            Value::Stream(v) => {
                for item in v {
                    self.translate(parent, &item);
                }
            }
            Value::Null => {
                parent.append(Compilable::new(CompilableData::Null));
            }
            Value::Boolean(v) => {
                parent.append(Compilable::new(CompilableData::Bool(*v)));
            }
            Value::Integer(v) => {
                parent.append(Compilable::new(CompilableData::Int(*v)));
            }
            Value::Float(v) => {
                parent.append(Compilable::new(CompilableData::Float(*v)));
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
                    let mut map_node = parent.append(Compilable::new(CompilableData::Map(HashMap::new())));
                    for (key, value) in v.iter() {
                        let mut key_node = map_node.append(Compilable::new(CompilableData::MapChild(key.to_string())));
                        self.translate(&mut key_node, value);
                    }
                }
            }
            Value::Gene(box v) => {
                let Gene{ kind, data, .. } = v.normalize();
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
                            let mut is_top_level = false;
                            if let CompilableData::Block = parent.value().data {
                                is_top_level = true;
                            }
                            let mut node = parent.append(Compilable::new(CompilableData::Var(name.clone(), is_top_level)));
                            let value = data[1].clone();
                            self.translate(&mut node, &value);
                        }
                    }
                    Value::Symbol(ref s) if s == "fn" => {
                        let name = data[0].to_string();

                        let borrowed = data[1].clone();
                        let matcher =  Matcher::from(&borrowed);

                        let mut tree = Tree::new(Compilable::new(CompilableData::Block));
                        let mut stmts = Vec::new();
                        for item in data.iter().skip(2) {
                            stmts.push(item.clone());
                        }
                        self.translate(&mut tree.root_mut(), &Value::Stream(stmts));
                        let body = self.compile_tree(&tree, name.clone(), false);
                        let body_id = body.id.clone();
                        self.module.add_block(body);

                        parent.append(Compilable::new(CompilableData::Function(name, matcher, body_id)));
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
                        if !else_stmts.is_empty() {
                            let mut if_else = if_node.append(Compilable::new(CompilableData::IfElse));
                            let mut if_else_stmts = if_else.append(Compilable::new(CompilableData::Statements));
                            for stmt in else_stmts {
                                self.translate(&mut if_else_stmts, &stmt);
                            }
                        }
                    }
                    Value::Symbol(ref s) if s == "loop" => {
                        let mut node = parent.append(Compilable::new(CompilableData::Loop));
                        for stmt in data {
                            self.translate(&mut node, &stmt);
                        }
                    }
                    Value::Symbol(ref s) if s == "break" => {
                        parent.append(Compilable::new(CompilableData::Break));
                    }
                    Value::Symbol(ref s) if s == "while" => {
                        let mut node = parent.append(Compilable::new(CompilableData::While));
                        for stmt in data {
                            self.translate(&mut node, &stmt);
                        }
                    }
                    Value::Symbol(s) => {
                        let mut node = parent.append(Compilable::new(CompilableData::Invocation));
                        node.append(Compilable::new(CompilableData::Symbol(s.to_string())));

                        // if data.len() == 0 {
                        //     // TODO: optimization
                        //     node.append(Compilable::new(CompilableData::InvocationArguments(data.clone())));
                        // } else if data.is_literal() {
                        //     node.append(Compilable::new(CompilableData::InvocationArguments(data.clone())));
                        // } else {
                            let mut new_arr = Vec::new();
                            // add literal values to new_arr
                            for (i, item) in data.iter().enumerate() {
                                if item.is_literal() {
                                    new_arr.insert(i, item.clone());
                                } else {
                                    new_arr.insert(i, Value::Void);
                                }
                            }
                            let mut node = node.append(Compilable::new(CompilableData::InvocationArguments(new_arr)));
                            // compile non-literal items
                            for (i, item) in data.iter().enumerate() {
                                // if !item.is_literal() {
                                    let mut node2 = node.append(Compilable::new(CompilableData::ArrayChild(i)));
                                    self.translate(&mut node2, item);
                                // }
                            }
                        // }
                    }
                    _ => {
                        let mut node = parent.append(Compilable::new(CompilableData::Invocation));
                        self.translate(&mut node, &kind);

                        // if data.len() == 0 {
                        //     // TODO: optimization
                        //     node.append(Compilable::new(CompilableData::InvocationArguments(data.clone())));
                        // } else if data.is_literal() {
                        //     node.append(Compilable::new(CompilableData::InvocationArguments(data.clone())));
                        // } else {
                            let mut new_arr = Vec::new();
                            // add literal values to new_arr
                            for (i, item) in data.iter().enumerate() {
                                if item.is_literal() {
                                    new_arr.insert(i, item.clone());
                                } else {
                                    new_arr.insert(i, Value::Void);
                                }
                            }
                            let mut node = node.append(Compilable::new(CompilableData::InvocationArguments(new_arr)));
                            // compile non-literal items
                            for (i, item) in data.iter().enumerate() {
                                // if !item.is_literal() {
                                    let mut node2 = node.append(Compilable::new(CompilableData::ArrayChild(i)));
                                    self.translate(&mut node2, item);
                                // }
                            }
                        // }
                    }
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

    fn compile_tree(&mut self, tree: &Tree<Compilable>, name: String, is_default: bool) -> Block {
        let mut block = Block::new(name);

        let name_usage = NodeWrapper(&tree.root()).get_name_usage();
        for (name, value) in name_usage {
            block.add_name(&name, value);
        }

        if is_default {
            block.add_instr(Instruction::Init);
        }

        self.compile_node(&tree.root(), &mut block);
        println!("{}", block);

        block
    }

    fn compile_node(&mut self, node: &NodeRef<Compilable>, block: &mut Block) {
        // dbg!(node.value().data.clone());
        match &node.value().data {
            CompilableData::Block => {
                for child in node.children() {
                    self.compile_node(&child, block);
                }
                block.add_instr(Instruction::CallEnd);
            }
            CompilableData::Null => {
                // block.add_instr(Instruction::Default(Value::Null));
                block.set_default(Value::Null);
            }
            CompilableData::Bool(v) => {
                // block.add_instr(Instruction::Default(Value::Boolean(*v)));
                block.set_default(Value::Boolean(*v));
            }
            CompilableData::Int(v) => {
                // block.add_instr(Instruction::Default(Value::Integer(*v)));
                block.set_default(Value::Integer(*v));
            }
            CompilableData::Float(v) => {
                // block.add_instr(Instruction::Default(Value::Float(*v)));
                block.set_default(Value::Float(*v));
            }
            CompilableData::String(v) => {
                // block.add_instr(Instruction::Default(Value::String(v.clone())));
                block.set_default(Value::String(v.clone()));
            }
            CompilableData::Symbol(s) => {
                let reg = block.get_reg_for(s).clone();
                let name_manager = block.get_name_manager(s);
                if name_manager.used_first_time() {
                    (*block).add_instr(Instruction::GetMemberInScope(s.to_string()));
                    (*block).add_instr(Instruction::CopyFromDefault(reg));
                } else {
                    (*block).add_instr(Instruction::CopyToDefault(reg));
                }
                block.use_name(s);
            }
            CompilableData::Array(v) => {
                let reg = block.get_reg();
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
                block.free_reg(&reg);
            }
            CompilableData::Map(v) => {
                // (*block).add_instr(Instruction::Default(Value::Map(v.clone())));
                block.set_default(Value::Map(v.clone()));
                let reg = block.get_reg();
                (*block).add_instr(Instruction::CopyFromDefault(reg));
                for child in node.children() {
                    match &child.value().data {
                        CompilableData::MapChild(key) => {
                            let value_node = child.first_child().unwrap();
                            self.compile_node(&value_node, block);
                            (*block).add_instr(Instruction::SetProp(reg, key.clone()));
                        }
                        _ => unimplemented!()
                    }
                }
                (*block).add_instr(Instruction::CopyToDefault(reg));
                block.free_reg(&reg);
            }
            CompilableData::Var(name, is_top_level) => {
                self.compile_node(&node.first_child().unwrap(), block);
                (*block).add_instr(Instruction::DefMemberInScope(name.clone()));
                let reg = block.get_reg_for(&name);
                (*block).add_instr(Instruction::CopyFromDefault(reg));
                block.use_name(name);
            }
            CompilableData::BinaryOp(op) => {
                let first = node.first_child().unwrap();
                self.compile_node(&first, block);
                let (allocated, first_reg) = block.save_default_to_reg();

                let second = first.next_sibling().unwrap();
                self.compile_node(&second, block);

                // (*block).add_instr(Instruction::BinaryOp(op.clone(), first_reg));
                match op as &str {
                    "+"  => (*block).add_instr(Instruction::Add(first_reg)),
                    "-"  => (*block).add_instr(Instruction::Sub(first_reg)),
                    "==" => (*block).add_instr(Instruction::Eq(first_reg)),
                    "<"  => (*block).add_instr(Instruction::Lt(first_reg)),
                    ">"  => (*block).add_instr(Instruction::Gt(first_reg)),
                    _ => unimplemented!()
                }

                if allocated {
                    block.free_reg(&first_reg);
                }
            }
            CompilableData::Assignment(name) => {
                self.compile_node(&node.first_child().unwrap(), block);

                // Update register allocated for <name>
                let reg = block.get_reg_for(name);
                (*block).add_instr(Instruction::CopyFromDefault(reg));
                block.use_name(name);

                (*block).add_instr(Instruction::SetMemberInScope(name.clone()));
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
            CompilableData::Function(name, matcher, body) => {
                (*block).add_instr(Instruction::Function(name.to_string(), matcher.clone(), body.to_string()));
                let reg = block.get_reg_for(&name);
                (*block).add_instr(Instruction::CopyFromDefault(reg));
                block.use_name(name);
            }
            CompilableData::Invocation => {
                let target_node = node.first_child().unwrap();
                self.compile_node(&target_node, block);
                let (allocated, target_reg) = block.save_default_to_reg();

                if let Some(args_node) = target_node.next_sibling() {
                    let args_reg = block.get_reg();
                    args_node.value().set_u16("reg", args_reg);
                    self.compile_node(&args_node, block);
                    (*block).add_instr(Instruction::Call(target_reg, Some(args_reg), HashMap::new()));
                    block.free_reg(&args_reg);
                } else {
                    (*block).add_instr(Instruction::Call(target_reg, None, HashMap::new()));
                }

                if allocated {
                    block.free_reg(&target_reg);
                }
            }
            CompilableData::InvocationArguments(_v) => {
                let reg = node.value().get_u16("reg");
                (*block).add_instr(Instruction::CreateArguments(reg));
                for child in node.children() {
                    match child.value().data {
                        CompilableData::ArrayChild(i) => {
                            self.compile_node(&child.first_child().unwrap(), block);
                            (*block).add_instr(Instruction::SetItem(reg, i));
                        }
                        _ => unimplemented!()
                    }
                }
            }
            CompilableData::Loop => {
                let start_pos = block.len();
                for node in node.children() {
                    self.compile_node(&node, block);
                }
                (*block).add_instr(Instruction::Jump(start_pos as i16));
                (*block).add_instr(Instruction::LoopEnd);
            }
            CompilableData::While => {
                let start_pos = block.len();
                let cond_node = node.first_child().unwrap();
                self.compile_node(&cond_node, block);

                let jump_pos = block.len();
                (*block).add_instr(Instruction::JumpToElse);

                let mut sibling = cond_node.next_sibling();
                while sibling.is_some() {
                    let sibling_node = sibling.unwrap();
                    self.compile_node(&sibling_node, block);
                    sibling = sibling_node.next_sibling();
                }

                (*block).add_instr(Instruction::Jump(start_pos as i16));
                (*block).add_instr(Instruction::LoopEnd);

                let end_pos = block.len();
                mem::replace(&mut (*block).instructions[jump_pos], Instruction::JumpIfFalse(end_pos as i16));
            }
            CompilableData::Break => {
                (*block).add_instr(Instruction::Break);
            }
            _ => unimplemented!()
        }
    }
}

/// Analyze names referenced
/// For each name, we want to know
///   - Whether it is a member of namespace or a variable in local scope or parent scope
///   - How many time is it used (considered many if it's used inside a loop)
///   - Whether it should be hoisted
///     If it is defined in a if branch, it should be rewritten like
///     Example 1:
///     (if a then (var b 1))
///     vvv
///     (if a then (var b 1) else (var b))
///
///     Example 2:
///     (if a then
///       (if b then (var c 1))
///     )
///     vvv
///     (if a then
///       (if b then (var c 1) else (var c))
///     else
///       (var c)
///     )
///
///     Example 3:
///     (a && (var b = 1))
///     vvv
///     ???
///
///     Example 4:
///     (a || (var b = 1))
///     vvv
///     ???
pub fn analyze_names<'a>(node: &'a mut NodeRef<'a, Compilable>) {
}

pub struct NodeWrapper<'a>(&'a NodeRef<'a, Compilable>);

impl<'a> NodeWrapper<'a> {
    pub fn get_name_usage(&self) -> HashMap<String, usize> {
        let mut result = HashMap::new();

        match self.0.value().data {
            CompilableData::Symbol(ref name) => {
                result.insert(name.clone(), 1);
            }
            CompilableData::Var(ref name, _is_top_level) => {
                result.insert(name.clone(), 1);
            }
            CompilableData::Function(ref name, ..) => {
                result.insert(name.clone(), 1);
            }
            _ => {}
        }

        for child in self.0.children() {
            let usage = NodeWrapper(&child).get_name_usage();
            for (name, value) in usage.iter() {
                if result.contains_key(name) {
                    result.insert(name.clone(), result[name] + value);
                } else {
                    result.insert(name.clone(), *value);
                }
            }
        }

        result
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
    pub options: RefCell<HashMap<String, Box<dyn Any>>>,
    // pub start_pos: Option<usize>,
    // pub instr_count: Option<usize>,
}

impl Compilable {
    pub fn new(data: CompilableData) -> Self {
        Compilable {
            data,
            options: RefCell::new(HashMap::new()),
            // start_pos: None,
            // instr_count: None,
        }
    }

    pub fn get_u16(&self, name: &str) -> u16 {
        let borrowed = self.options.borrow();
        *borrowed[name].downcast_ref::<u16>().unwrap()
    }
    pub fn set_u16(&self, name: &str, value: u16) {
        self.set(name, Box::new(value));
    }

    pub fn set(&self, name: &str, value: Box<dyn Any>) {
        self.options.borrow_mut().insert(name.to_string(), value);
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
    /// Var(name, is_top_level)
    Var(String, bool),
    BinaryOp(String),
    Assignment(String),
    If,
    IfPair,
    IfPairCondition,
    IfPairThen,
    IfElse,
    /// Function(name, arguments, body block uuid)
    Function(String, Matcher, String),
    Invocation,
    InvocationArguments(Vec<Value>),
    Loop,
    While,
    // WhileCondition,
    // WhileBody,
    Break,
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
        if self.data.is_empty() {
            return Gene{
                kind:  self.kind.clone(),
                props: self.props.clone(),
                data:  self.data.clone(),
            }
        }

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