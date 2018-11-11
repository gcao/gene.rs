use std::any::Any;
use std::collections::{BTreeMap};

use super::types::Value;
use super::compiler::Module;

pub struct VirtualMachine {
    registers_mgr: RegistersManager,
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        return VirtualMachine {
            registers_mgr: RegistersManager::new(),
        };
    }

    pub fn process(&mut self, module: Module) -> Box<Any> {
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