use crate::{Expression, Node};
use std::ops::{Add, Div, Mul, Neg, Not, Sub};
use token::StringValue;
impl From<f32> for Node {
    fn from(value: f32) -> Self {
        Value::from(value).into()
    }
}
impl From<StringValue> for Node {
    fn from(value: StringValue) -> Self {
        Value::from(value).into()
    }
}

impl From<bool> for Node {
    fn from(value: bool) -> Self {
        Value::from(value).into()
    }
}
impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Value::from(value).into()
    }
}
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Value {
    Number(f32),
    String(StringValue),
    Boolean(bool),
}
impl Value {
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}
impl From<Value> for Node {
    fn from(value: Value) -> Self {
        Node::Literal(value)
    }
}
impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Number(value)
    }
}
impl From<StringValue> for Value {
    fn from(value: StringValue) -> Self {
        Value::String(value)
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(StringValue(value))
    }
}
impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.into())
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}
impl Value {
    pub fn number_expression(value: impl Into<f32>) -> Self {
        Self::Number(value.into())
    }
}
impl PartialEq<Node> for Value {
    fn eq(&self, other: &Node) -> bool {
        other.eval().is_some_and(|other| self.eq(&other))
    }
}
impl Add for Value {
    type Output = Option<Value>;
    fn add(self, rhs: Self) -> Self::Output {
        println!("Adding literal {self:?} to literal {rhs:?}");
        match (self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Some(Value::Number(n1.add(n2))),
            _ => None,
        }
    }
}
impl Add<Node> for Value {
    type Output = Option<Node>;
    fn add(self, rhs: Node) -> Self::Output {
        println!("Adding literal {self:?} to expression {rhs:?}");
        rhs.eval().and_then(|rhs| self.add(rhs)).map(Node::Literal)
    }
}
impl Sub for Value {
    type Output = Option<Value>;
    fn sub(self, rhs: Self) -> Self::Output {
        println!("subing literal {self:?} to literal {rhs:?}");
        match (self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Some(Value::Number(n1.sub(n2))),
            _ => None,
        }
    }
}
impl Sub<Node> for Value {
    type Output = Option<Node>;
    fn sub(self, rhs: Node) -> Self::Output {
        println!("Subtracting expression {rhs:?} from literal {self:?}");
        rhs.eval().and_then(|rhs| self.sub(rhs)).map(Node::Literal)
    }
}
impl Div for Value {
    type Output = Option<Value>;
    fn div(self, rhs: Self) -> Self::Output {
        println!("diving literal {self:?} to literal {rhs:?}");
        match (self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Some(Value::Number(n1.div(n2))),
            _ => None,
        }
    }
}
impl Div<Node> for Value {
    type Output = Option<Node>;
    fn div(self, rhs: Node) -> Self::Output {
        println!("dividing literal {self:?} by expression {rhs:?}");
        rhs.eval().and_then(|rhs| self.div(rhs)).map(Node::Literal)
    }
}
impl Mul for Value {
    type Output = Option<Value>;
    fn mul(self, rhs: Self) -> Self::Output {
        println!("muling literal {self:?} to literal {rhs:?}");
        match (self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Some(Value::Number(n1.mul(n2))),
            _ => None,
        }
    }
}
impl Mul<Node> for Value {
    type Output = Option<Node>;
    fn mul(self, rhs: Node) -> Self::Output {
        println!("Multiplying literal {self:?} by expression {rhs:?}");
        rhs.eval().and_then(|rhs| self.mul(rhs)).map(Node::Literal)
    }
}
impl Neg for Value {
    type Output = Option<f32>;
    fn neg(self) -> Self::Output {
        println!("Negating literal-expression {self:?}");
        match self {
            Self::Number(f) => Some(f.neg()),
            _ => None,
        }
    }
}
impl Not for Value {
    type Output = bool;
    fn not(self) -> Self::Output {
        match self {
            Self::String(StringValue(s)) => s.is_empty(),
            Self::Number(f) => !(f != 0.),
            Self::Boolean(b) => b.not(),
        }
    }
}
