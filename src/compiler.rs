extern crate rand;

use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap};
use std::fmt;

use rand::prelude::random;

use super::types::Value;
use super::types::Gene;
use super::utils::new_uuidv4;

#[derive(Debug)]
pub struct Compiler {
    module: Module,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            module: Module::new(),
        }
    }

    pub fn compile(&mut self, ast: Value) -> Module {
        let mut block = Block::new("__default__".to_string());
        let block_id = block.id.clone();
        block.add_instr(Instruction::Init);
        self.compile_(&mut block, ast);
        block.add_instr(Instruction::CallEnd);

        println!("Block: {}", block);

        self.module.set_default_block(block);
        self.module.clone()
    }

    fn compile_(&mut self, block: &mut Block, ast: Value) -> () {
        match ast {
            Value::Symbol(s) => {
                (*block).add_instr(Instruction::GetMember(s));
            }
            Value::Array(v) => {
                // TODO: compile individual elements
                (*block).add_instr(Instruction::Default(Value::Array(v)));
            }
            Value::Map(_) => {
                // TODO: compile individual values
                (*block).add_instr(Instruction::Default(ast));
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

    fn compile_gene(&mut self, block: &mut Block, gene: Gene) -> () {
        let Gene {_type, mut data, ..} = gene;

        if *_type.borrow() == Value::Symbol("var".to_string()) {
            let first;
            {
                first = data.get_mut(0).unwrap().clone();
            }
            let second;
            {
                second = data.get(1).unwrap().borrow().clone();
            }
            match *first.borrow_mut() {
                Value::Symbol(ref name) => {
                    self.compile_(block, second.clone());
                    (*block).add_instr(Instruction::Define(name.clone(), "default".to_string()));
                }
                _ => unimplemented!()
            };
        } else if *_type.borrow() == Value::Symbol("+".to_string()) {
            let first = data.get(0).unwrap().borrow().clone();
            self.compile_(block, first);

            let first_reg = new_reg();
            (*block).add_instr(Instruction::Copy("default".to_string(), first_reg.clone()));

            let second = data.get(1).unwrap().borrow().clone();
            self.compile_(block, second);

            (*block).add_instr(Instruction::BinaryOp("+".to_string(), first_reg, "default".to_string()));
        };
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    pub id: String,
    pub blocks: BTreeMap<String, Block>,
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

    pub fn set_default_block(&mut self, block: Block) -> () {
        self.default_block_id = block.id.clone();
        self.blocks.insert(block.id.clone(), block);
    }

    pub fn get_default_block(&mut self) -> Block {
        self.blocks[&self.default_block_id].clone()
    }
}

#[derive(Clone, Debug)]
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
            name: name,
            instructions: instructions,
        }
    }

    pub fn add_instr(&mut self, instr: Instruction) -> () {
        self.instructions.push(instr);
    }
}

impl fmt::Display for Block {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("(Block ")?;
        fmt.write_str(&self.name)?;
        fmt.write_str("\n")?;
        for instr in &self.instructions {
            fmt.write_str(&instr.to_string())?;
            fmt.write_str("\n")?;
        }
        fmt.write_str(")")?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Instruction {
    Init,

    /// Save Value to default register
    Default(Value),
    /// Copy from one register to another
    Copy(String, String),

    Define(String, String),
    GetMember(String),

    BinaryOp(String, String, String),

    CallEnd,
}

impl fmt::Display for Instruction {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("(")?;
        match &self {
            Instruction::Init => {
                fmt.write_str("Init")?;
            }
            Instruction::Default(v) => {
                fmt.write_str("Default ")?;
                fmt.write_str(&v.to_string())?;
            }
            Instruction::Copy(first, second) => {
                fmt.write_str("Copy ")?;
                fmt.write_str(&first)?;
                fmt.write_str(" ")?;
                fmt.write_str(&second)?;
            }
            Instruction::Define(name, reg) => {
                fmt.write_str("Define ")?;
                fmt.write_str(&name)?;
                fmt.write_str(" ")?;
                fmt.write_str(&reg)?;
            }
            Instruction::GetMember(name) => {
                fmt.write_str("GetMember ")?;
                fmt.write_str(&name)?;
            }
            Instruction::BinaryOp(op, first, second) => {
                fmt.write_str(&first)?;
                fmt.write_str(" ")?;
                fmt.write_str(&op)?;
                fmt.write_str(" ")?;
                fmt.write_str(&second)?;
            }
            Instruction::CallEnd => {
                fmt.write_str("CallEnd")?;
            }
            _ => {
                fmt.write_str("???")?;
            }
        }
        fmt.write_str(")")?;
        Ok(())
    }
}

fn new_reg() -> String {
    format!("{}", random::<u32>())
}

fn normalize(gene: Gene) -> Gene {
    if gene.data.len() == 0 {
        gene
    } else {
        let borrowed = gene.data[0].clone();
        let first = borrowed.borrow_mut();
        match *first {
            Value::Symbol(ref s) if s == "+" => {
                let Gene {_type, mut data, props} = gene;
                let new_type = data.remove(0);
                data.insert(0, _type);
                Gene {
                    _type: new_type,
                    props: props,
                    data: data,
                }
            }
            _ => gene
        }
    }
}
