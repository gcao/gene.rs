pub mod types;

use std::ptr;
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use self::types::*;
use super::compiler::{Block, Instruction, Module};
use super::types::Value;
use super::utils::new_uuidv4;

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

        let mut registers_id;
        {
            let root_context = Context::root();
            let registers = self.registers_store.get(Rc::new(RefCell::new(root_context)));
            registers_id = registers.id;
        }

        self.pos = 0;
        let mut break_from_loop = false;
        let mut immature_break;

        // Use two level loop to separate instructions that change registers and those that don't
        // TODO: clean up and document logic
        while self.pos < block.instructions.len() {
            immature_break = false;

            {
                let mut registers = self.registers_store.find(registers_id);

                while self.pos < block.instructions.len() {
                    let instr = &block.instructions[self.pos];

                    // {
                    //     let instr = &block.instructions[self.pos];
                    //     println!("{: >5} {}", self.pos, instr);
                    // }

                    // Handle break from loop
                    if break_from_loop {
                        match instr {
                            Instruction::LoopEnd => {
                                break_from_loop = false;
                            }
                            _ => {
                                self.pos += 1;
                                continue;
                            }
                        }
                    }

                    match instr {
                        Instruction::Default(v) => {
                            self.pos += 1;
                            registers.default = Rc::new(RefCell::new(v.clone()));
                        }
                        Instruction::Save(reg, v) => {
                            self.pos += 1;
                            registers.insert(*reg, Rc::new(RefCell::new(v.clone())));
                        }
                        Instruction::CopyFromDefault(to) => {
                            self.pos += 1;
                            registers.insert(to.clone(), registers.default.clone());
                        }
                        Instruction::CopyToDefault(to) => {
                            self.pos += 1;
                            registers.default = registers.get(*to);
                        }
                        Instruction::DefMember(name) => {
                            self.pos += 1;
                            let value = registers.default.clone();
                            let mut context = registers.context.borrow_mut();
                            context.def_member(name.clone(), value, VarType::SCOPE);
                        }
                        Instruction::DefMemberInScope(name) => {
                            self.pos += 1;
                            let value = registers.default.clone();
                            let context = registers.context.borrow();
                            let mut scope = context.scope.borrow_mut();
                            scope.def_member(name.clone(), value);
                        }
                        Instruction::GetMember(name) => {
                            self.pos += 1;
                            let value = registers.get_member(name).unwrap();
                            registers.default = value;
                        }
                        Instruction::GetMemberInScope(name) => {
                            self.pos += 1;
                            let context = registers.context.borrow();
                            let value = context.scope.borrow().get_member(name).unwrap();
                            registers.default = value;
                        }
                        Instruction::SetMember(name) => {
                            self.pos += 1;
                            registers.set_member(name.clone(), registers.default.clone());
                        }
                        Instruction::SetMemberInScope(name) => {
                            self.pos += 1;
                            let context = registers.context.borrow();
                            let mut scope = context.scope.borrow_mut();
                            scope.set_member(name.clone(), registers.default.clone());
                        }
                        Instruction::Jump(pos) => {
                            self.pos = *pos as usize;
                        }
                        Instruction::JumpIfFalse(pos) => {
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
                        Instruction::Add(first) => {
                            self.pos += 1;
                            let first = registers.get(*first);
                            let second = registers.default.clone();
                            let result = add(first, second);
                            registers.default = result;
                        }
                        Instruction::Sub(first) => {
                            self.pos += 1;
                            let first = registers.get(*first);
                            let second = registers.default.clone();
                            let result = sub(first, second);
                            registers.default = result;
                        }
                        Instruction::Lt(first) => {
                            self.pos += 1;
                            let first = registers.get(*first);
                            let second = registers.default.clone();
                            let result = lt(first, second);
                            registers.default = result;
                        }
                        Instruction::Gt(first) => {
                            self.pos += 1;
                            let first = registers.get(*first);
                            let second = registers.default.clone();
                            let result = gt(first, second);
                            registers.default = result;
                        }
                        Instruction::Init => {
                            self.pos += 1;
                        }
                        Instruction::Function(name, args, body_id) => {
                            self.pos += 1;
                            let function_temp;
                            {
                                let context = registers.context.borrow_mut();
                                let function = Function::new(name.clone(), (*args).clone(), body_id.clone(), true, context.namespace.clone(), context.scope.clone());
                                function_temp = Rc::new(RefCell::new(function));
                                // context.def_member(name.clone(), function_temp.clone(), VarType::NAMESPACE);
                                let mut scope = context.scope.borrow_mut();
                                scope.def_member(name.clone(), function_temp.clone());
                            }
                            registers.default = function_temp.clone();
                        }
                        Instruction::Call(_target_reg, _args_reg, _options) => {
                            immature_break = true;
                            break;
                        }
                        Instruction::CallEnd => {
                            immature_break = true;
                            break;
                        }
                        Instruction::CreateArguments(reg) => {
                            self.pos += 1;
                            let data = Vec::<Rc<RefCell<Value>>>::new();
                            registers.insert(reg.clone(), Rc::new(RefCell::new(data)));
                        }
                        Instruction::SetItem(target_reg, index) => {
                            self.pos += 1;

                            let value;
                            {
                                let value_ = registers.default.borrow();
                                value = value_.downcast_ref::<Value>().unwrap().clone();
                            }
                            let target_temp = registers.get(*target_reg);
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
                            {
                                let value_ = registers.default.borrow();
                                value = value_.downcast_ref::<Value>().unwrap().clone();
                            }
                            let target_temp = registers.get(*target_reg);
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
                        _ => unimplemented!()
                    }
                }
            }

            if immature_break {
                let instr = &block.instructions[self.pos];
                match instr {
                    Instruction::Call(target_reg, args_reg, _options) => {
                        self.pos += 1;

                        let borrowed_;
                        let borrowed;
                        let target;
                        let new_context;

                        {
                            let registers = self.registers_store.find(registers_id);
                            borrowed_ = registers.get(*target_reg);
                            borrowed = borrowed_.borrow();
                            target = borrowed.downcast_ref::<Function>().unwrap();

                            let mut new_scope = Scope::new(target.parent_scope.clone());

                            if let Some(reg) = args_reg {
                                let args_temp = registers.get(*reg);
                                let args_ = args_temp.borrow();
                                let args = args_.downcast_ref::<Vec<Rc<RefCell<Value>>>>().unwrap();

                                for matcher in target.args.data_matchers.iter() {
                                    let arg_value = args[matcher.index].clone();
                                    new_scope.def_member(matcher.name.clone(), arg_value);
                                }
                            }

                            let new_namespace = Namespace::new(target.parent_namespace.clone());
                            new_context = Context::new(Rc::new(RefCell::new(new_namespace)), Rc::new(RefCell::new(new_scope)), None);
                        }

                        let new_registers = self.registers_store.get(Rc::new(RefCell::new(new_context)));

                        let ret_addr = Address::new(block.id.clone(), self.pos);
                        new_registers.caller = Some(ret_addr);
                        new_registers.caller_registers = registers_id;

                        registers_id = new_registers.id;
                        block = self.code_manager.blocks[&target.body].clone();
                        self.pos = 0;
                    }
                    Instruction::CallEnd => {
                        let old_registers_id = registers_id;

                        {
                            let registers = self.registers_store.find(registers_id);
                            let caller = registers.caller.as_ref();
                            if let Some(ret_addr) = caller {
                                block = self.code_manager.blocks[&ret_addr.block_id].clone();
                                self.pos = ret_addr.pos;

                                let value = registers.default.clone();
                                let caller_reg_id = registers.caller_registers;
                                let caller_registers = self.registers_store.find(caller_reg_id);
                                // Save returned value in caller's default register
                                caller_registers.default = value;

                                registers_id = caller_reg_id;
                            } else {
                                self.pos += 1;
                            }
                        }

                        self.registers_store.free(old_registers_id);
                    }
                    _ => unimplemented!()
                }
            } else {
                break;
            }
        }

        let registers = self.registers_store.find(registers_id);
        let result = registers.default.clone();
        // dbg!(result.borrow().downcast_ref::<Value>().unwrap());

        println!("Execution time: {:.6} seconds", start_time.elapsed().as_nanos() as f64 / 1_000_000_000.);

        result
    }
}

#[derive(Debug)]
pub struct Registers {
    pub id: usize,
    pub caller: Option<Address>,
    pub caller_registers: usize,
    pub default: Rc<RefCell<dyn Any>>,
    pub context: Rc<RefCell<Context>>,
    pub store: Vec<Option<Rc<RefCell<dyn Any>>>>,
}

impl Registers {
    pub fn new(id: usize, context: Rc<RefCell<Context>>) -> Self {
        Registers {
            id,
            caller: None,
            caller_registers: 0,
            default: Rc::new(RefCell::new(0)),
            context,
            store: Vec::new(),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
    }

    #[inline]
    pub fn insert(&mut self, key: u16, val: Rc<RefCell<dyn Any>>) {
        let len = self.store.len() as u16;
        if key < len {
            self.store[key as usize].replace(val);
        } else {
            if key > len {
                for _i in len..key {
                    self.store.push(None);
                }
            }
            self.store.push(Some(val));
        }
    }

    #[inline]
    pub fn get(&self, key: u16) -> Rc<RefCell<dyn Any>> {
        self.store[key as usize].as_ref().unwrap().clone()
     }

    #[inline]
    fn get_member(&self, name: &str) -> Option<Rc<RefCell<dyn Any>>> {
        let context = self.context.borrow();
        context.get_member(name)
    }

    #[inline]
    fn set_member(&mut self, name: String, value: Rc<RefCell<dyn Any>>) {
        let mut context = self.context.borrow_mut();
        context.set_member(name.clone(), value.clone());
    }
}

pub struct RegistersStore {
    store: Vec<Registers>,
    freed: Vec<usize>,
    next: usize,
}

impl RegistersStore {
    pub fn new() -> Self {
        RegistersStore {
            store: Vec::new(),
            freed: Vec::new(),
            next: 0,
        }
    }

    #[inline]
    pub fn get(&mut self, context: Rc<RefCell<Context>>) -> &mut Registers {
        if self.freed.is_empty() {
            let id = self.next;
            let registers = Registers::new(id, context);
            self.store.insert(self.next, registers);
            self.next += 1;
            self.store.get_mut(id).unwrap()
        } else {
            let id = self.freed.pop().unwrap();
            let registers = self.store.get_mut(id).unwrap();
            registers.context = context;
            registers
        }
    }

    #[inline]
    pub fn find(&mut self, id: usize) -> &mut Registers {
        self.store.get_mut(id).unwrap()
    }

    #[inline]
    pub fn free(&mut self, id: usize) {
        self.find(id).reset();
        self.freed.push(id);
    }
}

#[inline]
fn add<'a>(
    first: Rc<RefCell<dyn Any>>,
    second: Rc<RefCell<dyn Any>>,
) -> Rc<RefCell<dyn Any>> {
    let borrowed1 = first.borrow();
    let borrowed2 = second.borrow();
    let value1 = borrowed1.downcast_ref::<Value>().unwrap();
    let value2 = borrowed2.downcast_ref::<Value>().unwrap();
    match (value1, value2) {
        (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Integer(a + b))),
        _ => unimplemented!()
    }
}

#[inline]
fn sub<'a>(
    first: Rc<RefCell<dyn Any>>,
    second: Rc<RefCell<dyn Any>>,
) -> Rc<RefCell<dyn Any>> {
    let borrowed1 = first.borrow();
    let borrowed2 = second.borrow();
    let value1 = borrowed1.downcast_ref::<Value>().unwrap();
    let value2 = borrowed2.downcast_ref::<Value>().unwrap();
    match (value1, value2) {
        (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Integer(a - b))),
        _ => unimplemented!()
    }
}

#[inline]
fn lt<'a>(
    first: Rc<RefCell<dyn Any>>,
    second: Rc<RefCell<dyn Any>>,
) -> Rc<RefCell<dyn Any>> {
    let borrowed1 = first.borrow();
    let borrowed2 = second.borrow();
    let value1 = borrowed1.downcast_ref::<Value>().unwrap();
    let value2 = borrowed2.downcast_ref::<Value>().unwrap();
    match (value1, value2) {
        (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Boolean(a < b))),
        _ => unimplemented!()
    }
}

#[inline]
fn gt<'a>(
    first: Rc<RefCell<dyn Any>>,
    second: Rc<RefCell<dyn Any>>,
) -> Rc<RefCell<dyn Any>> {
    let borrowed1 = first.borrow();
    let borrowed2 = second.borrow();
    let value1 = borrowed1.downcast_ref::<Value>().unwrap();
    let value2 = borrowed2.downcast_ref::<Value>().unwrap();
    match (value1, value2) {
        (Value::Integer(a), Value::Integer(b)) => Rc::new(RefCell::new(Value::Boolean(a > b))),
        _ => unimplemented!()
    }
}

#[derive(Debug)]
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

// struct Scope {
//     pub id: usize,
//     /// Total usage of the scope
//     ///   When the scope is created/re-used, incremented by 1
//     ///   When the current block ends, decremented by 1
//     ///   When a member is referenced in a child scope, incremented by 1
//     ///   When the member is not used any more, or the child block ends, decrement by 1
//     ///   When usage is 0, clear members, deep_members etc
//     pub usage: usize,
//     pub parent: Option<usize>,
//     pub members: Vec<Rc<RefCell<dyn Any>>>,
//     pub deep_members: HashMap<String, Rc<RefCell<dyn Any>>>,
//     pub inherited_members: Option<HashMap<String, ScopeRef>>,
// }

// impl Scope {
//     pub fn new(id: usize) -> Self {
//         Scope {
//             id,
//             usage: 1,
//             parent: None,
//             members: Vec::new(),
//             deep_members: HashMap::new(),
//             inherited_members: None,
//         }
//     }
// }

// enum ScopeRef {
//     /// Member(scope id, index to members)
//     Member(usize, usize),
//     /// DeepMember(scope id, name in deep_members)
//     DeepMember(usize, String),
// }

// struct ScopeManager {
// }