extern crate rand;

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
        return Compiler {
            module: Module::new(),
        };
    }

    pub fn compile(&mut self, ast: Value) -> Module {
        let mut block = Block::new("__default__".into());
        let block_id = block.id.clone();
        block.add_instr(Instruction::Init);
        self.compile_(&mut block, ast);
        block.add_instr(Instruction::CallEnd);

        println!("Block: {}", block);

        self.module.set_default_block(block);
        return self.module.clone();
    }

    fn compile_(&mut self, block: &mut Block, ast: Value) {
        match ast {
            Value::Symbol(s) => {
                // TODO: compile into variable
            },
            Value::Array(v) => {
                // TODO: compile individual elements
                (*block).add_instr(Instruction::Default(Value::Array(v)));
            },
            Value::Map(_) => {
                // TODO: compile individual values
                (*block).add_instr(Instruction::Default(ast));
            },
            Value::Gene(v) => {
                self.compile_gene(block, v);
            },
            _ => {
                (*block).add_instr(Instruction::Default(ast));
            }
        }
    }

    fn compile_gene(&mut self, block: &mut Block, gene: Gene) {
        let Gene {_type, mut data, ..} = gene;

        if *_type == Value::Symbol("var".into()) {
            match data.get(0).unwrap() {
                box Value::Symbol(name) => {
                    match data.get(1).unwrap() {
                        box value => {
                            self.compile_(block, value.clone());
                            (*block).add_instr(Instruction::Define(name.clone(), "default".into()));
                        }
                    }
                },
                _ => unimplemented!()
            }
        }
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
        return Module {
            id: new_uuidv4(),
            blocks: BTreeMap::new(),
            default_block_id: "".into(),
        }
    }

    pub fn set_default_block(&mut self, block: Block) {
        self.default_block_id = block.id.clone();
        self.blocks.insert(block.id.clone(), block);
    }

    pub fn get_default_block(&mut self) -> Block {
        return self.blocks[&self.default_block_id].clone();
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
        return Block {
            id: new_uuidv4(),
            name: name,
            instructions: instructions,
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
    /// Not supported code should compile to TODO instruction with a message
    TODO(String),
    Init,
    /// Save Value to default register
    Default(Value),
    Define(String, String),
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
            Instruction::Define(name, reg) => {
                fmt.write_str("Define ")?;
                fmt.write_str(&name.to_string())?;
                fmt.write_str(" ")?;
                fmt.write_str(&reg.to_string())?;
            }
            Instruction::CallEnd => {
                fmt.write_str("CallEnd")?;
            }
            Instruction::TODO(_) => {
                fmt.write_str("TODO")?;
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
