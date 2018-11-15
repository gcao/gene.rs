mod types;

use std::any::Any;
use std::collections::{BTreeMap};

use super::types::Value;
use super::utils::new_uuidv4;
use super::compiler::{Module, Block, Instruction};

pub struct VirtualMachine {
    registers_store: BTreeMap<String, Registers>,
    registers_id: String,
    pos: usize,
    block: Option<Block>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        return VirtualMachine {
            registers_store: BTreeMap::new(),
            registers_id: "".into(),
            pos: 0,
            block: None,
        };
    }

    pub fn load_module(&mut self, mut module: Module) -> &Box<Any> {
        let block = module.get_default_block();
        return self.process(block);
    }

    pub fn process(&mut self, block: Block) -> &Box<Any> {
        self.create_registers();

        while self.pos < block.instructions.len() {
            let instr = &block.instructions[self.pos];
            match instr {
                Instruction::Default(v) => {
                    self.pos += 1;
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                    // println!("Value: {}", v);
                    registers.insert("default".into(), Box::new(v.clone()));
                }

                Instruction::CallEnd => {
                    self.pos += 1;
                    // TODO: return to caller
                }

                _ => {
                    self.pos += 1;
                    println!("Unimplemented instruction: {}", instr)
                }
            }
        }

        let registers = self.registers_store.get(&self.registers_id).unwrap();
        let result = registers.data.get("default".into()).unwrap();
        println!("Result: {}", (*result).downcast_ref::<Value>().unwrap());
        result
    }

    pub fn create_registers(&mut self) {
        let registers = Registers::new();
        let id = registers.id.clone();
        self.registers_id = id.clone();
        self.registers_store.insert(id, registers);
    }
}

pub struct Registers {
    pub id: String,
    pub data: BTreeMap<String, Box<Any>>,
}

impl Registers {
    pub fn new() -> Self {
        let data: BTreeMap<String, Box<Any>> =  BTreeMap::new();
        return Registers {
            id: new_uuidv4(),
            data: data,
        }
    }

    pub fn insert(&mut self, key: String, val: Box<Any>) {
        self.data.insert(key, val);
    }
}
