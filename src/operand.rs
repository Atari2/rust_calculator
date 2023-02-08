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
    pub right: Option<Box<Node>>,
    pub value: f64
}

#[derive(Debug)]
pub struct Tree {
    pub head: Node
}

impl Node {
    pub fn new_empty() -> Node {
        Node{operand: None, left: None, right: None, value: 0.0}
    }

    pub fn populate(&mut self, operands: &mut Vec<Operand>) -> Result<(), Box<dyn std::error::Error>> {
        if operands.len() > 0 {
            self.operand = operands.pop();
            let op: &Operand = self.operand.as_ref().ok_or("Empty operand")?;
            if op.priority != Priority::Number && op.priority != Priority::Unary && operands.len() > 1 {
                let mut left = Node::new_empty();
                let mut right = Node::new_empty();
                right.populate(operands)?;
                left.populate(operands)?;
                self.left = Some(Box::new(left));
                self.right = Some(Box::new(right));
            } else if op.priority == Priority::Unary {
                let mut left = Node::new_empty();
                left.populate(operands)?;
                self.left = Some(Box::new(left));
                self.right = None;
            } else if op.priority == Priority::Number {
                return Ok(())
            } else {
                return Err(Box::from("Broken mathematical expression, only valid unary operators or repeatable operators are -, ! and +"));
            }
        }
        Ok(())
    }
    pub fn navigate(&mut self) -> Result<f64, Box<dyn std::error::Error>> {
        let tuple = (&mut self.left, &mut self.right, &self.operand);
        if let (Some(left), Some(right), Some(op)) = tuple {
            let leftres = left.navigate()?;
            let rightres = right.navigate()?;
            let result = match op.symbol.as_str() {
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
            self.value = result;
            return Ok(result);
        } else if let (Some(left), None, Some(op)) = tuple {
            assert_eq!(op.priority, Priority::Unary);
            let mut val = left.navigate()?;
            val = match op.symbol.as_str() {
                "+" => val,
                "-" => -val,
                "~" => !(val as i64) as f64,
                _ => return Err(Box::from("Invalid unary operator"))
            };
            self.value = val;
            return Ok(val);
        } else {
            if let Some(op) = &self.operand {
                self.value = str::parse::<f64>(op.symbol.as_str())?;
                return Ok(self.value);
            } else {
                return Err(Box::from("Broken mathematical expression, only valid unary operators or repeatable operators are -, ! and +"    ))
            }
        }
    }

    pub fn print(&self, depth: usize) {
        if self.operand.is_some() {
            let depth_str = vec!['\t'; depth].iter().collect::<String>();
            if let Some(op) = &self.operand {
                println!("{}{:?} => partial result {}", depth_str, op, self.value);
            }
            if let Some(left) = &self.left {
                println!("{}left child: ", depth_str);
                left.print(depth + 1);
            }
            if let Some(right) = &self.right {
                println!("{}right child: ", depth_str);
                right.print(depth + 1);
            }
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

    pub fn navigate(&mut self) -> Result<f64, Box<dyn std::error::Error>> {
        self.head.navigate()
    }

    pub fn print(&self) {
        self.head.print(0);
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