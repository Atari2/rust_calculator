#[derive(Clone, Debug)]
pub struct Operand {
    pub symbol: String,
    pub priority: Priority
}

impl Operand {
    pub fn new(symbol: String, priority: Priority) -> Operand {
        Operand { symbol, priority }
    }
}

impl PartialEq for Operand {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for Operand {}

impl PartialOrd for Operand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority.cmp(&other.priority))
    }
}

impl Ord for Operand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

#[derive(Debug)]
pub struct Node {
    pub operand: Option<Operand>,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>
}

#[derive(Debug)]
pub struct Tree {
    pub head: Node
}

impl Node {
    pub fn new_empty() -> Node {
        Node{operand: None, left: None, right: None }
    }

    pub fn populate(&mut self, operands: &mut Vec<Operand>) -> Result<(), Box<dyn std::error::Error>> {
        if operands.len() > 0 {
            self.operand = operands.pop();
            let op: &Operand = self.operand.as_ref().ok_or("Empty operand")?;
            if op.priority != Priority::Number && op.priority != Priority::Unary && operands.len() > 1 {
                self.left = Some(Box::from(Node::new_empty()));
                self.right = Some(Box::from(Node::new_empty()));
                self.right.as_mut().unwrap().as_mut().populate(operands)?;
                self.left.as_mut().unwrap().as_mut().populate(operands)?;
            } else if op.priority == Priority::Unary {
                self.left = Some(Box::from(Node::new_empty()));
                self.right = None;
                self.left.as_mut().unwrap().as_mut().populate(operands)?;
            } else if op.priority == Priority::Number {
                return Ok(())
            } else {
                return Err(Box::from("Broken mathematical expression, only valid unary operators or repeatable operators are -, ! and +"));
            }
        }
        Ok(())
    }
    pub fn navigate(&self) -> Result<f64, Box<dyn std::error::Error>> {
        if self.right.is_some() && self.left.is_some() {
            let leftres = self.left.as_ref().unwrap().navigate()?;
            let rightres = self.right.as_ref().unwrap().navigate()?;
            let result = match self.operand.as_ref().ok_or("Empty operand")?.symbol.as_str() {
                "+" => leftres + rightres,
                "-" => leftres - rightres,
                "*" => leftres * rightres,
                "/" => leftres / rightres,
                "%" => leftres % rightres,
                "^" => leftres.powf(rightres),
                "|" => (leftres as i64 | rightres as i64) as f64,
                "&" => (leftres as i64 & rightres as i64) as f64,
                ">>" => (leftres as i64 >> rightres as i64) as f64,
                "<<" => ((leftres as i64) << (rightres as i64)) as f64,
                "(" | ")" => return Err(Box::from("Parens should be stripped before arriving here")), 
                _ => return Err(Box::from("Unrecognized operator"))
            };
            return Ok(result);
        } else if self.left.is_some() {
            assert_eq!(self.operand.as_ref().ok_or("Empty operand")?.priority, Priority::Unary);
            let op = self.operand.as_ref().unwrap();
            let mut val = self.left.as_ref().unwrap().navigate()?;
            val = match op.symbol.as_str() {
                "+" => val,
                "-" => -val,
                "~" => !(val as i64) as f64,
                _ => return Err(Box::from("Invalid unary operator"))
            };
            return Ok(val);
        } else {
            if self.operand.is_none() {
                return Err(Box::from("Broken mathematical expression, only valid unary operators or repeatable operators are -, ! and +"    ))
            }
            let op = self.operand.as_ref().unwrap();
            return Ok(str::parse::<f64>(op.symbol.as_str())?);
        }
    }
}

impl Tree {
    pub fn new() -> Tree {
        Tree { head: Node::new_empty() }
    }

    pub fn populate(&mut self, operands: &mut Vec<Operand>) -> Result<(), Box<dyn std::error::Error>> {
        self.head.populate(operands)
    }

    pub fn navigate(&self) -> Result<f64, Box<dyn std::error::Error>> {
        self.head.navigate()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Number = -1,
    Low = 0,
    Medium = 1,
    High = 2,
    Higher = 3,
    Max = 4,
    Unary = 5,
    LeftParens = 6,
    RightParens = 7,
}

#[derive(Debug, Clone)]
pub enum Type {
    Symbol(String, Priority),
    Number(String)
}