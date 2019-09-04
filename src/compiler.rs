extern crate rand;

use std::mem;
use std::any::Any;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use rand::prelude::random;

use super::types::Gene;
use super::types::Value;
use super::vm::types::{Function, Matcher};
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

#[derive(Debug)]
pub struct Compiler {
    module: Rc<RefCell<Module>>,
    // reg_trackers:
    //   key: block id,
    //   value: list of registers being used
    reg_trackers: HashMap<String, Vec<u16>>,
}

impl Compiler {
    pub fn new() -> Self {
        let module = Module::new();
        Compiler {
            module: Rc::new(RefCell::new(module)),
            reg_trackers: HashMap::new(),
        }
    }

    pub fn compile(&mut self, ast: Value) -> Rc<RefCell<Module>> {
        let mut block = Block::new("__default__".to_string());

        self.reg_trackers.insert(block.id.clone(), Vec::new());

        block.add_instr(Instruction::Init);
        self.compile_(&mut block, ast);
        block.add_instr(Instruction::CallEnd);

        println!("{}", block);

        let mut module = self.module.borrow_mut();
        module.set_default_block(block);
        self.module.clone()
    }

    fn compile_(&mut self, block: &mut Block, ast: Value) {
        match ast {
            Value::Symbol(s) => {
                (*block).add_instr(Instruction::GetMember(s));
            }
            Value::Array(v) => {
                self.compile_array(block, v)
            }
            Value::Map(m) => {
                self.compile_map(block, m)
            }
            Value::Gene(v) => {
                self.compile_gene(block, normalize(*v));
            }
            Value::Stream(stmts) => {
                for stmt in stmts {
                    self.compile_(block, stmt);
                }
            }
            _ => {
                (*block).add_instr(Instruction::Default(ast));
            }
        };
    }

    fn compile_array(&mut self, block: &mut Block, arr: Vec<Value>) {
        if arr.is_literal() {
            (*block).add_instr(Instruction::Default(Value::Array(arr)));
        } else {
            let mut arr2 = Vec::<Value>::new();
            for item in arr.iter() {
                if item.is_literal() {
                    arr2.push(item.clone());
                } else {
                    arr2.push(Value::Void);
                }
            }
            let reg = self.get_reg(block);
            (*block).add_instr(Instruction::Save(reg, Value::Array(arr2)));

            for (index, item) in arr.iter().enumerate() {
                if !item.is_literal() {
                    self.compile_(block, item.clone());
                    (*block).add_instr(Instruction::SetItem(reg, index));
                }
            }

            // Copy to default register
            (*block).add_instr(Instruction::CopyToDefault(reg));
            self.free_reg(block, reg);
        }
    }

    fn compile_map(&mut self, block: &mut Block, map: HashMap<String, Value>) {
        if map.is_literal() {
            (*block).add_instr(Instruction::Default(Value::Map(map)));
        } else {
            let mut map2 = HashMap::<String, Value>::new();
            for (key, value) in map.iter() {
                if value.is_literal() {
                    map2.insert(key.clone(), value.clone());
                }
            }
            let reg = self.get_reg(block);
            (*block).add_instr(Instruction::Save(reg, Value::Map(map2)));

            for (key, value) in map.iter() {
                if !value.is_literal() {
                    self.compile_(block, value.clone());
                    (*block).add_instr(Instruction::SetProp(reg, key.clone()));
                }
            }

            // Copy to default register
            (*block).add_instr(Instruction::CopyToDefault(reg));
            self.free_reg(block, reg);
        }
    }

    fn compile_gene(&mut self, block: &mut Block, gene: Gene) {
        let Gene {
            kind, data, ..
        } = gene;

        match kind {
            Value::Symbol(ref s) if s == "var" => {
                let first;
                {
                    first = data[0].clone();
                }
                let second;
                {
                    second = data[1].clone();
                }
                match first {
                    Value::Symbol(ref name) => {
                        self.compile_(block, second.clone());
                        (*block).add_instr(Instruction::DefMember(name.clone()));
                    }
                    _ => unimplemented!(),
                };
            }
            Value::Symbol(ref s) if s == "fn" => {
                let name = data[0].to_string();

                let mut body = Block::new(name.clone());
                let body_id = body.id.clone();

                self.reg_trackers.insert(body_id.clone(), Vec::new());

                let borrowed = data[1].clone();
                let matcher =  Matcher::from(&borrowed);

                self.compile_statements(&mut body, &data[2..]);
                body.add_instr(Instruction::CallEnd);
                println!("{}", body);

                let mut module = self.module.borrow_mut();
                module.add_block(body);

                (*block).add_instr(Instruction::Function(name, matcher, body_id));
            }
            Value::Symbol(ref s) if s == "if" => {
                self.compile_if(block, data);
            }
            Value::Symbol(ref s) if s == "=" => {
                let name = data[0].to_string();
                let value = data[1].clone();
                self.compile_(block, value);
                (*block).add_instr(Instruction::SetMember(name));
            }
            Value::Symbol(ref s) if is_binary_op(s) => {
                let first = data[0].clone();
                self.compile_(block, first);

                let first_reg = self.get_reg(block);
                (*block).add_instr(Instruction::CopyFromDefault(first_reg));

                let second = data[1].clone();
                self.compile_(block, second);

                (*block).add_instr(Instruction::BinaryOp(s.to_string(), first_reg));
                self.free_reg(block, first_reg);
            }
            Value::Symbol(ref s) if s == "while" => {
                self.compile_while(block, data);
            }
            Value::Symbol(ref s) if s == "break" => {
                (*block).add_instr(Instruction::Break);
            }
            _ => {
                // Invocation
                self.compile_(block, kind);
                let target_reg = self.get_reg(block);
                (*block).add_instr(Instruction::CopyFromDefault(target_reg));

                let options = HashMap::<String, Rc<dyn Any>>::new();

                let args_reg = self.get_reg(block);
                (*block).add_instr(Instruction::CreateArguments(args_reg));
                for (i, item) in data.iter().enumerate() {
                    self.compile_(block, item.clone());
                    (*block).add_instr(Instruction::SetItem(args_reg, i));
                }

                (*block).add_instr(Instruction::Call(target_reg, Some(args_reg), options));
                self.free_reg(block, target_reg);
                self.free_reg(block, args_reg);
            }
        };
    }

    fn compile_statements(&mut self, block: &mut Block, stmts: &[Value]) {
        for item in stmts.iter().cloned() {
            self.compile_(block, item);
        }
    }

    fn compile_if(&mut self, block: &mut Block, mut data: Vec<Value>) {
        let cond = data.remove(0);
        let mut then_stmts = Vec::new();
        let mut else_stmts = Vec::new();
        let mut is_else = false;
        for item in data.iter() {
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
        self.compile_(block, cond.clone());
        let cond_jump_index = block.instructions.len();
        (*block).add_instr(Instruction::Dummy);

        self.compile_statements(block, &then_stmts);
        let then_jump_index = block.instructions.len();
        (*block).add_instr(Instruction::Dummy);

        self.compile_statements(block, &else_stmts);

        let end_index = block.instructions.len();

        let else_start = then_jump_index + 1;
        mem::replace(&mut (*block).instructions[cond_jump_index], Instruction::JumpIfFalse(else_start as i16));
        mem::replace(&mut (*block).instructions[then_jump_index], Instruction::Jump(end_index as i16));
    }

    fn compile_while(&mut self, block: &mut Block, mut data: Vec<Value>) {
        let start_index = block.instructions.len();

        (*block).add_instr(Instruction::LoopStart);

        let cond = data.remove(0);
        self.compile_(block, cond);
        let jump_index = block.instructions.len();
        (*block).add_instr(Instruction::Dummy);

        for item in data.iter() {
            self.compile_(block, item.clone());
        }
        (*block).add_instr(Instruction::Jump(start_index as i16));
        (*block).add_instr(Instruction::LoopEnd);

        let end_index = block.instructions.len();
        mem::replace(&mut (*block).instructions[jump_index], Instruction::JumpIfFalse(end_index as i16));
    }

    /// 1. find and return available register
    /// 2. if all registers are occupied
    fn get_reg(&mut self, block: &mut Block) -> u16 {
        let trackers = self.reg_trackers.get_mut(&block.id).unwrap();
        for i in 0..16 {
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
}

impl Block {
    pub fn new(name: String) -> Self {
        let instructions = Vec::new();
        Block {
            id: new_uuidv4(),
            name,
            instructions,
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

    DefMember(String),
    GetMember(String),
    SetMember(String),

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

    /// BinaryOp(op, first reg)
    /// Second operand is in default reg
    /// Result is stored in default reg
    BinaryOp(String, u16),

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
                fmt.write_str("Dummy")?;
            }
            Instruction::Init => {
                fmt.write_str("Init")?;
            }
            Instruction::Default(v) => {
                fmt.write_str("Default ")?;
                fmt.write_str(&v.to_string())?;
            }
            Instruction::Save(reg, v) => {
                fmt.write_str("Save ")?;
                fmt.write_str(&reg.to_string())?;
                fmt.write_str(" ")?;
                fmt.write_str(&v.to_string())?;
            }
            Instruction::CopyFromDefault(reg) => {
                fmt.write_str("CopyFromDefault ")?;
                fmt.write_str(&reg.to_string())?;
            }
            Instruction::CopyToDefault(reg) => {
                fmt.write_str("CopyToDefault ")?;
                fmt.write_str(&reg.to_string())?;
            }
            Instruction::DefMember(name) => {
                fmt.write_str("DefMember ")?;
                fmt.write_str(name)?;
            }
            Instruction::GetMember(name) => {
                fmt.write_str("GetMember ")?;
                fmt.write_str(name)?;
            }
            Instruction::SetMember(name) => {
                fmt.write_str("SetMember ")?;
                fmt.write_str(name)?;
            }
            Instruction::GetItem(reg, index) => {
                fmt.write_str("GetItem ")?;
                fmt.write_str(&reg.to_string())?;
                fmt.write_str(" ")?;
                fmt.write_str(&index.to_string())?;
            }
            Instruction::SetItem(reg, index) => {
                fmt.write_str("SetItem ")?;
                fmt.write_str(&reg.to_string())?;
                fmt.write_str(" ")?;
                fmt.write_str(&index.to_string())?;
            }
            Instruction::SetProp(reg, key) => {
                fmt.write_str("Get ")?;
                fmt.write_str(&reg.to_string())?;
                fmt.write_str(" ")?;
                fmt.write_str(key)?;
            }
            Instruction::Jump(pos) => {
                fmt.write_str("Jump ")?;
                fmt.write_str(&pos.to_string())?;
            }
            Instruction::JumpIfFalse(pos) => {
                fmt.write_str("JumpIfFalse ")?;
                fmt.write_str(&pos.to_string())?;
            }
            Instruction::Break => {
                fmt.write_str("Break")?;
            }
            Instruction::LoopStart => {
                fmt.write_str("LoopStart")?;
            }
            Instruction::LoopEnd => {
                fmt.write_str("LoopEnd")?;
            }
            Instruction::BinaryOp(op, first) => {
                fmt.write_str(&first.to_string())?;
                fmt.write_str(" ")?;
                fmt.write_str(op)?;
                fmt.write_str(" Default")?;
            }
            Instruction::Function(name, _matcher, body_id) => {
                fmt.write_str("Function ")?;
                fmt.write_str(name)?;
                fmt.write_str(" ")?;
                // TODO: matcher to string
                // fmt.write_str(&matcher)?;
                // fmt.write_str(" ")?;
                fmt.write_str(body_id)?;
            }
            Instruction::Call(target_reg, args_reg, _options) => {
                fmt.write_str("Call ")?;
                fmt.write_str(&target_reg.to_string())?;
                fmt.write_str(" ")?;
                if let Some(reg) = args_reg {
                    fmt.write_str(&reg.to_string())?;
                }
            }
            Instruction::CallEnd => {
                fmt.write_str("CallEnd")?;
            }
            Instruction::CreateArguments(reg) => {
                fmt.write_str("CreateArguments ")?;
                fmt.write_str(&reg.to_string())?;
            }
            _ => {
                fmt.write_str("???")?;
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

fn normalize(gene: Gene) -> Gene {
    if gene.data.is_empty() {
        gene
    } else {
        let first = gene.data[0].clone();
        match first {
            Value::Symbol(ref s) if is_binary_op(s) || s == "=" => {
                let Gene {
                    kind,
                    props,
                    mut data,
                } = gene;
                let new_kind = data.remove(0);
                data.insert(0, kind.clone());
                Gene {
                    kind: new_kind,
                    props,
                    data,
                }
            }
            _ => gene,
        }
    }
}
