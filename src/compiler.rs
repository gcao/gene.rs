extern crate rand;

use std::mem;
use std::any::Any;
use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

use rand::prelude::random;

use super::types::Gene;
use super::types::Value;
use super::vm::types::{Function, Matcher};
use super::utils::new_uuidv4;

trait LiteralCheck {
    fn is_literal(&self) -> bool;
}

impl LiteralCheck for Value {
    fn is_literal(&self) -> bool {
        match self {
            Value::Symbol(s) => false,
            Value::Array(v) => v.is_literal(),
            Value::Gene(g) => g.is_literal(),
            _ => true
        }
    }
}

impl LiteralCheck for Vec<Value> {
    fn is_literal(&self) -> bool {
        self.iter().all(|item| item.is_literal())
    }
}

impl LiteralCheck for BTreeMap<String, Value> {
    fn is_literal(&self) -> bool {
        false
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
}

impl Compiler {
    pub fn new() -> Self {
        let module = Module::new();
        Compiler {
            module: Rc::new(RefCell::new(module)),
        }
    }

    pub fn compile(&mut self, ast: Value) -> Rc<RefCell<Module>> {
        let mut block = Block::new("__default__".to_string());
        // let block_id = block.id.clone();
        block.add_instr(Instruction::Init);
        self.compile_(&mut block, ast);
        block.add_instr(Instruction::CallEnd);

        println!("Block: {}", block);

        let mut module = self.module.borrow_mut();
        module.set_default_block(Rc::new(block));
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
                self.compile_gene(block, normalize(v));
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
            let reg = new_reg();
            (*block).add_instr(Instruction::Save(reg.clone(), Value::Array(arr2)));

            let mut index = 0;
            for item in arr.iter() {
                if !item.is_literal() {
                    self.compile_(block, item.clone());
                    (*block).add_instr(Instruction::SetItem(reg.clone(), index, "default".to_string()));
                }
                index += 1;
            }

            // Copy to default register
            (*block).add_instr(Instruction::Copy(reg, "default".to_string()));
        }
    }

    fn compile_map(&mut self, block: &mut Block, map: BTreeMap<String, Value>) {
        if map.is_literal() {
            (*block).add_instr(Instruction::Default(Value::Map(map)));
        } else {
            let mut map2 = BTreeMap::<String, Value>::new();
            for (key, value) in map.iter() {
                if value.is_literal() {
                    map2.insert(key.clone(), value.clone());
                }
            }
            let reg = new_reg();
            (*block).add_instr(Instruction::Save(reg.clone(), Value::Map(map2)));

            for (key, value) in map.iter() {
                if !value.is_literal() {
                    self.compile_(block, value.clone());
                    (*block).add_instr(Instruction::SetProp(reg.clone(), key.clone(), "default".to_string()));
                }
            }

            // Copy to default register
            (*block).add_instr(Instruction::Copy(reg, "default".to_string()));
        }
    }

    fn compile_gene(&mut self, block: &mut Block, gene: Gene) {
        let Gene {
            _type, data, props,
        } = gene;

        match *_type.borrow() {
            Value::Symbol(ref s) if s == "var" => {
                let first;
                {
                    first = data[0].clone();
                }
                let second;
                {
                    second = data[1].borrow().clone();
                }
                match *first.borrow_mut() {
                    Value::Symbol(ref name) => {
                        self.compile_(block, second.clone());
                        (*block).add_instr(Instruction::DefMember(name.clone(), "default".to_string()));
                    }
                    _ => unimplemented!(),
                };
            }
            Value::Symbol(ref s) if s == "fn" => {
                let name = data[0].borrow().to_string();

                let mut body = Block::new(name.clone());
                let body_id = body.id.clone();

                let matcher = self.compile_args(&mut body, data[1].clone());

                self.compile_statements(&mut body, &data[2..]);
                let mut module = self.module.borrow_mut();
                module.add_block(body_id.clone(), body);

                (*block).add_instr(Instruction::Function(name, matcher, body_id));
            }
            Value::Symbol(ref s) if s == "if" => {
                self.compile_if(block, data);
            }
            Value::Symbol(ref s) if s == "=" => {
                let name = data[0].borrow().to_string();
                let value = data[1].borrow().clone();
                self.compile_(block, value);
                (*block).add_instr(Instruction::SetMember(name, "default".to_string()));
            }
            Value::Symbol(ref s) if is_binary_op(s) => {
                let first = data[0].borrow().clone();
                self.compile_(block, first);

                let first_reg = new_reg();
                (*block).add_instr(Instruction::Copy("default".to_string(), first_reg.clone()));

                let second = data[1].borrow().clone();
                self.compile_(block, second);

                (*block).add_instr(Instruction::BinaryOp(
                    s.to_string(),
                    first_reg,
                    "default".to_string(),
                ));
            }
            Value::Symbol(ref s) if s == "while" => {
                self.compile_while(block, data);
            }
            Value::Symbol(ref s) if s == "break" => {
                (*block).add_instr(Instruction::Break);
            }
            _ => {
                // Invocation
                let borrowed_type = _type.borrow().clone();
                self.compile_(block, borrowed_type);
                let target_reg = new_reg();
                (*block).add_instr(Instruction::Copy("default".to_string(), target_reg.clone()));

                let mut options = BTreeMap::<String, Rc<Any>>::new();

                let args_reg = new_reg();
                (*block).add_instr(Instruction::CreateArguments(args_reg.clone()));
                for (i, item) in data.iter().enumerate() {
                    let borrowed = item.borrow();
                    self.compile_(block, (*borrowed).clone());
                    (*block).add_instr(Instruction::SetItem(args_reg.clone(), i, "default".to_string()));
                }

                options.insert("args".to_string(), Rc::new(args_reg.clone()));

                (*block).add_instr(Instruction::Call(target_reg.clone(), options));
            }
        };
    }

    fn compile_args(&mut self, block: &mut Block, args: Rc<RefCell<Value>>) -> Matcher {
        let borrowed = args.borrow();
        return Matcher::from(&*borrowed);
    }

    fn compile_statements(&mut self, block: &mut Block, stmts: &[Rc<RefCell<Value>>]) {
        for item in stmts.iter().cloned() {
            let borrowed = item.borrow().clone();
            self.compile_(block, borrowed);
        }
    }

    fn compile_if(&mut self, block: &mut Block, mut data: Vec<Rc<RefCell<Value>>>) {
        let cond = data.remove(0);
        let mut then_stmts = Vec::<Rc<RefCell<Value>>>::new();
        let mut else_stmts = Vec::<Rc<RefCell<Value>>>::new();
        let mut is_else = false;
        for item in data.iter() {
            if is_else {
                else_stmts.push(item.clone());
            } else {
                let borrowed_item = item.borrow();
                match *borrowed_item {
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
        self.compile_(block, cond.borrow().clone());
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

    fn compile_while(&mut self, block: &mut Block, mut data: Vec<Rc<RefCell<Value>>>) {
        let start_index = block.instructions.len();

        (*block).add_instr(Instruction::LoopStart);

        let cond = data.remove(0);
        self.compile_(block, cond.borrow().clone());
        let jump_index = block.instructions.len();
        (*block).add_instr(Instruction::Dummy);

        for item in data.iter() {
            self.compile_(block, item.borrow().clone());
        }
        (*block).add_instr(Instruction::Jump(start_index as i16));
        (*block).add_instr(Instruction::LoopEnd);

        let end_index = block.instructions.len();
        mem::replace(&mut (*block).instructions[jump_index], Instruction::JumpIfFalse(end_index as i16));
    }
}

pub struct Statements(Vec<Value>);

#[derive(Debug)]
pub struct Module {
    pub id: String,
    pub blocks: BTreeMap<String, Rc<Block>>,
    default_block_id: String,
}

impl Module {
    pub fn new() -> Self {
        Module {
            id: new_uuidv4(),
            blocks: BTreeMap::new(),
            default_block_id: "".to_string(),
        }
    }

    pub fn set_default_block(&mut self, block: Rc<Block>) {
        self.default_block_id = block.id.clone();
        self.blocks.insert(block.id.clone(), block.clone());
    }

    pub fn get_default_block(&self) -> Rc<Block> {
        let block = &self.blocks[&self.default_block_id];
        block.clone()
    }

    pub fn add_block(&mut self, id: String, block: Block) {
        self.blocks.insert(id, Rc::new(block));
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
        let instructions = vec![];
        Block {
            id: new_uuidv4(),
            name,
            instructions,
        }
    }

    pub fn add_instr(&mut self, instr: Instruction) {
        self.instructions.push(instr);
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
    Save(String, Value),
    /// Copy from one register to another
    Copy(String, String),

    DefMember(String, String),
    GetMember(String),
    SetMember(String, String),

    // // Literal types: number, string, boolean, array of literals, map of literals, gene with literals only
    // // TODO: Is it better to use individual instruction like CreateInt etc?
    // CreateLiteral(String, Box<Any>),

    /// GetItem(target reg, index)
    GetItem(String, usize),
    // /// GetItemDynamic(target reg, index reg)
    // GetItemDynamic(String, String),
    /// SetItem(target reg, index, value reg)
    SetItem(String, usize, String),
    // /// SetItemDynamic(target reg, index reg, value reg)
    // SetItemDynamic(String, String, String),

    // /// GetProp(target reg, name)
    // GetProp(String, String),
    // /// GetPropDynamic(target reg, name reg)
    // GetPropDynamic(String, String),
    /// SetProp(target reg, name, value reg)
    SetProp(String, String, String),
    // /// SetPropDynamic(target reg, name reg, value reg)
    // SetPropDynamic(String, String, String),

    Jump(i16),
    JumpIfFalse(i16),
    Break,
    LoopStart,
    LoopEnd,

    /// BinaryOp(op, first reg, second reg)
    /// Result is stored in default reg
    BinaryOp(String, String, String),

    /// Function(name, args reg, block id)
    Function(String, Matcher, String),
    /// Create an argument object and store in a register
    CreateArguments(String),

    /// Call(options)
    Call(String, BTreeMap<String, Rc<Any>>),
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
                fmt.write_str(reg)?;
                fmt.write_str(" ")?;
                fmt.write_str(&v.to_string())?;
            }
            Instruction::Copy(first, second) => {
                fmt.write_str("Copy ")?;
                fmt.write_str(first)?;
                fmt.write_str(" ")?;
                fmt.write_str(second)?;
            }
            Instruction::DefMember(name, reg) => {
                fmt.write_str("DefMember ")?;
                fmt.write_str(name)?;
                fmt.write_str(" ")?;
                fmt.write_str(reg)?;
            }
            Instruction::GetMember(name) => {
                fmt.write_str("GetMember ")?;
                fmt.write_str(name)?;
            }
            Instruction::SetMember(name, reg) => {
                fmt.write_str("SetMember ")?;
                fmt.write_str(name)?;
                fmt.write_str(" ")?;
                fmt.write_str(reg)?;
            }
            Instruction::GetItem(name, index) => {
                fmt.write_str("GetItem ")?;
                fmt.write_str(name)?;
                fmt.write_str(" ")?;
                fmt.write_str(&index.to_string())?;
            }
            Instruction::SetItem(name, index, value) => {
                fmt.write_str("Get ")?;
                fmt.write_str(name)?;
                fmt.write_str(" ")?;
                fmt.write_str(&index.to_string())?;
                fmt.write_str(" ")?;
                fmt.write_str(value)?;
            }
            Instruction::SetProp(name, key, value) => {
                fmt.write_str("Get ")?;
                fmt.write_str(name)?;
                fmt.write_str(" ")?;
                fmt.write_str(key)?;
                fmt.write_str(" ")?;
                fmt.write_str(value)?;
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
            Instruction::BinaryOp(op, first, second) => {
                fmt.write_str(first)?;
                fmt.write_str(" ")?;
                fmt.write_str(op)?;
                fmt.write_str(" ")?;
                fmt.write_str(second)?;
            }
            Instruction::Function(name, matcher, body_id) => {
                fmt.write_str("Function ")?;
                fmt.write_str(name)?;
                fmt.write_str(" ")?;
                // TODO: matcher to string
                // fmt.write_str(&matcher)?;
                // fmt.write_str(" ")?;
                fmt.write_str(body_id)?;
            }
            Instruction::Call(target, options) => {
                fmt.write_str("Call ")?;
                fmt.write_str(target)?;
            }
            Instruction::CallEnd => {
                fmt.write_str("CallEnd")?;
            }
            Instruction::CreateArguments(reg) => {
                fmt.write_str("CreateArguments ")?;
                fmt.write_str(reg)?;
            }
            // _ => {
            //     fmt.write_str("???")?;
            // }
        }
        fmt.write_str(")")?;
        Ok(())
    }
}

fn new_reg() -> String {
    format!("{}", random::<u32>())
}

fn is_binary_op(op: &str) -> bool {
    let binary_ops = vec!["+", "<"];
    binary_ops.contains(&op)
}

fn normalize(gene: Gene) -> Gene {
    if gene.data.is_empty() {
        gene
    } else {
        let borrowed = gene.data[0].clone();
        let first = borrowed.borrow_mut();
        match *first {
            Value::Symbol(ref s) if is_binary_op(s) || s == "=" => {
                let Gene {
                    _type,
                    mut data,
                    props,
                } = gene;
                let new_type = data.remove(0);
                data.insert(0, _type);
                Gene {
                    _type: new_type,
                    props,
                    data,
                }
            }
            _ => gene,
        }
    }
}
