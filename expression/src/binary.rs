use crate::{literal::Value, Expression, Node, OperatorNode};
use core::{
    fmt,
    ops::{Add, Div, Mul, Sub},
};
pub trait BinaryOperator: OperatorNode {
    type A: Expression;
    type B: Expression;
    fn identity(
        &self,
    ) -> Box<
        dyn Fn(
            <Self::A as Expression>::Output,
            <Self::B as Expression>::Output,
        ) -> Option<<Self as OperatorNode>::Output>,
    >;
    fn express(
        self,
        a: Self::A,
        b: Self::B,
    ) -> BinaryExpression<Self::A, Self::B, <Self as OperatorNode>::Output>
    where
        Self: Sized + 'static,
        Self::A: 'static,
        Self::B: 'static,
        Self::Output: 'static,
    {
        BinaryExpression {
            operand_a: a,
            operand_b: b,
            operator: Box::new(self),
        }
    }
}
#[derive(Debug)]
pub struct BinaryExpression<A, B, Output>
where
    A: Expression,
    B: Expression,
    Output: Expression,
{
    operand_a: A,
    operand_b: B,
    operator: Box<dyn BinaryOperator<A = A, B = B, Output = Output>>,
}
impl From<BinaryExpression<Node, Node, Node>> for Node {
    fn from(value: BinaryExpression<Node, Node, Node>) -> Self {
        Node::Binary(Box::new(value))
    }
}
impl<A, B, Out> Expression for BinaryExpression<A, B, Out>
where
    A: Expression + fmt::Debug,
    B: Expression + fmt::Debug,
    Out: Expression + fmt::Debug,
{
    type Output = Out;
    fn eval(&self) -> Option<Self::Output> {
        self.operand_a.eval().and_then(|a| {
            self.operand_b.eval().and_then(|b| {
                let res = self.operator.identity()(a, b);
                println!("Evaluated {self:#?} as {res:?}");
                res
            })
        })
    }
}
#[derive(Debug)]
pub enum ArithmeticOperator {
    Plus,
    Minus,
    Divides,
    Times,
}
impl OperatorNode for ArithmeticOperator {
    type Output = Node;
}
impl Add<Value> for Node {
    type Output = Node;
    fn add(self, rhs: Value) -> Self::Output {
        Node::binary(ArithmeticOperator::Plus.express(self, Node::Literal(rhs)))
    }
}
impl BinaryOperator for ArithmeticOperator {
    type A = Node;
    type B = Node;
    fn identity(
        &self,
    ) -> Box<dyn Fn(<Node as Expression>::Output, <Node as Expression>::Output) -> Option<Node>>
    {
        match self {
            Self::Plus => Box::new(|a, b| a.add(b).map(Node::Literal)),
            Self::Minus => Box::new(|a, b| a.sub(b).map(Node::Literal)),
            Self::Divides => Box::new(|a, b| a.div(b).map(Node::Literal)),
            Self::Times => Box::new(|a, b| a.mul(b).map(Node::Literal)),
        }
    }
}
#[cfg(test)]
mod arithmetic_op_tests {
    use super::*;
    #[test]
    fn add() {
        assert_eq!(
            ArithmeticOperator::Plus
                .express(15.0.into(), 5.0.into())
                .eval(),
            Some(20.0.into())
        )
    }
    #[test]
    fn div() {
        assert_eq!(
            ArithmeticOperator::Divides
                .express(15.0.into(), 5.0.into())
                .eval(),
            Some(3.0.into())
        )
    }
    #[test]
    fn add_then_div() {
        assert!(ArithmeticOperator::Divides
            .express(
                ArithmeticOperator::Plus
                    .express(15.0.into(), 5.0.into())
                    .into(),
                4.0.into(),
            )
            .eval()
            .is_some_and(|l| l.eq(&5.0.into())));
    }
}
#[derive(Debug)]
pub enum EqualityOperator {
    Eq,
    Ne,
}
impl OperatorNode for EqualityOperator {
    type Output = Node;
}
impl BinaryOperator for EqualityOperator {
    type A = Node;
    type B = Node;
    fn identity(
        &self,
    ) -> Box<
        dyn Fn(
            <Self::A as Expression>::Output,
            <Self::B as Expression>::Output,
        ) -> Option<Self::Output>,
    > {
        match self {
            Self::Eq => Box::new(|a, b| Some(a.eq(&b)).map(Value::Boolean).map(Node::Literal)),
            Self::Ne => Box::new(|a, b| Some(a.ne(&b)).map(Value::Boolean).map(Node::Literal)),
        }
    }
}
#[cfg(test)]
mod equality_operator_node_tests {
    use super::*;
    #[test]
    fn string_to_string_equality() {
        assert!(EqualityOperator::Eq
            .express("Test".into(), "Test".into(),)
            .eval()
            .as_ref()
            .and_then(Node::as_bool)
            .is_some_and(std::convert::identity));
        assert!(EqualityOperator::Eq
            .express("Test".into(), "Testing".into(),)
            .eval()
            .as_ref()
            .and_then(Node::as_bool)
            .map(std::ops::Not::not)
            .is_some_and(std::convert::identity))
    }
    #[test]
    fn number_to_number_equality() {
        assert!(EqualityOperator::Eq
            .express(10.0.into(), 10.0.into(),)
            .eval()
            .as_ref()
            .and_then(Node::as_bool)
            .is_some_and(std::convert::identity));
        assert!(EqualityOperator::Eq
            .express(10.0.into(), 0.0.into(),)
            .eval()
            .as_ref()
            .and_then(Node::as_bool)
            .map(std::ops::Not::not)
            .is_some_and(std::convert::identity))
    }
    #[test]
    fn bool_to_bool_equality() {
        assert!(EqualityOperator::Eq
            .express(false.into(), false.into(),)
            .eval()
            .as_ref()
            .and_then(Node::as_bool)
            .is_some_and(std::convert::identity));
        assert!(EqualityOperator::Eq
            .express(true.into(), false.into())
            .eval()
            .as_ref()
            .and_then(Node::as_bool)
            .map(std::ops::Not::not)
            .is_some_and(std::convert::identity))
    }
}
#[derive(Debug)]
pub enum OrderingOperatorNode {
    Lt { equal: bool },
    Gt { equal: bool },
}
impl OperatorNode for OrderingOperatorNode {
    type Output = Node;
}
impl BinaryOperator for OrderingOperatorNode {
    type A = Node;
    type B = Node;
    fn identity(
        &self,
    ) -> Box<
        dyn Fn(
            <Self::A as Expression>::Output,
            <Self::B as Expression>::Output,
        ) -> Option<Self::Output>,
    > {
        match self {
            Self::Lt { equal: true } => {
                Box::new(|a, b| Some(a.le(&b)).map(Value::Boolean).map(Node::Literal))
            }
            Self::Lt { equal: false } => {
                Box::new(|a, b| Some(a.lt(&b)).map(Value::Boolean).map(Node::Literal))
            }
            Self::Gt { equal: true } => {
                Box::new(|a, b| Some(a.ge(&b)).map(Value::Boolean).map(Node::Literal))
            }
            Self::Gt { equal: false } => {
                Box::new(|a, b| Some(a.gt(&b)).map(Value::Boolean).map(Node::Literal))
            }
        }
    }
}
#[cfg(test)]
mod ord_tests {
    use super::*;
    #[test]
    fn numbers_lt() {
        let res = OrderingOperatorNode::Lt { equal: false }
            .express(1.0.into(), 2.0.into())
            .eval();
        assert_eq!(res, Some(true.into()));
        let res = OrderingOperatorNode::Lt { equal: false }
            .express(1.0.into(), 1.0.into())
            .eval();
        assert_eq!(res, Some(false.into()));
        let res = OrderingOperatorNode::Lt { equal: true }
            .express(1.0.into(), 1.0.into())
            .eval();
        assert_eq!(res, Some(true.into()));
    }
}
