use crate::{debug, literal::Value, Expression, Node, OperatorNode};
use core::{
    fmt,
    ops::{Add, Div, Mul, Sub},
};
use std::ops;
type BinaryMap<A, B, Output> =
    dyn Fn(<A as Expression>::Output, <B as Expression>::Output) -> Option<Output>;
pub trait BinaryOperator: OperatorNode {
    type A: Expression;
    type B: Expression;
    fn try_call(&self, a: Self::A, b: Self::B) -> Option<<Self as OperatorNode>::Output> {
        a.eval()
            .and_then(|a| b.eval().and_then(|b| self.identity()(a, b)))
    }
    fn identity(&self) -> Box<BinaryMap<Self::A, Self::B, <Self as OperatorNode>::Output>>;
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
impl<A, B, Output> fmt::Debug for BinaryExpression<A, B, Output>
where
    A: Expression + fmt::Debug,
    B: Expression + fmt::Debug,
    Output: Expression + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{:?} {:?} {:?}",
            self.operand_a, self.operator, self.operand_b
        ))
    }
}
impl From<BinaryExpression<Node, Node, ops::ControlFlow<Value, Node>>> for Node {
    fn from(value: BinaryExpression<Node, Node, ops::ControlFlow<Value, Node>>) -> Self {
        Node::Binary {
            operand_a: Box::new(value.operand_a),
            operand_b: Box::new(value.operand_b),
            operator: value.operator,
        }
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
        debug!("Evaluating {self:?}");
        self.operand_a.eval().and_then(|a| {
            self.operand_b.eval().and_then(|b| {
                let res = self.operator.identity()(a, b);
                println!("Evaluated {self:#?} as {res:?}");
                res
            })
        })
    }
}
pub enum ArithmeticOperator {
    Plus,
    Minus,
    Divides,
    Times,
}
impl fmt::Debug for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticOperator::Plus => f.write_str("+"),
            ArithmeticOperator::Minus => f.write_str("-"),
            ArithmeticOperator::Divides => f.write_str("/"),
            ArithmeticOperator::Times => f.write_str("x"),
        }
    }
}
impl OperatorNode for ArithmeticOperator {
    type Output = ops::ControlFlow<Value, Node>;
}
impl Add<Value> for Node {
    type Output = Option<Value>;
    fn add(self, rhs: Value) -> Self::Output {
        self.eval().and_then(|a| a.add(rhs))
        // Node::binary(ArithmeticOperator::Plus.express(self, Node::Literal(rhs)))
    }
}
impl BinaryOperator for ArithmeticOperator {
    type A = Node;
    type B = Node;
    fn identity(
        &self,
    ) -> Box<
        dyn Fn(
            <Node as Expression>::Output,
            <Node as Expression>::Output,
        ) -> Option<ops::ControlFlow<Value, Node>>,
    > {
        match self {
            Self::Plus => Box::new(|a, b| a.add(b).map(ops::ControlFlow::Break)),
            Self::Minus => Box::new(|a, b| a.sub(b).map(ops::ControlFlow::Break)),
            Self::Divides => Box::new(|a, b| a.div(b).map(ops::ControlFlow::Break)),
            Self::Times => Box::new(|a, b| a.mul(b).map(ops::ControlFlow::Break)),
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
            Some(ops::ControlFlow::Break(20.0.into()))
        )
    }
    #[test]
    fn div() {
        assert_eq!(
            ArithmeticOperator::Divides
                .express(15.0.into(), 5.0.into())
                .eval(),
            Some(ops::ControlFlow::Break(3.0.into()))
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
            .is_some_and(|l| l.eq(&ops::ControlFlow::Break(5.0.into()))));
    }
}
pub enum EqualityOperator {
    Eq,
    Ne,
}
impl fmt::Debug for EqualityOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EqualityOperator::Eq => f.write_str("=="),
            EqualityOperator::Ne => f.write_str("!="),
        }
    }
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
            Self::Eq => Box::new(|a, b| Some(Value::Boolean(a.eq(&b)).into())),
            Self::Ne => Box::new(|a, b| Some(Value::Boolean(a.ne(&b)).into())),
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
pub enum OrderingOperatorNode {
    Lt { equal: bool },
    Gt { equal: bool },
}
impl fmt::Debug for OrderingOperatorNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderingOperatorNode::Lt { equal } => f.write_str(if *equal { "<=" } else { "<" }),
            OrderingOperatorNode::Gt { equal } => f.write_str(if *equal { ">=" } else { ">" }),
        }
    }
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
            Self::Lt { equal: true } => Box::new(|a, b| Some(Value::Boolean(a.le(&b)).into())),
            Self::Lt { equal: false } => Box::new(|a, b| Some(Value::Boolean(a.lt(&b)).into())),
            Self::Gt { equal: true } => Box::new(|a, b| Some(Value::Boolean(a.ge(&b)).into())),
            Self::Gt { equal: false } => Box::new(|a, b| Some(Value::Boolean(a.gt(&b)).into())),
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
