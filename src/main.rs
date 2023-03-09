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
    let mut prev_index = 0_usize;
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

fn parse_line(orig_buf: &str) -> Result<f64, Box<dyn error::Error>> {
    let buf: String = orig_buf.chars().filter(|c| !c.is_whitespace()).collect();
    let numbers = split_line(&buf);
    let mut tree = Tree::new();
    let mut stack: Vec<Operand> = vec![];
    let mut output: Vec<Operand> = vec![];
    let mut canparseunary = true;
    for part in numbers {
        match part {
            Type::Number(num) => {
                canparseunary = false;
                output.push(Operand::new(num.clone(), Priority::Number));
                while !stack.is_empty() {
                    match stack.last() {
                        Some(Operand {priority: Priority::Unary, ..}) => {
                            let last = match stack.pop() {
                                Some(operand) => operand,
                                None => return Err(Box::from("Invalid unary operator"))
                            };
                            output.push(last);
                        }
                        _ => break
                    }
                }
            }
            Type::Symbol(sym, prio) => {
                match prio {
                    Priority::Low | Priority::Medium | Priority::High | Priority::Higher | Priority::Max => {
                        if canparseunary {
                            while let Some(last) = stack.last() {
                                if last.priority > Priority::Unary && last.priority != Priority::LeftParens {
                                    output.push(last.clone());
                                    stack.pop();
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
                            while let Some(last) = stack.last() {
                                if last.priority >= prio && last.priority != Priority::LeftParens {
                                    output.push(last.clone());
                                    stack.pop();
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
                        if stack.is_empty() {
                            return Err(Box::from("Mismatched parenthesis"));
                        }
                        while let Some(last) = stack.last() {
                            if last.priority == Priority::LeftParens {
                                break;
                            }
                            if let Some(last) = stack.pop() {
                                output.push(last);
                            }
                            if stack.is_empty() {
                                return Err(Box::from("Mismatched parenthesis"));
                            }
                        }
                        if let Some(Operand{priority: Priority::LeftParens, ..}) = stack.last() {
                            stack.pop();
                        }
                        canparseunary = false;
                    }
                    Priority::Number | Priority::Unary => return Err(Box::from("Numbers or unary operators shouldn't be parsed here"))
                }
            }
        } 
    }
    while let Some(last) = stack.pop() {
        if last.priority == Priority::LeftParens || last.priority == Priority::RightParens {
            return Err(Box::from("Mismatched parenthesis"));
        }
        output.push(last);
    }
    tree.head = Node::new_empty();
    match tree.populate(&mut output) {
        Ok(_) => (),
        Err(e) => {
            tree.print(); 
            return Err(e)
        }
    }
    let ret = Ok(tree.navigate()?);
    tree.print();
    ret
}

#[cfg(test)]
mod tests {
    use super::parse_line;
    #[test]
    fn basic_test() {
        let a = "1 + 2 * 3".to_string();
        let b = "(1 + 2) * 3".to_string();
        let c = "1 + 2 * 3 + 4".to_string();
        let d = "(1+2)*(1*2-3)*3^4".to_string();
        let val = parse_line(&a);
        let val2 = parse_line(&b);
        let val3 = parse_line(&c);
        let val4 = parse_line(&d);
        assert_eq!(val.unwrap(), 7.0);
        assert_eq!(val2.unwrap(), 9.0);
        assert_eq!(val3.unwrap(), 11.0);
        assert_eq!(val4.unwrap(), -243.0);
    }
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
