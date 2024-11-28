use crate::{literal, trace, Expression, Node, OperatorNode};
use core::{
    fmt,
    ops::{Neg, Not},
};
use std::ops;
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
pub enum UnaryOperator {
    Not,
    Neg,
}
impl fmt::Debug for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if matches!(self, UnaryOperator::Not) {
            "!"
        } else {
            "-"
        })
    }
}
impl OperatorNode for UnaryOperator {
    type Output = ops::ControlFlow<literal::Value, Node>;
}
impl UnaryNodeOperator for UnaryOperator {
    type A = Node;
    fn identity(
        &self,
    ) -> Box<dyn Fn(<Self::A as Expression>::Output) -> Option<<Self as OperatorNode>::Output>>
    {
        match self {
            Self::Not => Box::new(|a| Some(literal::Value::Boolean(a.not()).into())),
            Self::Neg => Box::new(|a| {
                a.neg()
                    .map(literal::Value::Number)
                    .map(ops::ControlFlow::Break)
            }),
        }
    }
}
pub struct UnaryExpression<T, Output>
where
    T: Expression,
    Output: Expression,
{
    operand: T,
    operator: Box<dyn UnaryNodeOperator<A = T, Output = Output>>,
}
impl<T, Output> fmt::Debug for UnaryExpression<T, Output>
where
    T: Expression + fmt::Debug,
    Output: Expression + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}{:?}", self.operator, self.operand))
    }
}
impl From<UnaryExpression<Node, ops::ControlFlow<literal::Value, Node>>> for Node {
    fn from(value: UnaryExpression<Node, ops::ControlFlow<literal::Value, Node>>) -> Self {
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
                Some(ops::ControlFlow::Break(literal::Value::from(-1.0)))
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
            assert_eq!(
                UnaryOperator::Not.express(true.into()).eval(),
                Some(ops::ControlFlow::Break(false.into()))
            );
        }
        #[test]
        fn false_to_true() {
            assert_eq!(
                UnaryOperator::Not.express(false.into()).eval(),
                Some(ops::ControlFlow::Break(true.into()))
            );
        }
    }
}
