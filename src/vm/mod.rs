mod types;

use std::any::Any;
use std::collections::{BTreeMap};

use super::types::Value;
use super::compiler::{Module, Block};

pub struct VirtualMachine {
    registers_mgr: RegistersManager,
    pos: i32,
    block: Option<Block>,
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        return VirtualMachine {
            registers_mgr: RegistersManager::new(),
            pos: 0,
            block: None,
        };
    }

    pub fn load_module(&mut self, mut module: Module) -> Box<Any> {
        let block = module.get_default_block();
        return self.process(block);
    }

    pub fn process(&mut self, block: Block) -> Box<Any> {
        return Box::new(0);
    }
}

pub struct Registers {
    id: String,
    data: BTreeMap<String, Box<Any>>,
}

pub struct RegistersManager {
    store: BTreeMap<String, Registers>,
}

impl RegistersManager {
    pub fn new() -> RegistersManager {
        return RegistersManager {
            store: BTreeMap::new(),
        };
    }
}
