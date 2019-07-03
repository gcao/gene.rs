pub mod types;

use std::ptr;
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::time::Instant;

use self::types::*;
use super::compiler::{Block, Instruction, Module};
use super::types::Value;
use super::utils::new_uuidv4;

use super::benchmarker::Benchmarker;

const DEFAULT_REG: &str = "default";
const CONTEXT_REG: &str = "context";

pub struct VirtualMachine {
    registers_store: HashMap<String, Rc<RefCell<Registers>>>,
    registers_id: String,
    pos: usize,
    // app: Application,
    code_manager: CodeManager,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            registers_store: HashMap::new(),
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
        let start_time = Instant::now();

        self.create_registers();

        let mut registers_ = self.registers_store.get(&self.registers_id).unwrap().clone();
        {
            let root_context = Context::root();
            let registers_temp = registers_.clone();
            let mut registers = registers_temp.borrow_mut();
            registers.insert(CONTEXT_REG.into(), Rc::new(RefCell::new(root_context)));
        }

        self.pos = 0;
        let mut break_from_loop = false;

        let mut benchmarker = Benchmarker::new();
        benchmarker.loop_start();

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

            benchmarker.report_loop();

            // println!("{: <20} {: >5} {}", block.name, self.pos, instr);
            // dbg!(instr);

            match instr {
                Instruction::Default(v) => {
                    benchmarker.default_time.report_start();

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.insert(DEFAULT_REG.into(), Rc::new(RefCell::new(v.clone())));

                    benchmarker.default_time.report_end();
                }

                Instruction::Save(reg, v) => {
                    benchmarker.save_time.report_start();

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.insert(reg.clone(), Rc::new(RefCell::new(v.clone())));

                    benchmarker.save_time.report_end();
                }

                Instruction::Copy(from, to) => {
                    benchmarker.copy_time.report_start();

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let from_value = registers.data[from].clone();
                    registers.insert(to.clone(), from_value);

                    benchmarker.copy_time.report_end();
                }

                Instruction::DefMember(name, reg) => {
                    benchmarker.def_member_time.report_start();

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let value;
                    {
                        value = registers.data[reg].clone();
                    }
                    {
                        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
                        let context = borrowed.downcast_mut::<Context>().unwrap();
                        context.def_member(name.clone(), value, VarType::SCOPE);
                    }

                    benchmarker.def_member_time.report_end();
                }

                Instruction::GetMember(name) => {
                    benchmarker.get_member_time.report_start();

                    self.pos += 1;
                    let value = self.get_member(registers_.clone(), name.clone()).unwrap();
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.insert(DEFAULT_REG.into(), value);

                    benchmarker.get_member_time.report_end();
                }

                Instruction::SetMember(name, reg) => {
                    benchmarker.set_member_time.report_start();

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let value;
                    {
                        value = registers.data[reg].clone();
                    }
                    {
                        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
                        let context = borrowed.downcast_mut::<Context>().unwrap();
                        context.set_member(name.clone(), value);
                    }

                    benchmarker.set_member_time.report_end();
                }

                Instruction::Jump(pos) => {
                    benchmarker.jump_time.report_start();

                    self.pos = *pos as usize;

                    benchmarker.jump_time.report_end();
                }

                Instruction::JumpIfFalse(pos) => {
                    benchmarker.jump_if_false_time.report_start();

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow_mut();
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

                    benchmarker.jump_if_false_time.report_end();
                }

                Instruction::Break => {
                    // benchmarker.break_time.report_start();

                    self.pos += 1;
                    break_from_loop = true;

                    // benchmarker.break_time.report_end();
                }

                Instruction::LoopStart => {
                    // benchmarker.default_time.report_start();

                    self.pos += 1;

                    // benchmarker.default_time.report_end();
                }

                Instruction::LoopEnd => {
                    // benchmarker.default_time.report_start();

                    self.pos += 1;

                    // benchmarker.default_time.report_end();
                }

                Instruction::BinaryOp(op, first, second) => {
                    benchmarker.binary_op_time.report_start();

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let first = registers.data[first].clone();
                    let second = registers.data[second].clone();
                    let result = binary_op(op, first, second);
                    registers.data.insert(DEFAULT_REG.into(), result);

                    benchmarker.binary_op_time.report_end();
                }

                Instruction::Init => {
                    benchmarker.init_time.report_start();

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.data.insert(CONTEXT_REG.into(), Rc::new(RefCell::new(Context::root())));

                    benchmarker.init_time.report_end();
                }

                Instruction::Function(name, args, body_id) => {
                    benchmarker.function_time.report_start();

                    self.pos += 1;
                    let function_temp;
                    {
                        let registers_temp = registers_.clone();
                        let mut registers = registers_temp.borrow_mut();
                        let mut borrowed = registers.data.get_mut(CONTEXT_REG).unwrap().borrow_mut();
                        let context = borrowed.downcast_mut::<Context>().unwrap();
                        let function = Function::new(name.clone(), (*args).clone(), body_id.clone(), true, context.namespace.clone(), context.scope.clone());
                        function_temp = Rc::new(RefCell::new(function));
                        context.def_member(name.clone(), function_temp.clone(), VarType::NAMESPACE);
                    }
                    {
                        let registers_temp = registers_.clone();
                        let mut registers = registers_temp.borrow_mut();
                        registers.data.insert(DEFAULT_REG.into(), function_temp.clone());
                    }

                    benchmarker.function_time.report_end();
                }

                Instruction::Call(target_reg, options) => {
                    benchmarker.call_time.report_start();

                    self.pos += 1;

                    let registers_temp = registers_.clone();
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

                    let id = new_registers.id.clone();
                    let registers = Rc::new(RefCell::new(new_registers));
                    registers_ = registers.clone();
                    self.registers_store.insert(id, registers);

                    block = self.code_manager.get_block(target.body.to_string()).clone();
                    self.pos = 0;

                    benchmarker.call_time.report_end();
                }

                Instruction::CallEnd => {
                    benchmarker.call_end_time.report_start();

                    let registers_temp = registers_.clone();
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
                        registers_ = caller_registers_temp.clone();
                        let mut caller_registers = caller_registers_temp.borrow_mut();

                        // Save returned value in default register
                        let default = registers.data[DEFAULT_REG].clone();
                        caller_registers.insert(DEFAULT_REG.into(), default);
                    } else {
                        self.pos += 1;
                    }

                    benchmarker.call_end_time.report_end();
                }

                Instruction::CreateArguments(reg) => {
                    benchmarker.create_arguments_time.report_start();

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let data = Vec::<Rc<RefCell<Value>>>::new();
                    registers.insert(reg.clone(), Rc::new(RefCell::new(data)));

                    benchmarker.create_arguments_time.report_end();
                }

                Instruction::GetItem(_reg, _index) => unimplemented!(),

                Instruction::SetItem(target_reg, index, value_reg) => {
                    benchmarker.set_item_time.report_start();

                    self.pos += 1;

                    let value;

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    {
                        let value_ = registers.data[value_reg].borrow();
                        value = value_.downcast_ref::<Value>().unwrap().clone();
                    }
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

                    benchmarker.set_item_time.report_end();
                }

                Instruction::SetProp(target_reg, key, value_reg) => {
                    // benchmarker.default_time.report_start();

                    self.pos += 1;

                    let value;

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    {
                        let value_ = registers.data[value_reg].borrow();
                        value = value_.downcast_ref::<Value>().unwrap().clone();
                    }
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

                    // benchmarker.default_time.report_end();
                }

                Instruction::Dummy => unimplemented!(),
            }
        }

        benchmarker.loop_end();
        println!("{}", benchmarker);
        // dbg!(benchmarker);

        let registers = registers_.borrow();
        let result = registers.data[DEFAULT_REG].clone();
        // dbg!(result.borrow().downcast_ref::<Value>().unwrap());

        println!("Execution time: {:.6} seconds", start_time.elapsed().as_nanos() as f64 / 1_000_000_000.);

        result
    }

    pub fn create_registers(&mut self) {
        let registers = Registers::new();
        let id = registers.id.clone();
        self.registers_id = id.clone();
        self.registers_store.insert(id, Rc::new(RefCell::new(registers)));
    }

    fn get_member(&self, registers: Rc<RefCell<Registers>>, name: String) -> Option<Rc<RefCell<Any>>> {
        let registers_ = registers.borrow();
        let mut borrowed = registers_.data[CONTEXT_REG].borrow_mut();
        let context = borrowed.downcast_mut::<Context>().unwrap();
        context.get_member(name)
    }
}

#[derive(Debug)]
pub struct Registers {
    pub id: String,
    pub data: HashMap<String, Rc<RefCell<Any>>>,
}

impl Registers {
    pub fn new() -> Self {
        let data = HashMap::new();
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
    pub blocks: HashMap<String, Rc<Block>>,
}

impl CodeManager {
    pub fn new() -> Self {
        CodeManager {
            blocks: HashMap::new(),
        }
    }

    pub fn get_block(&self, id: String) -> Rc<Block> {
        self.blocks[&id].clone()
    }

    pub fn set_block(&mut self, id: String, block: Rc<Block>) {
        self.blocks.insert(id, block);
    }
}