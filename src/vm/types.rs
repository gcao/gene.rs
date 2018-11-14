use std::any::Any;
use std::collections::{BTreeMap};

use super::super::compiler::Block;

#[derive(Debug)]
pub struct Application {
}

impl Application {
}

#[derive(Debug)]
pub struct Context {
    namespace: Namespace
}

impl Context {
}

#[derive(Debug)]
pub struct Namespace {
    members: BTreeMap<String, Box<Any>>,
}

#[derive(Debug)]
pub struct Scope {
    pub parent: Option<Box<Scope>>,
    pub variables: BTreeMap<String, Box<Any>>,
}

impl Scope {
    pub fn new(parent: Option<Scope>) -> Self {
        let parent =
            if parent.is_none() {
                None
            } else {
                Some(Box::new(parent.unwrap()))
            };
        return Scope {
            parent: parent,
            variables: BTreeMap::new(),
        };
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
        return Function {
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
