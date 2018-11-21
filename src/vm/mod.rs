mod types;

use std::any::Any;
use std::collections::{BTreeMap};
use std::rc::Rc;
use std::cell::{RefCell, RefMut};

use super::types::Value;
use super::utils::new_uuidv4;
use super::compiler::{Module, Block, Instruction};
use self::types::*;

const DEFAULT_REG: &str = "default";
const CONTEXT_REG: &str = "context";

pub struct VirtualMachine {
    registers_store: BTreeMap<String, Registers>,
    registers_id: String,
    pos: usize,
    block: Option<Block>,
    app: Application,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            registers_store: BTreeMap::new(),
            registers_id: "".into(),
            pos: 0,
            block: None,
            app: Application::new(),
        }
    }

    pub fn load_module(&mut self, mut module: Module) -> &Rc<RefCell<Any>> {
        let block = module.get_default_block();
        self.process(block)
    }

    pub fn process(&mut self, block: Block) -> &Rc<RefCell<Any>> {
        self.create_registers();

        {
            let root_context = Context::root();
            let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
            registers.data.insert(CONTEXT_REG.into(), Rc::new(RefCell::new(root_context)));
        }

        self.pos = 0;
        while self.pos < block.instructions.len() {
            let instr = &block.instructions[self.pos];
            match instr {
                Instruction::Default(v) => {
                    self.pos += 1;
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                    // println!("Value: {}", v);
                    registers.insert(DEFAULT_REG.into(), Rc::new(RefCell::new(v.clone())));
                }

                Instruction::Define(name, reg) => {
                    self.pos += 1;
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                    let value;
                    {
                        value = Rc::clone(registers.data.get(reg.into()).unwrap());
                    }
                    {
                        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
                        let context = borrowed.downcast_mut::<Context>().unwrap();
                        context.def_member(name.clone(), value, VarType::SCOPE);
                    }
                }

                Instruction::GetMember(name) => {
                    self.pos += 1;
                    let value;
                    {
                        let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
                        let context = borrowed.downcast_mut::<Context>().unwrap();
                        value = Rc::clone(&context.get_member(name.clone()).unwrap());
                    }
                    {
                        let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                        registers.insert(DEFAULT_REG.into(), value);
                    }
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
        let result = registers.data.get(DEFAULT_REG.into()).unwrap();
        println!("Result: {}", result.borrow().downcast_ref::<Value>().unwrap());
        result
    }

    pub fn create_registers(&mut self) -> () {
        let registers = Registers::new();
        let id = registers.id.clone();
        self.registers_id = id.clone();
        self.registers_store.insert(id, registers);
    }

    fn get_member(&mut self, name: String) -> Option<Rc<RefCell<Any>>> {
        let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
        let context = borrowed.downcast_mut::<Context>().unwrap();
        context.get_member(name).map(|val| Rc::clone(&val))
    }
}

pub struct Registers {
    pub id: String,
    pub data: BTreeMap<String, Rc<RefCell<Any>>>,
}

impl Registers {
    pub fn new() -> Self {
        let data =  BTreeMap::new();
        Registers {
            id: new_uuidv4(),
            data: data,
        }
    }

    pub fn insert(&mut self, key: String, val: Rc<RefCell<Any>>) -> () {
        self.data.insert(key, val);
    }
}
