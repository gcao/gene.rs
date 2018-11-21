use std::any::Any;
use std::collections::{BTreeMap};
use std::rc::Rc;
use std::cell::RefCell;

use super::super::compiler::Block;

#[derive(Debug)]
pub struct Application {
}

impl Application {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[derive(Debug)]
pub struct Context {
    pub parent: Option<Box<Context>>,
    pub namespace: Namespace,
    pub scope: Scope,
    pub _self: Option<Box<Any>>,
}

pub enum VarType {
    SCOPE,
    NAMESPACE,
}

impl Context {
    pub fn new(parent: Context) -> Self {
        Self {
            parent: Some(Box::new(parent)),
            namespace: Namespace::root(),
            scope: Scope::root(),
            _self: None,
        }
    }

    pub fn root() -> Self {
        Self {
            parent: None,
            namespace: Namespace::root(),
            scope: Scope::root(),
            _self: None,
        }
    }

    pub fn def_member(&mut self, name: String, value: Rc<RefCell<Any>>, var_type: VarType) -> () {
        match var_type {
            VarType::SCOPE => {
                self.scope.def_member(name, value);
            }
            VarType::NAMESPACE => {
                self.scope.def_member(name, value);
            }
        }
    }

    pub fn get_member(&self, name: String) -> Option<Rc<RefCell<Any>>> {
        let result = self.scope.get_member(name.clone());
        if result.is_none() {
            self.namespace.get_member(name)
        } else {
            result
        }
    }
}

#[derive(Debug)]
pub struct Namespace {
    parent: Option<Box<Namespace>>,
    members: BTreeMap<String, Rc<RefCell<Any>>>,
}

impl Namespace {
    pub fn new(parent: Self) -> Self {
        Self {
            parent: Some(Box::new(parent)),
            members: BTreeMap::new(),
        }
    }

    pub fn root() -> Self {
        Self {
            parent: None,
            members: BTreeMap::new(),
        }
    }

    pub fn def_member(&mut self, name: String, value: Rc<RefCell<Any>>) -> () {
        self.members.insert(name, value);
    }

    pub fn get_member(&self, name: String) -> Option<Rc<RefCell<Any>>> {
        self.members.get(&name).map(|val| Rc::clone(val))
    }
}

#[derive(Debug)]
pub struct Scope {
    pub parent: Option<Box<Scope>>,
    pub members: BTreeMap<String, Rc<RefCell<Any>>>,
}

impl Scope {
    pub fn new(parent: Self) -> Self {
        Scope {
            parent: Some(Box::new(parent)),
            members: BTreeMap::new(),
        }
    }

    pub fn root() -> Self {
        Scope {
            parent: None,
            members: BTreeMap::new(),
        }
    }

    pub fn def_member(&mut self, name: String, value: Rc<RefCell<Any>>) -> () {
        self.members.insert(name, value);
    }

    pub fn get_member(&self, name: String) -> Option<Rc<RefCell<Any>>> {
        self.members.get(&name).map(|val| Rc::clone(val))
    }
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Block,
    pub inherit_scope: bool,
    pub namespace: Namespace,
    pub scope: Scope,
}

impl<'a> Function {
    pub fn new(name: String, body: Block, inherit_scope: bool, namespace: Namespace, scope: Scope) -> Self {
        Function {
            name: name,
            body: body,
            inherit_scope: inherit_scope,
            namespace: namespace,
            scope: scope,
        }
    }
}

#[derive(Debug)]
pub struct Module {
    name: String,
    methods: BTreeMap<String, Function>,
}
