use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use super::super::types::Value;
use super::super::compiler::Block;

#[derive(Debug)]
pub struct Application {}

impl Application {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct Context {
    pub namespace: Rc<RefCell<Namespace>>,
    pub scope: Rc<RefCell<Scope>>,
    pub _self: Option<Rc<RefCell<Any>>>,
}

pub enum VarType {
    SCOPE,
    NAMESPACE,
}

impl Context {
    pub fn new(namespace: Rc<RefCell<Namespace>>, scope: Rc<RefCell<Scope>>, _self: Option<Rc<RefCell<Any>>>) -> Self {
        Self {
            namespace,
            scope,
            _self,
        }
    }

    pub fn root() -> Self {
        Self {
            namespace: Rc::new(RefCell::new(Namespace::root())),
            scope: Rc::new(RefCell::new(Scope::root())),
            _self: None,
        }
    }

    pub fn def_member(&mut self, name: String, value: Rc<RefCell<Any>>, var_type: VarType) {
        match var_type {
            VarType::SCOPE => {
                self.scope.borrow_mut().def_member(name, value);
            }
            VarType::NAMESPACE => {
                self.scope.borrow_mut().def_member(name, value);
            }
        }
    }

    pub fn get_member(&self, name: String) -> Option<Rc<RefCell<Any>>> {
        let result = self.scope.borrow().get_member(name.clone());
        if result.is_none() {
            self.namespace.borrow().get_member(name)
        } else {
            result
        }
    }

    pub fn set_member(&mut self, name: String, value: Rc<RefCell<Any>>) {
        if self.scope.borrow().has_member(name.clone()) {
            self.scope.borrow_mut().set_member(name, value);
        } else if self.namespace.borrow().has_member(name.clone()) {
            self.namespace.borrow_mut().set_member(name, value);
        } else {
            panic!("Undefined variable: {}", name);
        }
    }

}

#[derive(Clone, Debug)]
pub struct Namespace {
    parent: Option<Rc<RefCell<Namespace>>>,
    members: HashMap<String, Rc<RefCell<Any>>>,
}

impl Namespace {
    pub fn new(parent: Rc<RefCell<Namespace>>) -> Self {
        Self {
            parent: Some(parent),
            members: HashMap::new(),
        }
    }

    pub fn root() -> Self {
        Self {
            parent: None,
            members: HashMap::new(),
        }
    }

    pub fn def_member(&mut self, name: String, value: Rc<RefCell<Any>>) {
        self.members.insert(name, value);
    }

    pub fn get_member(&self, name: String) -> Option<Rc<RefCell<Any>>> {
        if self.members.contains_key(&name) {
            self.members.get(&name).cloned()
        } else if self.parent.is_some() {
            self.parent.clone().unwrap().borrow().get_member(name)
        } else {
            None
        }
    }

    pub fn set_member(&mut self, name: String, value: Rc<RefCell<Any>>) {
        if self.members.contains_key(&name) {
            self.members.insert(name.clone(), value);
        } else {
            let parent = self.parent.clone().unwrap();
            let mut borrowed = parent.borrow_mut();
            borrowed.set_member(name, value);
        }
    }

    pub fn has_member(&self, name: String) -> bool {
        if self.members.contains_key(&name) {
            true
        } else if self.parent.is_some() {
            self.parent.clone().unwrap().borrow().has_member(name)
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub members: HashMap<String, Rc<RefCell<Any>>>,
}

impl Scope {
    pub fn new(parent: Rc<RefCell<Scope>>) -> Self {
        Scope {
            parent: Some(parent),
            members: HashMap::new(),
        }
    }

    pub fn root() -> Self {
        Scope {
            parent: None,
            members: HashMap::new(),
        }
    }

    pub fn def_member(&mut self, name: String, value: Rc<RefCell<Any>>) {
        self.members.insert(name, value);
    }

    pub fn get_member(&self, name: String) -> Option<Rc<RefCell<Any>>> {
        let value = self.members.get(&name);
        if value.is_none() && self.parent.is_some() {
            let parent_ = self.parent.clone().unwrap();
            let parent = parent_.borrow();
            let v = parent.get_member(name);
            if v.is_some() {
                Some(v.unwrap().clone())
            } else {
                None
            }
        } else {
            value.cloned()
        }
    }

    pub fn set_member(&mut self, name: String, value: Rc<RefCell<Any>>) {
        if self.members.contains_key(&name) {
            self.members.insert(name.clone(), value);
        } else {
            self.parent.clone().unwrap().borrow_mut().set_member(name, value);
        }
    }

    pub fn has_member(&self, name: String) -> bool {
        if self.members.contains_key(&name) {
            true
        } else if self.parent.is_some() {
            self.parent.clone().unwrap().borrow().has_member(name)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataMatcher {
    pub name: String,
    pub index: usize,
}

impl DataMatcher {
    pub fn new(name: String, index: usize) -> Self {
        DataMatcher {
            name,
            index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Matcher {
    pub data_matchers: Vec<DataMatcher>,
}

impl Matcher {
    pub fn new(data_matchers: Vec<DataMatcher>) -> Self {
        Matcher {
            data_matchers,
        }
    }
}

impl From<&Value> for Matcher {
    fn from(v: &Value) -> Matcher {
        match v {
            Value::Symbol(name) => {
                let mut matchers = Vec::<DataMatcher>::new();
                if name != "_" {
                    matchers.push(DataMatcher::new(name.to_string(), 0));
                }
                Matcher::new(matchers)
            }
            Value::Array(args) => {
                let mut matchers = Vec::<DataMatcher>::new();
                for (index, name) in args.iter().enumerate() {
                    matchers.push(DataMatcher::new(name.to_string(), index));
                }
                Matcher::new(matchers)
            }
            _ => unimplemented!()
        }
    }
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Matcher,
    pub body: String,
    pub inherit_scope: bool,
    pub parent_namespace: Rc<RefCell<Namespace>>,
    pub parent_scope: Rc<RefCell<Scope>>,
}

impl<'a> Function {
    pub fn new(
        name: String,
        args: Matcher,
        body: String,
        inherit_scope: bool,
        parent_namespace: Rc<RefCell<Namespace>>,
        parent_scope: Rc<RefCell<Scope>>,
    ) -> Self {
        Function {
            name,
            args,
            body,
            inherit_scope,
            parent_namespace,
            parent_scope,
        }
    }
}

#[derive(Debug)]
pub struct Arguments {
    pub data: Vec<Rc<RefCell<Any>>>,
}

impl Arguments {
    pub fn new(
        data: Vec<Rc<RefCell<Any>>>,
    ) -> Self {
        Arguments {
            // props,
            data,
        }
    }
}

#[derive(Debug)]
pub struct Module {
    name: String,
    methods: HashMap<String, Function>,
}

#[derive(Debug)]
pub struct Address {
    block_id: String,
    position: usize,
}

impl Address {
    pub fn new(
        block_id: String,
        position: usize,
    ) -> Self {
        Address {
            block_id,
            position,
        }
    }
}

#[derive(Debug)]
pub struct RegAddress {
    registers_id: String,
    register: String,
}

impl RegAddress {
    pub fn new(
        registers_id: String,
        register: String,
    ) -> Self {
        RegAddress {
            registers_id,
            register
        }
    }
}