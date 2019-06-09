pub mod types;

use std::ptr;
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;
use std::rc::Rc;

use self::types::*;
use super::compiler::{Block, Instruction, Module};
use super::types::Value;
use super::utils::new_uuidv4;

const DEFAULT_REG: &str = "default";
const CONTEXT_REG: &str = "context";

pub struct VirtualMachine {
    registers_store: BTreeMap<String, Rc<RefCell<Registers>>>,
    registers_id: String,
    pos: usize,
    block: Option<Rc<Block>>,
    app: Application,
    code_manager: CodeManager,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            registers_store: BTreeMap::new(),
            registers_id: "".into(),
            pos: 0,
            block: None,
            app: Application::new(),
            code_manager: CodeManager::new(),
        }
    }

    pub fn load_module(&mut self, module: &Module) -> Rc<RefCell<Any>> {
        let block = module.get_default_block();

        module.blocks.values().for_each(|block| {
            let id = block.id.clone();
            self.code_manager.set_block(id, block.clone());
        });

        self.process(block.clone())
    }

    pub fn process(&mut self, mut block: Rc<Block>) -> Rc<RefCell<Any>> {
        self.create_registers();

        {
            let root_context = Context::root();
            let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
            registers.insert(CONTEXT_REG.into(), Rc::new(RefCell::new(root_context)));
        }

        self.pos = 0;
        while self.pos < block.instructions.len() {
            let instr = &block.instructions[self.pos];
            dbg!(instr);
            match instr {
                Instruction::Default(v) => {
                    self.pos += 1;
                    let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
                    registers.insert(DEFAULT_REG.into(), Rc::new(RefCell::new(v.clone())));
                }

                Instruction::Copy(from, to) => {
                    self.pos += 1;
                    let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
                    let from_value = registers.data[from].clone();
                    registers.insert(to.clone(), from_value);
                }

                Instruction::DefMember(name, reg) => {
                    self.pos += 1;
                    let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
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
                    let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
                    registers.insert(DEFAULT_REG.into(), value);
                }

                Instruction::BinaryOp(op, first, second) => {
                    self.pos += 1;
                    let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
                    let first = registers.data[first].clone();
                    let second = registers.data[second].clone();
                    let result = binary_op(op, first, second);
                    registers.data.insert(DEFAULT_REG.into(), result);
                }

                Instruction::Init => {
                    self.pos += 1;
                    let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
                    registers.data.insert(CONTEXT_REG.into(), Rc::new(RefCell::new(Context::root())));
                }

                Instruction::Function(name, args, body_id) => {
                    self.pos += 1;
                    let function_temp;
                    {
                        let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
                        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
                        let context = borrowed.downcast_mut::<Context>().unwrap();
                        let function = Function::new(name.clone(), (*args).clone(), body_id.clone(), true, context.namespace.clone(), context.scope.clone());
                        function_temp = Rc::new(RefCell::new(function));
                        context.def_member(name.clone(), function_temp.clone(), VarType::NAMESPACE);
                    }
                    {
                        let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
                        registers.data.insert(DEFAULT_REG.into(), function_temp.clone());
                    }
                }

                Instruction::Call(options) => {
                    self.pos += 1;

                    let registers_temp = self.registers_store[&self.registers_id].clone();
                    let registers = registers_temp.borrow();
                    let borrowed = registers.data[DEFAULT_REG].borrow();
                    let target = borrowed.downcast_ref::<Function>().unwrap();

                    let args_reg = options["args"].clone();
                    let args = registers.data.get(args_reg.downcast_ref::<String>().unwrap());

                    let ret_addr = Address::new(block.id.clone(), self.pos);
                    let mut borrowed_ctx = registers.data[CONTEXT_REG].borrow_mut();

                    let caller_context = borrowed_ctx.downcast_mut::<Context>().unwrap();
                    let caller_scope = caller_context.scope.clone();
                    let caller_namespace = caller_context.namespace.clone();

                    let mut new_registers = Registers::new();

                    let new_scope = Scope::new(caller_scope);
                    let new_namespace = target.namespace.clone();

                    let new_context = Context::new(new_namespace, Rc::new(RefCell::new(new_scope)), None);
                    new_registers.insert("context".to_string(), Rc::new(RefCell::new(new_context)));

                    self.registers_store.insert(new_registers.id.clone(), Rc::new(RefCell::new(new_registers)));

                    block = self.code_manager.get_block(target.body.to_string()).clone();
                    self.pos = 0;
                }

                Instruction::CallEnd => {
                    self.pos += 1;
                    // TODO: return to caller
                }

                Instruction::Function(name, args, body) => {
                    self.pos += 1;
                    let registers_temp = self.registers_store[&self.registers_id].clone();
                    let registers = registers_temp.borrow();
                    let _context = registers.data[CONTEXT_REG].borrow();
                    let context = _context.downcast_ref::<Context>().unwrap();
                    let f = Function::new(name.to_string(), (*args).clone(), body.to_string(), false, context.namespace.clone(), context.scope.clone());

                    let registers_mut_temp = self.registers_store[&self.registers_id].clone();
                    let mut registers_mut = registers_mut_temp.borrow_mut();
                    registers_mut.data.insert(DEFAULT_REG.into(), Rc::new(RefCell::new(f)));
                }

                _ => {
                    self.pos += 1;
                    println!("Unimplemented instruction: {}", instr)
                }
            }
        }

        let registers = self.registers_store[&self.registers_id].borrow();
        let result = registers.data[DEFAULT_REG].clone();
        dbg!(result.borrow().downcast_ref::<Value>().unwrap());
        result
    }

    pub fn create_registers(&mut self) {
        let registers = Registers::new();
        let id = registers.id.clone();
        self.registers_id = id.clone();
        self.registers_store.insert(id, Rc::new(RefCell::new(registers)));
    }

    fn get_member(&mut self, name: String) -> Option<Rc<RefCell<Any>>> {
        let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
        let context = borrowed.downcast_mut::<Context>().unwrap();
        context.get_member(name).map(|val| val.clone())
    }
}

#[derive(Debug)]
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

pub struct Address {
    pub block_id: String,
    pub pos: usize,
}

impl Address {
    pub fn new(block_id: String, pos: usize) -> Self {
        Address {
            block_id,
            pos,
        }
    }
}

pub struct CodeManager {
    pub blocks: BTreeMap<String, Rc<Block>>,
}

impl CodeManager {
    pub fn new() -> Self {
        CodeManager {
            blocks: BTreeMap::new(),
        }
    }

    pub fn get_block(&self, id: String) -> Rc<Block> {
        self.blocks[&id].clone()
    }

    pub fn set_block(&mut self, id: String, block: Rc<Block>) {
        self.blocks.insert(id, block);
    }
}