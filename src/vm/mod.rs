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

const CALLER_REG: u16 = 0;
const CALLER_REGISTERS_REG: u16 = 1;

pub struct VirtualMachine {
    registers_store: RegistersStore,
    pos: usize,
    // app: Application,
    code_manager: CodeManager,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            registers_store: RegistersStore::new(),
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
        let mut registers_ = self.registers_store.get(Rc::new(RefCell::new(root_context)));

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
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.default = Rc::new(RefCell::new(v.clone()));
                }

                Instruction::Save(reg, v) => {
                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.insert(*reg, Rc::new(RefCell::new(v.clone())));
                }

                Instruction::CopyFromDefault(to) => {
                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let default;
                    {
                        default = registers.default.clone();
                    }
                    registers.insert(to.clone(), default);

                }

                Instruction::CopyToDefault(to) => {
                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.default = registers.get(to);
                }

                Instruction::DefMember(name) => {
                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    let value = registers.default.clone();
                    {
                        let mut context = registers.context.borrow_mut();
                        context.def_member(name.clone(), value, VarType::SCOPE);
                    }
                }

                Instruction::GetMember(name) => {
                    self.pos += 1;
                    let value = self.get_member(registers_.clone(), name).unwrap();
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    registers.default = value;
                }

                Instruction::SetMember(name) => {
                    self.pos += 1;
                    let value;
                    {
                        let registers_temp = registers_.clone();
                        let registers = registers_temp.borrow();
                        value = registers.default.clone();
                    }
                    self.set_member(registers_.clone(), name.clone(), value);
                }

                Instruction::Jump(pos) => {
                    self.pos = *pos as usize;
                }

                Instruction::JumpIfFalse(pos) => {
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
                }

                Instruction::Break => {
                    self.pos += 1;
                    break_from_loop = true;
                }

                Instruction::LoopStart => {
                    self.pos += 1;
                }

                Instruction::LoopEnd => {
                    self.pos += 1;
                }

                Instruction::BinaryOp(op, first) => {
                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let first = registers.get(first);
                    let second = registers.default.clone();
                    let result = binary_op(op, first, second);
                    registers.default = result;
                }

                Instruction::Init => {
                    self.pos += 1;
                }

                Instruction::Function(name, args, body_id) => {
                    self.pos += 1;
                    let function_temp;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    {
                        let mut context = registers.context.borrow_mut();
                        let function = Function::new(name.clone(), (*args).clone(), body_id.clone(), true, context.namespace.clone(), context.scope.clone());
                        function_temp = Rc::new(RefCell::new(function));
                        context.def_member(name.clone(), function_temp.clone(), VarType::NAMESPACE);
                    }
                    registers.default = function_temp.clone();
                }

                Instruction::Call(target_reg, args_reg, _options) => {
                    self.pos += 1;

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    let borrowed_ = registers.get(target_reg);
                    let borrowed = borrowed_.borrow();
                    let target = borrowed.downcast_ref::<Function>().unwrap();

                    let mut new_scope = Scope::new(target.parent_scope.clone());

                    {
                        let args_temp = registers.get(args_reg);
                        let args_ = args_temp.borrow();
                        let args = args_.downcast_ref::<Vec<Rc<RefCell<Value>>>>().unwrap();

                        for matcher in target.args.data_matchers.iter() {
                            let arg_value = args[matcher.index].clone();
                            new_scope.def_member(matcher.name.clone(), arg_value);
                        }
                    }

                    let new_namespace = Namespace::new(target.parent_namespace.clone());
                    let new_context = Context::new(Rc::new(RefCell::new(new_namespace)), Rc::new(RefCell::new(new_scope)), None);
                    let new_registers_ = self.registers_store.get(Rc::new(RefCell::new(new_context)));
                    registers_ = new_registers_.clone();
                    let mut new_registers = new_registers_.borrow_mut();

                    let ret_addr = Address::new(block.id.clone(), self.pos);
                    new_registers.insert(CALLER_REG, Rc::new(RefCell::new(ret_addr)));
                    new_registers.insert(CALLER_REGISTERS_REG, Rc::new(RefCell::new(registers.id.clone())));

                    block = self.code_manager.blocks[&target.body].clone();
                    self.pos = 0;
                }

                Instruction::CallEnd => {
                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    let borrowed_ = registers.get(&CALLER_REG);
                    let borrowed = borrowed_.borrow();
                    if let Some(ret_addr) = borrowed.downcast_ref::<Address>() {
                        block = self.code_manager.blocks[&ret_addr.block_id].clone();
                        self.pos = ret_addr.pos;

                        let borrowed_ = registers.get(&CALLER_REGISTERS_REG);
                        let registers_id_borrowed = borrowed_.borrow();
                        let registers_id = registers_id_borrowed.downcast_ref::<String>().unwrap();
                        let caller_registers = self.registers_store.find(registers_id);
                        self.registers_store.free(&registers.id);
                        registers_ = caller_registers.clone();

                        // Save returned value in caller's default register
                        caller_registers.borrow_mut().default = registers.default.clone();
                    } else {
                        self.pos += 1;
                    }
                }

                Instruction::CreateArguments(reg) => {
                    self.pos += 1;
                    let registers_temp = registers_.clone();
                    let mut registers = registers_temp.borrow_mut();
                    let data = Vec::<Rc<RefCell<Value>>>::new();
                    registers.insert(reg.clone(), Rc::new(RefCell::new(data)));
                }

                Instruction::GetItem(_reg, _index) => unimplemented!(),

                Instruction::SetItem(target_reg, index) => {
                    self.pos += 1;

                    let value;

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    {
                        let value_ = registers.default.borrow();
                        value = value_.downcast_ref::<Value>().unwrap().clone();
                    }
                    let target_temp = registers.get(target_reg);
                    let mut target_ = target_temp.borrow_mut();
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

                Instruction::SetProp(target_reg, key) => {
                    self.pos += 1;

                    let value;

                    let registers_temp = registers_.clone();
                    let registers = registers_temp.borrow();
                    {
                        let value_ = registers.default.borrow();
                        value = value_.downcast_ref::<Value>().unwrap().clone();
                    }
                    let target_temp = registers.get(target_reg);
                    let mut target_ = target_temp.borrow_mut();
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
            }
        }

        let registers = registers_.borrow();
        let result = registers.default.clone();
        // dbg!(result.borrow().downcast_ref::<Value>().unwrap());

        println!("Execution time: {:.6} seconds", start_time.elapsed().as_nanos() as f64 / 1_000_000_000.);

        result
    }

    #[inline]
    fn get_member(&self, registers: Rc<RefCell<Registers>>, name: &str) -> Option<Rc<RefCell<dyn Any>>> {
        let registers_ = registers.borrow();
        let context = registers_.context.borrow();
        context.get_member(name)
    }

    #[inline]
    fn set_member(&self, registers: Rc<RefCell<Registers>>, name: String, value: Rc<RefCell<dyn Any>>) {
        let registers_ = registers.borrow();
        let mut context = registers_.context.borrow_mut();
        context.set_member(name.clone(), value.clone());
    }
}

#[derive(Debug)]
pub struct Registers {
    pub id: String,
    pub default: Rc<RefCell<dyn Any>>,
    pub context: Rc<RefCell<Context>>,
    pub cache: [Rc<RefCell<dyn Any>>; 16],
    pub store: HashMap<u16, Rc<RefCell<dyn Any>>>,
    // pub members_cache: HashMap<String, Rc<RefCell<dyn Any>>>,
}

impl Registers {
    pub fn new(context: Rc<RefCell<Context>>) -> Self {
        let dummy = Rc::new(RefCell::new(0));

        Registers {
            id: new_uuidv4(),
            default: dummy.clone(),
            context,
            cache: [
                dummy.clone(), dummy.clone(), dummy.clone(), dummy.clone(),
                dummy.clone(), dummy.clone(), dummy.clone(), dummy.clone(),
                dummy.clone(), dummy.clone(), dummy.clone(), dummy.clone(),
                dummy.clone(), dummy.clone(), dummy.clone(), dummy.clone(),
            ],
            store: HashMap::new(),
            // members_cache: HashMap::new(),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
    }

    #[inline]
    pub fn insert(&mut self, key: u16, val: Rc<RefCell<dyn Any>>) {
        if key < 16 {
            self.cache[key as usize] = val;
        } else {
            self.store.insert(key, val);
        }
    }

    #[inline]
    pub fn get(&self, key: &u16) -> Rc<RefCell<dyn Any>> {
        if *key < 16 {
            self.cache[*key as usize].clone()
        } else {
            self.store[key].clone()
        }
     }
}

pub struct RegistersStore {
    cache: HashMap<String, Rc<RefCell<Registers>>>,
    freed: Vec<String>,
}

impl RegistersStore {
    pub fn new() -> Self {
        RegistersStore {
            cache: HashMap::new(),
            freed: Vec::new(),
        }
    }

    #[inline]
    pub fn get(&mut self, context: Rc<RefCell<Context>>) -> Rc<RefCell<Registers>> {
        if self.freed.len() > 0 {
            let id = self.freed.pop().unwrap();
            let registers = self.cache.get(&id).unwrap();
            {
                registers.borrow_mut().reset();
                registers.borrow_mut().context = context;
            }
            registers.clone()
        } else {
            let registers = Registers::new(context.clone());
            let id = registers.id.clone();
            let wrapped = Rc::new(RefCell::new(registers));
            self.cache.insert(id.clone(), wrapped.clone());
            wrapped
        }
    }

    #[inline]
    pub fn find(&self, id: &str) -> Rc<RefCell<Registers>> {
        self.cache[id].clone()
    }

    #[inline]
    pub fn free(&mut self, id: &str) {
        self.freed.push(id.to_string());
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