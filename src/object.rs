use std::any::Any;
use std::collections::{HashMap, LinkedList};
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::result::Result;
use std::str::{Chars, FromStr};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Object {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Symbol(String),
    List(LinkedList<Object>),
    Vector(Vec<Object>),
    Map(HashMap<Object, Object>),
    Other(Arc<dyn Any>),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Null, Object::Null) => true,
            (Object::Null, Object::Bool(x)) => !x,
            (Object::Bool(x), Object::Bool(y)) => x == y,
            (Object::Integer(x), Object::Integer(y)) => x == y,
            (Object::Integer(x), Object::Float(y)) => *x as f64 == *y,
            (Object::Float(x), Object::Integer(y)) => *x == *y as f64,
            (Object::Float(x), Object::Float(y)) => x == y,
            (Object::String(x), Object::String(y)) => x == y,
            (Object::Symbol(x), Object::Symbol(y)) => x == y,
            (Object::List(x), Object::List(y)) => x == y,
            (Object::Vector(x), Object::Vector(y)) => x == y,
            (Object::Map(x), Object::Map(y)) => x == y,
            _ => false,
        }
    }
}

impl Eq for Object {}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Object::Integer(n) => n.hash(state),
            Object::String(s) => s.hash(state),
            Object::Symbol(s) => s.hash(state),
            _ => {}
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Null => write!(f, "null"),
            Object::Bool(n) => write!(f, "{}", n),
            Object::Integer(n) => write!(f, "{}", n),
            Object::Float(n) => write!(f, "{}", n),
            Object::String(s) => write!(f, "{:?}", s),
            Object::Symbol(s) => write!(f, "{}", s),
            Object::List(list) => {
                if list.is_empty() {
                    return write!(f, "()");
                }
                let mut s = String::new();
                s.push('(');
                for obj in list {
                    s.push_str(&(obj.to_string() + " "));
                }
                s.pop();
                s.push(')');
                write!(f, "{}", s)
            }
            Object::Vector(vector) => {
                if vector.is_empty() {
                    return write!(f, "[]");
                }
                let mut s = String::new();
                s.push('[');
                for obj in vector {
                    s.push_str(&(obj.to_string() + ", "));
                }
                s.pop();
                s.pop();
                s.push(']');
                write!(f, "{}", s)
            }
            Object::Map(map) => {
                if map.is_empty() {
                    return write!(f, "{{}}");
                }
                let mut s = String::new();
                s.push('{');
                for (key, value) in map {
                    s.push_str(&(key.to_string() + ": " + &value.to_string() + ", "));
                }
                s.pop();
                s.pop();
                s.push('}');
                write!(f, "{}", s)
            }
            Object::Other(other) => write!(f, "<{:?}>", other),
        }
    }
}

#[derive(Debug)]
pub struct ParseObjectError;

impl fmt::Display for ParseObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseObjectError {}

fn atomize_expr_escape_char(chars: &mut Chars) -> Result<char, ParseObjectError> {
    if let Some(c) = chars.next() {
        match c {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            _ => Err(ParseObjectError {}),
        }
    } else {
        Err(ParseObjectError {})
    }
}

fn atomize_expr_chars_to_str(chars: &mut Chars) -> Result<Object, ParseObjectError> {
    let mut s = String::new();
    while let Some(c) = chars.next() {
        if c == '\\' {
            s.push(atomize_expr_escape_char(chars)?);
        } else if c == '"' {
            return Ok(Object::String(s));
        } else {
            s.push(c);
        }
    }
    Err(ParseObjectError {})
}

fn atomize_expr_push(
    expr: &mut LinkedList<Object>,
    s: &mut String,
) -> Result<(), ParseObjectError> {
    if s.is_empty() {
        return Ok(());
    }
    if s.chars().next().unwrap().is_numeric() {
        if let Ok(n) = s.parse::<i64>() {
            expr.push_back(Object::Integer(n));
        } else if let Ok(n) = s.parse::<f64>() {
            expr.push_back(Object::Float(n));
        } else {
            return Err(ParseObjectError {});
        }
    } else if s == "null" {
        expr.push_back(Object::Null);
    } else if s == "true" {
        expr.push_back(Object::Bool(true));
    } else if s == "false" {
        expr.push_back(Object::Bool(false));
    } else {
        expr.push_back(Object::Symbol(s.clone()));
    }
    s.clear();
    Ok(())
}

fn atomize_expr(s: &str) -> Result<LinkedList<Object>, ParseObjectError> {
    let mut expr = LinkedList::new();
    let mut chars = s.chars();
    let mut s = String::new();
    while let Some(c) = chars.next() {
        if c == '"' {
            atomize_expr_push(&mut expr, &mut s)?;
            expr.push_back(atomize_expr_chars_to_str(&mut chars)?);
        } else if c.is_whitespace() {
            atomize_expr_push(&mut expr, &mut s)?;
        } else if c == '.' && s.chars().next().unwrap().is_numeric() {
            s.push(c);
        } else if c.is_ascii_punctuation() && c != '_' {
            atomize_expr_push(&mut expr, &mut s)?;
            expr.push_back(Object::Symbol(c.to_string()));
        } else {
            s.push(c);
        }
    }
    Ok(expr)
}

fn parse_list(
    expr: &mut LinkedList<Object>,
    is_delimiter: &mut dyn FnMut(&Object) -> bool,
) -> Result<LinkedList<Object>, ParseObjectError> {
    let mut list = LinkedList::new();
    while !expr.is_empty() {
        if is_delimiter(expr.front().unwrap()) {
            return Ok(list);
        }
        list.push_back(parse_mut_expr(expr)?);
    }
    Err(ParseObjectError {})
}

fn parse_mut_expr(expr: &mut LinkedList<Object>) -> Result<Object, ParseObjectError> {
    if expr.is_empty() {
        return Err(ParseObjectError {});
    }
    if *expr.front().unwrap() == Object::Symbol("(".to_string()) {
        expr.pop_front();
        let list = parse_list(expr, &mut |obj| *obj == Object::Symbol(")".to_string()))?;
        expr.pop_front();
        return Ok(Object::List(list));
    }
    if *expr.front().unwrap() == Object::Symbol("[".to_string()) {
        expr.pop_front();
        let mut vector = Vec::new();
        while !expr.is_empty() {
            if *expr.front().unwrap() == Object::Symbol("]".to_string()) {
                expr.pop_front();
                return Ok(Object::Vector(vector));
            }
            if *expr.front().unwrap() == Object::Symbol(",".to_string()) {
                expr.pop_front();
            }
            let mut list = parse_list(expr, &mut |obj| {
                *obj == Object::Symbol(",".to_string()) || *obj == Object::Symbol("]".to_string())
            })?;
            if list.len() != 1 {
                return Err(ParseObjectError {});
            }
            vector.push(list.pop_front().unwrap());
        }
        return Err(ParseObjectError {});
    }
    if *expr.front().unwrap() == Object::Symbol("{".to_string()) {
        expr.pop_front();
        let mut map = HashMap::new();
        while !expr.is_empty() {
            if *expr.front().unwrap() == Object::Symbol("}".to_string()) {
                expr.pop_front();
                return Ok(Object::Map(map));
            }
            if *expr.front().unwrap() == Object::Symbol(",".to_string()) {
                expr.pop_front();
            }
            let mut list = parse_list(expr, &mut |obj| {
                *obj == Object::Symbol(",".to_string()) || *obj == Object::Symbol("}".to_string())
            })?;
            if list.len() != 3 {
                return Err(ParseObjectError {});
            }
            let first = list.pop_front().unwrap();
            let second = list.pop_front().unwrap();
            if second != Object::Symbol(":".to_string()) {
                return Err(ParseObjectError {});
            }
            let third = list.pop_front().unwrap();
            map.insert(first, third);
        }
        return Err(ParseObjectError {});
    }
    Ok(expr.pop_front().unwrap())
}

fn parse_expr(expr: &LinkedList<Object>) -> Result<Object, ParseObjectError> {
    parse_mut_expr(&mut expr.clone())
}

impl FromStr for Object {
    type Err = ParseObjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let expr = atomize_expr(s)?;
        parse_expr(&expr)
    }
}
