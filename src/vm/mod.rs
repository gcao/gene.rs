mod types;

use std::any::Any;
use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::rc::Rc;

use self::types::*;
use super::compiler::{Block, Instruction, Module};
use super::types::Value;
use super::utils::new_uuidv4;

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

    pub fn load_module(&mut self, module: &Module) -> &Rc<RefCell<Any>> {
        let block = module.get_default_block();
        self.process(block.clone())
    }

    pub fn process(&mut self, block_: Rc<RefCell<Block>>) -> &Rc<RefCell<Any>> {
        let block = block_.borrow();
        self.create_registers();

        {
            let root_context = Context::root();
            let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
            registers
                .data
                .insert(CONTEXT_REG.into(), Rc::new(RefCell::new(root_context)));
        }

        self.pos = 0;
        while self.pos < block.instructions.len() {
            let instr = &block.instructions[self.pos];
            match instr {
                Instruction::Default(v) => {
                    self.pos += 1;
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                    registers.insert(DEFAULT_REG.into(), Rc::new(RefCell::new(v.clone())));
                }

                Instruction::Copy(from, to) => {
                    self.pos += 1;
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                    let from_value = registers.data[from].clone();
                    registers.insert(to.clone(), from_value);
                }

                Instruction::Define(name, reg) => {
                    self.pos += 1;
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                    let value;
                    {
                        value = registers.data[reg].clone();
                    }
                    {
                        let mut borrowed =
                            registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
                        let context = borrowed.downcast_mut::<Context>().unwrap();
                        context.def_member(name.clone(), value, VarType::SCOPE);
                    }
                }

                Instruction::GetMember(name) => {
                    self.pos += 1;
                    let value = self.get_member(name.clone()).unwrap();
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                    registers.insert(DEFAULT_REG.into(), value);
                }

                Instruction::BinaryOp(op, first, second) => {
                    self.pos += 1;
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                    let first = registers.data[first].clone();
                    let second = registers.data[second].clone();
                    let result = binary_op(op, first, second);
                    registers.data.insert(DEFAULT_REG.into(), result);
                }

                Instruction::Init => {
                    self.pos += 1;
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                    registers.data.insert(CONTEXT_REG.into(), Rc::new(RefCell::new(Context::root())));
                }

                Instruction::CallEnd => {
                    self.pos += 1;
                    // TODO: return to caller
                }

                Instruction::Function(name, body) => {
                    self.pos += 1;
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
                    let c = registers.data[CONTEXT_REG].clone();
                    let b = c.borrow();
                    let context = b.downcast_ref::<Context>().unwrap();
                    let f = Function::new(name.to_string(), body.to_string(), false, context.namespace.clone(), context.scope.clone());
                    registers.data.insert(DEFAULT_REG.into(), Rc::new(RefCell::new(f)));
                }

                Instruction::CallEnd => {
                    self.pos += 1;
                    // TODO: return to caller
                }

                Instruction::Function(name, body) => {
                    self.pos += 1;
                }

                _ => {
                    self.pos += 1;
                    println!("Unimplemented instruction: {}", instr)
                }
            }
        }

        let registers = &self.registers_store[&self.registers_id];
        let result = &registers.data[DEFAULT_REG];
        println!(
            "Result: {}",
            result.borrow().downcast_ref::<Value>().unwrap()
        );
        result
    }

    pub fn create_registers(&mut self) {
        let registers = Registers::new();
        let id = registers.id.clone();
        self.registers_id = id.clone();
        self.registers_store.insert(id, registers);
    }

    fn get_member(&mut self, name: String) -> Option<Rc<RefCell<Any>>> {
        let registers = self.registers_store.get_mut(&self.registers_id).unwrap();
        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
        let context = borrowed.downcast_mut::<Context>().unwrap();
        context.get_member(name).map(|val| val.clone())
    }
}

pub struct Registers {
    pub id: String,
    pub data: BTreeMap<String, Rc<RefCell<Any>>>,
}

impl Registers {
    pub fn new() -> Self {
        let data = BTreeMap::new();
        Registers {
            id: new_uuidv4(),
            data,
        }
    }

    pub fn insert(&mut self, key: String, val: Rc<RefCell<Any>>) {
        self.data.insert(key, val);
    }
}

fn binary_op<'a>(
    op: &'a str,
    first: Rc<RefCell<Any>>,
    second: Rc<RefCell<Any>>,
) -> Rc<RefCell<Any>> {
    let borrowed1 = first.borrow();
    let borrowed2 = second.borrow();
    let value1 = borrowed1.downcast_ref::<Value>().unwrap();
    let value2 = borrowed2.downcast_ref::<Value>().unwrap();
    match (value1, value2) {
        (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Integer(a + b))),
        _ => unimplemented!(),
    }
}
