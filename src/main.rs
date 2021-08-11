#![allow(dead_code)]

use std::io::{self, BufRead};
use std::error;
use std::collections::HashMap;
mod operand;
use operand::{Operand, Priority, Type, Tree, Node};

macro_rules! map {
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
}

fn split_line(buf: &String) -> Vec<Type> {
    // << and >> are not here because match_indices apparently can't match &[str] but only &[char]. Sad times.
    // when I find a way to split them easily, the rest is already set up to handle them.
    let symbols = &['+', '-', '*', '/', '%', '^', '|', '&', '(', ')', '~'][..];
    let map: HashMap<&str, Priority> = 
    map!{
        "+" => Priority::High, 
        "-" => Priority::High, 
        "*" => Priority::Higher, 
        "/" => Priority::Higher,
        "%" => Priority::Higher,
        "^" => Priority::Max,
        "~" => Priority::Low,
        "|" => Priority::Low,
        "&" => Priority::Low,
        ">>" => Priority::Low,
        "<<" => Priority::Low,
        "(" => Priority::LeftParens,
        ")" => Priority::RightParens
    };
    let indices: Vec<_> = buf.match_indices(symbols).collect();
    let mut separated: Vec<Type> = vec![];
    let mut prev_index = 0 as usize;
    for (index, symbol) in indices {
        if prev_index != index && index - prev_index != 0 {
            separated.push(Type::Number(buf[prev_index..index].trim().into()));
        }
        prev_index = index + symbol.len();
        separated.push(Type::Symbol(symbol.trim().into(), map[symbol]));
    }
    if prev_index != buf.len() {
        separated.push(Type::Number(buf[prev_index..].trim().into()));
    }
    separated
}

fn parse_line(buf: &String) -> Result<f64, Box<dyn error::Error>> {
    let numbers = split_line(buf);
    let mut tree = Tree::new();
    let mut stack: Vec<Operand> = vec![];
    let mut output: Vec<Operand> = vec![];
    let mut canparseunary = true;
    for part in numbers {
        match part {
            Type::Number(num) => {
                canparseunary = false;
                output.push(Operand::new(num.clone(), Priority::Number));
                while stack.len() > 0 {
                    if stack.last().unwrap().priority == Priority::Unary {
                        output.push(stack.pop().unwrap());
                    } else {
                        break;
                    }
                }
            }
            Type::Symbol(sym, prio) => {
                match prio {
                    Priority::Low | Priority::Medium | Priority::High | Priority::Higher | Priority::Max => {
                        if canparseunary {
                            while stack.len() != 0 {
                                let last = stack.last().unwrap();
                                if last.priority > Priority::Unary && last.priority != Priority::LeftParens {
                                    output.push(stack.pop().unwrap());
                                } else {
                                    break;
                                }
                            }
                            match sym.as_str() {
                                "~" | "+" | "-" => {
                                    stack.push(Operand::new(sym.clone(), Priority::Unary))
                                }
                                _ => return Err(Box::from("Invalid unary operator"))
                            }
                        } 
                        else {
                            while stack.len() != 0 {
                                let last = stack.last().unwrap();
                                if last.priority >= prio && last.priority != Priority::LeftParens {
                                    output.push(stack.pop().unwrap());
                                } else {
                                    break;
                                }
                            }
                            stack.push(Operand::new(sym.clone(), prio));
                        }
                        canparseunary = true;
                    }
                    Priority::LeftParens => { 
                        stack.push(Operand::new(sym.clone(), prio));
                        canparseunary = true;
                    }
                    Priority::RightParens => {
                        if stack.len() == 0 {
                            return Err(Box::from("Mismatched parenthesis"));
                        }
                        while stack.last().unwrap().priority != Priority::LeftParens {
                            output.push(stack.pop().unwrap());
                            if stack.len() == 0 {
                                return Err(Box::from("Mismatched parenthesis"));
                            }
                        }
                        if stack.last().unwrap().priority == Priority::LeftParens {
                            stack.pop();
                        }
                        canparseunary = false;
                    }
                    Priority::Number | Priority::Unary => return Err(Box::from("Numbers or unary operators shouldn't be parsed here"))
                }
            }
        } 
    }
    while stack.len() > 0 {
        let last = stack.pop().unwrap();
        if last.priority == Priority::LeftParens || last.priority == Priority::RightParens {
            return Err(Box::from("Mismatched parenthesis"));
        }
        output.push(last);
    }
    println!("{:?}", output);
    tree.head = Node::new_empty();
    tree.populate(&mut output)?;
    Ok(tree.navigate()?)
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let val = parse_line(&line?);
        match val {
            Ok(float_res) => {
                println!("Result is {}", float_res);
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }
    Ok(())
}
