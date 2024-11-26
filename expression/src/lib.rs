use core::fmt;
use std::ops::{Add, Div, Mul, Neg, Not, Sub};
pub trait Expression {
    type Output;
    fn eval(&self) -> Option<Self::Output>;
}
impl<T> Expression for Option<T>
where
    T: Expression + Copy,
{
    type Output = T;
    fn eval(&self) -> Option<Self::Output> {
        let Some(ref val) = self else {
            return None;
        };

        Some(*val)
    }
}
impl<T> Expression for Box<T>
where
    T: Expression,
{
    type Output = <T as Expression>::Output;
    fn eval(&self) -> Option<Self::Output> {
        T::eval(self)
    }
}
impl<T> Expression for Box<dyn Fn() -> T>
where
    T: Expression,
{
    type Output = <T as Expression>::Output;
    fn eval(&self) -> Option<Self::Output> {
        self().eval()
    }
}
mod literal;
mod unary;
use unary::UnaryExpression;
pub use unary::{UnaryNodeOperator, UnaryOperator};
pub trait OperatorNode: fmt::Debug {
    type Output: Expression;
}
pub mod binary;
use binary::{ArithmeticOperator, BinaryExpression, BinaryOperator};
impl Add for Node {
    type Output = Option<Node>;
    fn add(self, rhs: Self) -> Self::Output {
        println!("Adding expression {self:?} to expression {rhs:?}");
        if let Node::Literal(v) = self {
            v.add(rhs)
        } else {
            self.eval().and_then(|a| a.add(rhs)).into()
            // self.eval()
            //     .map(Node::Literal)
            //     .map(|a| ArithmeticOperator::Plus.express(a, rhs).into())
        }
    }
}
impl Sub for Node {
    type Output = Option<Node>;
    fn sub(self, rhs: Self) -> Self::Output {
        println!("Subtracting expression {rhs:?} from expression {self:?}");
        if let Node::Literal(v) = self {
            v.sub(rhs)
        } else {
            self.eval().and_then(|a| a.sub(rhs)).into()
        }
    }
}
impl Div for Node {
    type Output = Option<Node>;
    fn div(self, rhs: Self) -> Self::Output {
        println!("divtracting expression {rhs:?} from expression {self:?}");
        if let Node::Literal(v) = self {
            v.div(rhs)
        } else {
            self.eval().and_then(|a| a.div(rhs)).into()
        }
    }
}
impl Mul for Node {
    type Output = Option<Node>;
    fn mul(self, rhs: Self) -> Self::Output {
        println!("Multiplying expression {rhs:?} from expression {self:?}");
        if let Node::Literal(v) = self {
            v.mul(rhs)
        } else {
            self.eval().and_then(|a| a.mul(rhs)).into()
        }
    }
}
impl Neg for Node {
    type Output = Option<f32>;
    fn neg(self) -> Self::Output {
        println!("Negating expression {self:?}");
        if let Node::Literal(v) = self {
            v.neg()
        } else {
            self.eval().and_then(Neg::neg)
        }
    }
}
impl Not for Node {
    type Output = Option<bool>;
    fn not(self) -> Self::Output {
        println!("Negating expression {self:?}");
        if let Node::Literal(v) = self {
            Some(v.not())
        } else {
            self.eval().map(Not::not)
        }
    }
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Node::Literal(l1) => l1.eq(other),
            other => other.eval().is_some_and(|e| e.eq(other)),
        }
    }
}
pub enum Node {
    Literal(literal::Value),
    Unary(Box<UnaryExpression<Node, Node>>),
    Binary(Box<BinaryExpression<Node, Node, Node>>),
    Grouping(Box<Node>),
}
impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(v) => f.debug_tuple("LiteralExpression").field(v).finish(),
            Self::Unary(u) => f.write_fmt(format_args!("{u:?}")),
            Self::Binary(b) => f.write_fmt(format_args!("{b:?}")),
            Self::Grouping(e) => f.debug_tuple("GroupingExpression").field(e).finish(),
        }
    }
}
impl Expression for Node {
    type Output = literal::Value;
    fn eval(&self) -> Option<Self::Output> {
        println!("Evaluating ExpressionNode {self:?}");
        match self {
            Self::Literal(v) => Some(Self::Literal(v.clone())),
            Self::Binary(b) => b.eval(),
            Self::Unary(u) => u.eval(),
            Self::Grouping(a) => a.eval().map(Node::Literal),
        }
        .and_then(|node| match node {
            Node::Literal(v) => {
                println!("Result is {v:?}");
                Some(v)
            }
            other => other.eval(),
        })
    }
}
impl Node {
    pub fn binary(b: impl Into<BinaryExpression<Node, Node, Node>>) -> Self {
        Self::Binary(Box::new(b.into()))
    }
    pub fn plus(a: impl Into<Node>, b: impl Into<Node>) -> Self {
        ArithmeticOperator::Plus.express(a.into(), b.into()).into()
    }
    pub fn minus(a: impl Into<Node>, b: impl Into<Node>) -> Self {
        ArithmeticOperator::Minus.express(a.into(), b.into()).into()
    }
    pub fn divides(a: impl Into<Node>, b: impl Into<Node>) -> Self {
        ArithmeticOperator::Divides
            .express(a.into(), b.into())
            .into()
    }
    pub fn times(a: impl Into<Node>, b: impl Into<Node>) -> Self {
        ArithmeticOperator::Times.express(a.into(), b.into()).into()
    }
    pub fn literal_value(value: impl Into<literal::Value>) -> Self {
        Self::Literal(value.into())
    }
    pub fn number(value: impl Into<f32>) -> Self {
        Self::literal_value(value.into())
    }
    pub fn negation(a: impl Into<Node>) -> Self {
        UnaryOperator::Neg.express(a.into()).into()
    }
    pub fn inversion(a: impl Into<Node>) -> Self {
        UnaryOperator::Not.express(a.into()).into()
    }
    #[inline]
    pub fn as_literal(&self) -> Option<&literal::Value> {
        match self {
            Node::Literal(l) => Some(l),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        self.as_literal().and_then(literal::Value::as_bool)
    }
}

#[cfg(test)]
mod expr_node_tests {
    use super::*;
    mod add_tests {
        use super::*;

        #[test]
        fn numbers() {
            println!("\n===\nnumbers");
            let e = Node::plus(Node::literal_value(1.0), Node::number(2.0)).eval();
            assert_eq!(e, Some(3.0.into()));
        }
        #[test]
        fn exprs() {
            println!("\n===\nexprs");
            let e = Node::plus(Node::negation(Node::plus(2.0, 3.0)), Node::plus(2.0, 3.0)).eval();
            assert_eq!(e, Some(0.0.into()))
        }
    }
}
