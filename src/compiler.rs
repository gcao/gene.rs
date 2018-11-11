use std::collections::{BTreeMap};
use std::fmt;

use super::types::Value;
use super::utils::new_uuidv4;

pub struct Compiler {
    module: Module,
}

impl Compiler {
    pub fn new() -> Compiler {
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

        self.module.blocks.insert(block_id, block);
        return self.module.clone();
    }

    fn compile_(&mut self, block: &mut Block, ast: Value) {
        match ast {
            Value::Integer(v) => {
                (*block).add_instr(Instruction::TODO(ast.to_string()));
            },
            _ => {
                (*block).add_instr(Instruction::TODO(ast.to_string()));
            }
        }
    }
}

#[derive(Clone)]
pub struct Module {
    pub id: String,
    pub blocks: BTreeMap<String, Block>,
}

impl Module {
    pub fn new() -> Module {
        return Module {
            id: new_uuidv4(),
            blocks: BTreeMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct Block {
    pub id: String,
    pub name: String,
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn new(name: String) -> Block {
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

#[derive(Clone)]
pub enum Instruction {
    /// Not supported code should compile to TODO instruction with a message
    TODO(String),
    Init,
    /// Save Value to default register
    Default(Value),
    CallEnd,
}

impl fmt::Display for Instruction {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("(")?;
        match &self {
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