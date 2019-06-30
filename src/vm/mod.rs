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
    // app: Application,
    code_manager: CodeManager,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            registers_store: BTreeMap::new(),
            registers_id: "".into(),
            pos: 0,
            // app: Application::new(),
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
        let mut break_from_loop = false;
        while self.pos < block.instructions.len() {
            let instr = &block.instructions[self.pos];

            // Handle break from loop
            if break_from_loop {
                self.pos += 1;
                match instr {
                    Instruction::LoopEnd => {
                        break_from_loop = false;
                    }
                    _ => {
                        continue;
                    }
                }
            }

            // println!("{: <20} {: >5} {}", block.name, self.pos, instr);
            // dbg!(instr);
            match instr {
                Instruction::Default(v) => {
                    self.pos += 1;
                    let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
                    registers.insert(DEFAULT_REG.into(), Rc::new(RefCell::new(v.clone())));
                }

                Instruction::Save(reg, v) => {
                    self.pos += 1;
                    let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
                    registers.insert(reg.clone(), Rc::new(RefCell::new(v.clone())));
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
                        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
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

                Instruction::SetMember(name, reg) => {
                    self.pos += 1;
                    let mut registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow_mut();
                    let value;
                    {
                        value = registers.data[reg].clone();
                    }
                    {
                        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
                        let context = borrowed.downcast_mut::<Context>().unwrap();
                        context.set_member(name.clone(), value);
                    }
                }

                Instruction::Jump(pos) => {
                    self.pos = *pos as usize;
                }

                Instruction::JumpIfFalse(pos) => {
                    let registers = self.registers_store.get_mut(&self.registers_id).unwrap().borrow();
                    let value_ = registers.data[DEFAULT_REG].borrow();
                    let value = value_.downcast_ref::<Value>().unwrap();
                    match value {
                        Value::Boolean(b) => {
                            if *b {
                                self.pos += 1;
                            } else {
                                self.pos = *pos as usize;
                            }
                        }
                        _ => unimplemented!()
                    }
                }

                Instruction::Break => {
                    self.pos += 1;
                    break_from_loop = true;
                }

                Instruction::LoopStart | Instruction::LoopEnd => {
                    self.pos += 1;
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

                Instruction::Call(target_reg, options) => {
                    self.pos += 1;

                    let registers_temp = self.registers_store[&self.registers_id].clone();
                    let registers = registers_temp.borrow();
                    let borrowed = registers.data[target_reg].borrow();
                    let target = borrowed.downcast_ref::<Function>().unwrap();

                    let args_reg = options["args"].clone();
                    let args_ = registers.data.get(args_reg.downcast_ref::<String>().unwrap()).unwrap();

                    let ret_addr = Address::new(block.id.clone(), self.pos);
                    // let mut borrowed_ctx = registers.data[CONTEXT_REG].borrow_mut();

                    // let caller_context = borrowed_ctx.downcast_mut::<Context>().unwrap();
                    // let caller_scope = caller_context.scope.clone();

                    let mut new_registers = Registers::new();
                    new_registers.insert("caller".to_string(), Rc::new(RefCell::new(ret_addr)));
                    new_registers.insert("caller_registers".to_string(), Rc::new(RefCell::new(registers.id.clone())));

                    let mut new_scope = Scope::new(target.scope.clone());
                    let new_namespace = target.namespace.clone();

                    {
                        let borrowed = args_.borrow();
                        let args = borrowed.downcast_ref::<Vec<Rc<RefCell<Value>>>>().unwrap();

                        for matcher in target.args.data_matchers.iter() {
                            // TODO: define members for arguments
                            let arg_value = args[matcher.index].clone();
                            new_scope.def_member(matcher.name.clone(), arg_value);
                        }
                    }

                    let new_context = Context::new(new_namespace, Rc::new(RefCell::new(new_scope)), None);
                    new_registers.insert("context".to_string(), Rc::new(RefCell::new(new_context)));
                    self.registers_id = new_registers.id.clone();

                    self.registers_store.insert(new_registers.id.clone(), Rc::new(RefCell::new(new_registers)));

                    block = self.code_manager.get_block(target.body.to_string()).clone();
                    self.pos = 0;
                }

                Instruction::CallEnd => {
                    let registers_temp = self.registers_store[&self.registers_id].clone();
                    let registers = registers_temp.borrow();
                    if registers.data.contains_key("caller") {
                        let borrowed = registers.data["caller"].borrow();
                        let ret_addr = borrowed.downcast_ref::<Address>().unwrap();

                        block = self.code_manager.get_block(ret_addr.block_id.to_string()).clone();
                        self.pos = ret_addr.pos;

                        let registers_borrowed = registers.data["caller_registers"].borrow();
                        let registers_id = registers_borrowed.downcast_ref::<String>().unwrap();
                        self.registers_id = registers_id.clone();

                        let caller_registers_temp = self.registers_store[registers_id].clone();
                        let mut caller_registers = caller_registers_temp.borrow_mut();

                        // Save returned value in default register
                        let default = registers.data["default"].clone();
                        caller_registers.insert("default".to_string(), default);
                    } else {
                        self.pos += 1;
                    }
                }

                Instruction::CreateArguments(reg) => {
                    self.pos += 1;
                    let registers_ = self.registers_store[&self.registers_id].clone();
                    let mut registers = registers_.borrow_mut();
                    let data = Vec::<Rc<RefCell<Value>>>::new();
                    registers.insert(reg.clone(), Rc::new(RefCell::new(data)));
                }

                Instruction::GetItem(_reg, _index) => unimplemented!(),

                Instruction::SetItem(target_reg, index, value_reg) => {
                    self.pos += 1;

                    let value;

                    let registers_ = self.registers_store[&self.registers_id].clone();
                    {
                        let registers = registers_.borrow();
                        let value_ = registers.data[value_reg].borrow();
                        value = value_.downcast_ref::<Value>().unwrap().clone();
                    }
                    let registers = registers_.borrow();
                    let mut target_ = registers.data[target_reg].borrow_mut();
                    if let Some(args) = target_.downcast_mut::<Vec<Rc<RefCell<Value>>>>() {
                        while *index >= args.len() {
                            args.push(Rc::new(RefCell::new(Value::Void)));
                        }
                        args[*index] = Rc::new(RefCell::new(value));
                    } else if let Some(args) = target_.downcast_mut::<Value>() {
                        match args {
                            Value::Array(arr) => {
                                while *index >= arr.len() {
                                    arr.push(Value::Void);
                                }
                                arr[*index] = value.clone();
                            }
                            _ => unimplemented!()
                        }
                    } else {
                        unimplemented!();
                    }
                }

                Instruction::SetProp(target_reg, key, value_reg) => {
                    self.pos += 1;

                    let value;

                    let registers_ = self.registers_store[&self.registers_id].clone();
                    {
                        let registers = registers_.borrow();
                        let value_ = registers.data[value_reg].borrow();
                        value = value_.downcast_ref::<Value>().unwrap().clone();
                    }
                    let registers = registers_.borrow();
                    let mut target_ = registers.data[target_reg].borrow_mut();
                    if let Some(v) = target_.downcast_mut::<Value>() {
                        match v {
                            Value::Map(map) => {
                                map.insert(key.clone(), value);
                            }
                            _ => unimplemented!()
                        }
                    } else {
                        unimplemented!();
                    }
                }

                Instruction::Dummy => unimplemented!(),

                // _ => unimplemented!()
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
    match op {
        "+" => {
            match (value1, value2) {
                (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Integer(a + b))),
                _ => unimplemented!()
            }
        }
        "-" => {
            match (value1, value2) {
                (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Integer(a - b))),
                _ => unimplemented!()
            }
        }
        "*" => {
            match (value1, value2) {
                (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Integer(a * b))),
                _ => unimplemented!()
            }
        }
        "/" => {
            match (value1, value2) {
                (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Integer(a / b))),
                _ => unimplemented!()
            }
        }
        "<" => {
            match (value1, value2) {
                (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Boolean(a < b))),
                _ => unimplemented!()
            }
        }
        "<=" => {
            match (value1, value2) {
                (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Boolean(a <= b))),
                _ => unimplemented!()
            }
        }
        ">" => {
            match (value1, value2) {
                (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Boolean(a > b))),
                _ => unimplemented!()
            }
        }
        ">=" => {
            match (value1, value2) {
                (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Boolean(a >= b))),
                _ => unimplemented!()
            }
        }
        "==" => {
            match (value1, value2) {
                (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Boolean(a == b))),
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
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