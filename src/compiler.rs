use std::collections::{BTreeMap};

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
        return self.module.clone();
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
}

#[derive(Clone)]
pub enum Instruction {
    /// Save Value to default register
    Default(Value),
}
