use crate::object::Object;
use std::collections::{HashMap, LinkedList};
use std::process;
use std::sync::Arc;

pub type PrimitiveFunction = fn(&Object) -> Object;

fn quit(_: &Object) -> Object {
    process::exit(0)
}

fn get(obj: &Object) -> Object {
    if let Object::List(list) = obj {
        if list.len() < 3 {
            return Object::Null;
        }
        let mut iter = list.iter();
        iter.next();
        match iter.next().unwrap() {
            Object::Vector(vector) => {
                if let Object::Integer(index) = iter.next().unwrap() {
                    if let Some(value) = vector.get(*index as usize) {
                        return value.clone();
                    }
                }
            }
            Object::Map(map) => {
                if let Some(value) = map.get(iter.next().unwrap()) {
                    return value.clone();
                }
            }
            _ => {}
        }
    }
    Object::Null
}

pub struct Evaluator {
    global: Object,
}

impl Evaluator {
    pub fn new() -> Evaluator {
        let mut map = HashMap::new();
        let get: PrimitiveFunction = get;
        map.insert(
            Object::Symbol("get".to_string()),
            Object::Other(Arc::new(get)),
        );
        let quit: PrimitiveFunction = quit;
        map.insert(
            Object::Symbol("quit".to_string()),
            Object::Other(Arc::new(quit)),
        );
        Evaluator {
            global: Object::Map(map),
        }
    }

    pub fn eval(&self, obj: &Object) -> Object {
        match obj {
            Object::Symbol(s) => self.eval_symbol(s),
            Object::List(list) => self.eval_list(list),
            _ => obj.clone(),
        }
    }

    fn eval_symbol(&self, string: &str) -> Object {
        let symbol = Object::Symbol(string.to_string());
        if let Object::Map(global) = &self.global {
            if let Some(obj) = global.get(&symbol) {
                return obj.clone();
            }
        }
        symbol
    }

    fn eval_list(&self, list: &LinkedList<Object>) -> Object {
        if list.is_empty() {
            return Object::Null;
        }
        let mut iter = list.iter();
        let obj = self.eval(iter.next().unwrap());
        if let Object::Other(other) = obj.clone() {
            if let Some(primitive_function) = other.downcast_ref::<PrimitiveFunction>() {
                let mut after_eval = LinkedList::new();
                after_eval.push_back(obj);
                for obj in iter {
                    after_eval.push_back(obj.clone());
                }
                return primitive_function(&Object::List(after_eval));
            }
        }
        Object::Null
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
