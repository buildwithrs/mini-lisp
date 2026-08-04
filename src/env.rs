use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::value::Value;

pub type EnvRef = Rc<RefCell<Env>>;

#[derive(Clone)]
pub struct Env {
    pub bindings: HashMap<String, Value>,
    pub parent: Option<EnvRef>,
}

impl Env {
    pub fn root() -> EnvRef {
        Rc::new(RefCell::new(Env {
            bindings: HashMap::new(),
            parent: None,
        }))
    }

    pub fn child(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Env {
            bindings: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn define(env: &EnvRef, name: String, val: Value) {
        env.borrow_mut().bindings.insert(name, val);
    }

    pub fn lookup(env: &EnvRef, name: &str) -> Option<Value> {
        let current = env.borrow();
        if let Some(value) = current.bindings.get(name) {
            return Some(value.clone());
        }
        
        let parent = current.parent.clone()?;

        drop(current);
        Self::lookup(&parent, name)
    }
}
