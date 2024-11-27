use crate::{literal, trace, Expression, Node, OperatorNode};
use core::{
    fmt,
    ops::{Neg, Not},
};
pub trait UnaryNodeOperator: OperatorNode {
    type A: Expression;
    fn identity(
        &self,
    ) -> Box<dyn Fn(<Self::A as Expression>::Output) -> Option<<Self as OperatorNode>::Output>>;
    fn express(self, a: Self::A) -> UnaryExpression<Self::A, Self::Output>
    where
        Self: Sized + 'static,
        Self::A: 'static,
        Self::Output: 'static,
    {
        UnaryExpression {
            operand: a,
            operator: Box::new(self),
        }
    }
}
#[derive(Debug)]
pub enum UnaryOperator {
    Not,
    Neg,
}
impl OperatorNode for UnaryOperator {
    type Output = Node;
}
impl UnaryNodeOperator for UnaryOperator {
    type A = Node;
    fn identity(&self) -> Box<dyn Fn(<Box<Node> as Expression>::Output) -> Option<Node>> {
        match self {
            Self::Not => Box::new(|a| Some(literal::Value::Boolean(a.not()).into())),
            Self::Neg => Box::new(|a| a.neg().map(literal::Value::Number).map(Node::Literal)),
        }
    }
}
#[derive(Debug)]
pub struct UnaryExpression<T, Output>
where
    T: Expression,
    Output: Expression,
{
    operand: T,
    operator: Box<dyn UnaryNodeOperator<A = T, Output = Output>>,
}
impl From<UnaryExpression<Node, Node>> for Node {
    fn from(value: UnaryExpression<Node, Node>) -> Self {
        Node::Unary {
            operand: Box::new(value.operand),
            operator: value.operator,
        }
    }
}
impl<O, Output> Expression for UnaryExpression<O, Output>
where
    O: Expression + fmt::Debug,
    Output: Expression + fmt::Debug,
{
    type Output = Output;
    fn eval(&self) -> Option<Self::Output> {
        self.operand.eval().and_then(|a| {
            let res = self.operator.identity()(a);
            trace!("Evaluated {self:?} to {res:?}");
            res
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    mod neg {
        use super::*;
        #[test]
        fn number() {
            assert_eq!(
                UnaryOperator::Neg.express(1.0.into()).eval(),
                Some(Node::from(-1.0))
            )
        }
        #[test]
        fn string() {
            assert!(UnaryOperator::Neg.express("test".into()).eval().is_none())
        }
        #[test]
        fn boolean() {
            assert!(UnaryOperator::Neg.express(true.into()).eval().is_none())
        }
    }
    mod not {
        use super::*;
        #[test]
        fn true_to_false() {
            assert!(UnaryOperator::Not
                .express(true.into())
                .eval()
                .as_ref()
                .and_then(Node::as_bool)
                .is_some_and(|v| !v));
        }
        #[test]
        fn false_to_true() {
            assert!(UnaryOperator::Not
                .express(false.into())
                .eval()
                .as_ref()
                .and_then(Node::as_bool)
                .is_some_and(std::convert::identity));
        }
    }
}
