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

pub struct VirtualMachine {
    registers_store: HashMap<String, Rc<RefCell<Registers>>>,
    pos: usize,
    // app: Application,
    code_manager: CodeManager,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            registers_store: HashMap::new(),
            pos: 0,
            // app: Application::new(),
            code_manager: CodeManager::new(),
        }
    }

    pub fn load_module(&mut self, module: &Module) -> Rc<RefCell<dyn Any>> {
        let block = module.get_default_block();

        module.blocks.values().for_each(|block| {
            let id = block.id.clone();
            self.code_manager.set_block(id, block.clone());
        });

        self.process(block.clone())
    }

    pub fn process(&mut self, mut block: Rc<Block>) -> Rc<RefCell<dyn Any>> {
        let start_time = Instant::now();

        let root_context = Context::root();
        let registers_temp = Registers::new(Rc::new(RefCell::new(root_context)));
        let id = registers_temp.id.clone();
        let mut registers_ = Rc::new(RefCell::new(registers_temp));
        self.registers_store.insert(id, registers_.clone());

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
                    benchmarker.op_start("Default");

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.default = Rc::new(RefCell::new(v.clone()));

                    benchmarker.op_end();
                }

                Instruction::Save(reg, v) => {
                    benchmarker.op_start("Save");

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.insert(reg.clone(), Rc::new(RefCell::new(v.clone())));

                    benchmarker.op_end();
                }

                Instruction::CopyFromDefault(to) => {
                    benchmarker.op_start("CopyFromDefault");

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let default;
                    {
                        default = registers.default.clone();
                    }
                    registers.insert(to.clone(), default);

                    benchmarker.op_end();
                }

                Instruction::CopyToDefault(to) => {
                    benchmarker.op_start("CopyToDefault");

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.default = registers.data[to].clone();

                    benchmarker.op_end();
                }

                Instruction::DefMember(name) => {
                    benchmarker.op_start("DefMember");

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    let value = registers.default.clone();
                    {
                        let mut context = registers.context.borrow_mut();
                        context.def_member(name.clone(), value, VarType::SCOPE);
                    }

                    benchmarker.op_end();
                }

                Instruction::GetMember(name) => {
                    benchmarker.op_start("GetMember");

                    self.pos += 1;
                    let value = self.get_member(registers_.clone(), name.clone()).unwrap();
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.default = value;

                    benchmarker.op_end();
                }

                Instruction::SetMember(name) => {
                    benchmarker.op_start("SetMember");

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    let value = registers.default.clone();
                    {
                        let mut context = registers.context.borrow_mut();
                        context.set_member(name.clone(), value);
                    }

                    benchmarker.op_end();
                }

                Instruction::Jump(pos) => {
                    benchmarker.op_start("Jump");

                    self.pos = *pos as usize;

                    benchmarker.op_end();
                }

                Instruction::JumpIfFalse(pos) => {
                    benchmarker.op_start("JumpIfFalse");

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    let value_ = registers.default.borrow();
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

                    benchmarker.op_end();
                }

                Instruction::Break => {
                    benchmarker.op_start("Break");

                    self.pos += 1;
                    break_from_loop = true;

                    benchmarker.op_end();
                }

                Instruction::LoopStart => {
                    benchmarker.op_start("LoopStart");

                    self.pos += 1;

                    benchmarker.op_end();
                }

                Instruction::LoopEnd => {
                    benchmarker.op_start("LoopEnd");

                    self.pos += 1;

                    benchmarker.op_end();
                }

                Instruction::BinaryOp(op, first) => {
                    benchmarker.op_start("BinaryOp");

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let first = registers.data[first].clone();
                    let second = registers.default.clone();
                    let result = binary_op(op, first, second);
                    registers.default = result;

                    benchmarker.op_end();
                }

                Instruction::Init => {
                    benchmarker.op_start("Init");

                    self.pos += 1;

                    benchmarker.op_end();
                }

                Instruction::Function(name, args, body_id) => {
                    benchmarker.op_start("Function");

                    self.pos += 1;
                    let function_temp;
                    {
                        let registers_temp = registers_.clone();
                        let registers = registers_temp.borrow();
                        let mut context = registers.context.borrow_mut();
                        let function = Function::new(name.clone(), (*args).clone(), body_id.clone(), true, context.namespace.clone(), context.scope.clone());
                        function_temp = Rc::new(RefCell::new(function));
                        context.def_member(name.clone(), function_temp.clone(), VarType::NAMESPACE);
                    }
                    {
                        let registers_temp = registers_.clone();
                        let mut registers = registers_temp.borrow_mut();
                        registers.default = function_temp.clone();
                    }

                    benchmarker.op_end();
                }

                Instruction::Call(target_reg, args_reg, _options) => {
                    benchmarker.op_start("Call");

                    self.pos += 1;

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    let borrowed = registers.data[target_reg].borrow();
                    let target = borrowed.downcast_ref::<Function>().unwrap();

                    let mut new_scope = Scope::new(target.parent_scope.clone());

                    {
                        let args_ = registers.data[args_reg].borrow();
                        let args = args_.downcast_ref::<Vec<Rc<RefCell<Value>>>>().unwrap();

                        for matcher in target.args.data_matchers.iter() {
                            let arg_value = args[matcher.index].clone();
                            new_scope.def_member(matcher.name.clone(), arg_value);
                        }
                    }

                    let new_namespace = Namespace::new(target.parent_namespace.clone());
                    let new_context = Context::new(Rc::new(RefCell::new(new_namespace)), Rc::new(RefCell::new(new_scope)), None);
                    let mut new_registers = Registers::new(Rc::new(RefCell::new(new_context)));

                    let ret_addr = Address::new(block.id.clone(), self.pos);
                    new_registers.insert("caller".to_string(), Rc::new(RefCell::new(ret_addr)));
                    new_registers.insert("caller_registers".to_string(), Rc::new(RefCell::new(registers.id.clone())));

                    let id = new_registers.id.clone();
                    let registers = Rc::new(RefCell::new(new_registers));
                    registers_ = registers.clone();
                    self.registers_store.insert(id, registers);

                    block = self.code_manager.blocks[&target.body].clone();
                    self.pos = 0;

                    benchmarker.op_end();
                }

                Instruction::CallEnd => {
                    benchmarker.op_start("CallEnd");

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    if registers.data.contains_key("caller") {
                        let borrowed = registers.data["caller"].borrow();
                        let ret_addr = borrowed.downcast_ref::<Address>().unwrap();

                        block = self.code_manager.blocks[&ret_addr.block_id].clone();
                        self.pos = ret_addr.pos;

                        let registers_id_borrowed = registers.data["caller_registers"].borrow();
                        let registers_id = registers_id_borrowed.downcast_ref::<String>().unwrap();
                        let caller_registers_temp = self.registers_store[registers_id].clone();
                        registers_ = caller_registers_temp.clone();

                        let mut caller_registers = caller_registers_temp.borrow_mut();
                        // Save returned value in caller's default register
                        caller_registers.default = registers.default.clone();
                    } else {
                        self.pos += 1;
                    }

                    benchmarker.op_end();
                }

                Instruction::CreateArguments(reg) => {
                    benchmarker.op_start("CreateArguments");

                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let data = Vec::<Rc<RefCell<Value>>>::new();
                    registers.insert(reg.clone(), Rc::new(RefCell::new(data)));

                    benchmarker.op_end();
                }

                Instruction::GetItem(_reg, _index) => unimplemented!(),

                Instruction::SetItem(target_reg, index) => {
                    benchmarker.op_start("SetItem");

                    self.pos += 1;

                    let value;

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    {
                        let value_ = registers.default.borrow();
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

                    benchmarker.op_end();
                }

                Instruction::SetProp(target_reg, key) => {
                    benchmarker.op_start("SetProp");

                    self.pos += 1;

                    let value;

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    {
                        let value_ = registers.default.borrow();
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

                    benchmarker.op_end();
                }

                Instruction::Dummy => unimplemented!(),
            }
        }

        benchmarker.loop_end();
        println!("{}", benchmarker);
        // dbg!(benchmarker);

        let registers = registers_.borrow();
        let result = registers.default.clone();
        // dbg!(result.borrow().downcast_ref::<Value>().unwrap());

        println!("Execution time: {:.6} seconds", start_time.elapsed().as_nanos() as f64 / 1_000_000_000.);

        result
    }

    fn get_member(&self, registers: Rc<RefCell<Registers>>, name: String) -> Option<Rc<RefCell<dyn Any>>> {
        let registers_ = registers.borrow();
        let context = registers_.context.borrow();
        context.get_member(name)
    }
}

#[derive(Debug)]
pub struct Registers {
    pub id: String,
    pub default: Rc<RefCell<dyn Any>>,
    pub context: Rc<RefCell<Context>>,
    pub data: HashMap<String, Rc<RefCell<dyn Any>>>,
}

impl Registers {
    pub fn new(context: Rc<RefCell<Context>>) -> Self {
        let data = HashMap::new();
        Registers {
            id: new_uuidv4(),
            default: Rc::new(RefCell::new(0)), // Put a dummy value
            context,
            data,
        }
    }

    pub fn insert(&mut self, key: String, val: Rc<RefCell<dyn Any>>) {
        self.data.insert(key, val);
    }
}

fn binary_op<'a>(
    op: &'a str,
    first: Rc<RefCell<dyn Any>>,
    second: Rc<RefCell<dyn Any>>,
) -> Rc<RefCell<dyn Any>> {
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

    // pub fn get_block(&self, id: String) -> Rc<Block> {
    //     self.blocks[&id].clone()
    // }

    pub fn set_block(&mut self, id: String, block: Rc<Block>) {
        self.blocks.insert(id, block);
    }
}
