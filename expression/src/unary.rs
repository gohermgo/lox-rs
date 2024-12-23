use crate::{literal, Expression, Node, OperatorNode};
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
            Self::Not => Box::new(|a| {
                Some(a.not())
                    .map(literal::Value::Boolean)
                    .map(Node::Literal)
            }),
            Self::Neg => Box::new(|a| a.neg().map(literal::Value::Number).map(Node::Literal)),
        }
    }
}
#[derive(Debug)]
pub struct UnaryExpression<O, Output>
where
    O: Expression,
    Output: Expression,
{
    operand: O,
    operator: Box<dyn UnaryNodeOperator<A = O, Output = Output>>,
}
impl From<UnaryExpression<Node, Node>> for Node {
    fn from(value: UnaryExpression<Node, Node>) -> Self {
        Node::Unary(Box::new(value))
    }
}
impl<O, Output> Expression for UnaryExpression<O, Output>
where
    O: Expression + fmt::Debug,
    Output: Expression + fmt::Debug,
{
    type Output = Output;
    fn eval(&self) -> Option<Self::Output> {
        println!("Evaluating UnaryExpression {self:?}");
        self.operand
            .eval()
            .and_then(|a| self.operator.identity()(a))
    }
}
