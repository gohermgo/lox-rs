use core::fmt;
use std::{
    any,
    ops::{self, Add, Div, Mul, Neg, Not, Sub},
};
#[cfg(test)]
mod test_log {
    pub(crate) fn log_tag(tag: impl core::fmt::Display) {
        print!("[{tag:5}]")
    }
    pub(crate) fn log_string_with_tag(tag: &str, s: impl core::fmt::Display) {
        log_tag(tag);
        println!(" {s}")
    }
    pub(crate) fn log_args_with_tag(tag: &str, args: core::fmt::Arguments) {
        log_string_with_tag(tag, std::fmt::format(args))
    }
    macro_rules! expand {
        () => {{
           $crate::test_log::log_tag("");
        }};
        ($pre:literal) => {{
            $crate::test_log::log_tag($pre)
        }};
        ($pre:literal, $($arg:tt)*) => {{
            $crate::test_log::log_args_with_tag($pre, format_args!($($arg)*));
        }};

    }
    pub(crate) use expand;
    #[macro_export]
    macro_rules! warn {
        () => {
            $crate::test_log::expand!("WARN")
        };
        ($($arg:tt)*) => {
            $crate::test_log::expand!("WARN", $($arg)*);
        };
    }
    #[macro_export]
    macro_rules! info {
        ($($arg:tt)*) => {
            $crate::test_log::expand!("INFO", $($arg)*);
        };
    }
    #[macro_export]
    macro_rules! trace {
        () => {
            $crate::test_log::expand!("TRACE")
        };
        ($($arg:tt)*) => {
            $crate::test_log::expand!("TRACE", $($arg)*);
        };
    }
    #[macro_export]
    macro_rules! debug {
        ($($arg:tt)+) => {
            $crate::test_log::expand!($("DEBUG", $arg)+);
        };
    }
    #[macro_export]
    macro_rules! error {
        ($($arg:tt)+) => {
            $crate::test_log::expand!($("ERROR", $arg)+);
        };
    }
}
#[allow(unused_imports)]
#[cfg(not(test))]
use log::{debug, error, info, trace, warn};
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
            warn!("Option-expression of {} was empty", any::type_name::<T>());
            return None;
        };
        Some(*val)
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
pub use unary::{UnaryNodeOperator, UnaryOperator};
pub trait OperatorNode: fmt::Debug {
    type Output: Expression;
}
pub mod binary;
use binary::{ArithmeticOperator, BinaryExpression, BinaryOperator};
impl Add for Node {
    type Output = Node;
    fn add(self, rhs: Self) -> Self::Output {
        trace!("Adding literal {self:?} to expression {rhs:?}");
        ArithmeticOperator::Plus.express(self, rhs).into()
        // if let Node::Literal(v) = self {
        //     v.add(rhs)
        // } else {
        //     self.eval().and_then(|a| a.add(rhs))
        // }
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
    Unary {
        operand: Box<Node>,
        operator:
            Box<dyn UnaryNodeOperator<A = Node, Output = ops::ControlFlow<literal::Value, Node>>>,
    },
    Binary {
        operand_a: Box<Node>,
        operand_b: Box<Node>,
        operator: Box<
            dyn BinaryOperator<A = Node, B = Node, Output = ops::ControlFlow<literal::Value, Node>>,
        >,
    },
    Grouping(Box<Node>),
}
impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(v) => f.write_fmt(format_args!("{v:?}")),
            Self::Grouping(e) => f.debug_tuple("GroupingExpression").field(e).finish(),
            Self::Unary { operand, operator } => {
                f.write_fmt(format_args!("{:?}{:?}", operand, operator))
            }
            Self::Binary {
                operand_a,
                operand_b,
                operator,
            } => f.write_fmt(format_args!(
                "{:?} {:?} {:?}",
                operand_a, operator, operand_b
            )),
        }
    }
}
pub trait ExpressionStep {
    type Output;
    type Next: ExpressionStep<Output = Self::Output>;
    fn step(&self) -> Option<ops::ControlFlow<Self::Output, Self::Next>>;
    #[inline]
    fn evaluate(&self) -> Option<Self::Output> {
        match self.step()? {
            ops::ControlFlow::Continue(c) => c.evaluate(),
            ops::ControlFlow::Break(b) => Some(b),
        }
    }
}
impl ExpressionStep for literal::Value {
    type Output = literal::Value;
    type Next = Node;
    fn step(&self) -> Option<ops::ControlFlow<Self::Output, Self::Next>> {
        Some(ops::ControlFlow::Break(self.clone()))
    }
}
impl<B, C> ExpressionStep for ops::ControlFlow<B, C>
where
    B: Clone,
    C: Expression,
    <C as Expression>::Output: ExpressionStep<Output = B>,
{
    type Output = B;
    type Next = <<C as Expression>::Output as ExpressionStep>::Next;
    fn step(&self) -> Option<ops::ControlFlow<Self::Output, Self::Next>> {
        match self {
            ops::ControlFlow::Break(b) => Some(ops::ControlFlow::Break(b.clone())),
            ops::ControlFlow::Continue(c) => c.eval().as_ref().and_then(ExpressionStep::step),
        }
    }
}
impl ExpressionStep for Node {
    type Output = literal::Value;
    type Next = Node;
    fn step(&self) -> Option<ops::ControlFlow<Self::Output, Self::Next>> {
        error!("STEPPING");
        match self {
            Self::Grouping(n) => n.step(),
            Self::Unary { operand, operator } => {
                let value = operand.evaluate()?;
                operator.identity()(value)
            }
            Self::Binary {
                operand_a,
                operand_b,
                operator,
            } => {
                let a = operand_a.evaluate()?;
                let b = operand_b.evaluate()?;
                operator.identity()(a, b)
            }
            Self::Literal(v) => v.evaluate().map(ops::ControlFlow::Break),
        }
    }
}
impl<T, N, E> Expression for E
where
    E: ExpressionStep<Output = T, Next = N>,
    N: ExpressionStep<Output = T, Next = N>,
{
    type Output = T;
    #[inline]
    fn eval(&self) -> Option<Self::Output> {
        let next = self.step()?;
        match next {
            ops::ControlFlow::Continue(c) => c.evaluate(),
            ops::ControlFlow::Break(b) => Some(b),
        }
    }
}
pub trait Evaluate {
    type Output;
    type Next: Evaluate<Output = Self::Output>;
    fn next(&self) -> Option<ops::ControlFlow<Self::Output, Self::Next>>;
    fn finish(&self) -> Option<Self::Output> {
        let next = self.next()?;
        match next {
            ops::ControlFlow::Continue(next) => next.finish(),
            ops::ControlFlow::Break(out) => Some(out),
        }
    }
}
impl Evaluate for Node {
    type Output = literal::Value;
    type Next = Node;
    fn next(&self) -> Option<ops::ControlFlow<Self::Output, Self::Next>> {
        match self {
            Self::Grouping(a) => a.next(),
            Self::Literal(v) => Some(ops::ControlFlow::Break(v.clone())),
            Self::Unary { operand, operator } => {
                debug!("Evaluating unary-node {self:?}");
                match operand.as_ref() {
                    Node::Literal(v) => operator.identity()(v.clone()),
                    other => other.evaluate().and_then(operator.identity()),
                }
            }
            Self::Binary {
                operand_a,
                operand_b,
                operator,
            } => {
                debug!("Evaluating binary-node {self:?}");
                match operand_a.as_ref() {
                    Node::Literal(a) => operand_b.as_ref().evaluate().zip(Some(a.clone())),
                    other => other
                        .evaluate()
                        .and_then(|a| operand_b.as_ref().evaluate().map(|b| (a, b))),
                }
                .and_then(|(a, b)| operator.identity()(a, b))
            }
        }
    }
}
impl Node {
    //     fn evaluate(&self) -> Option<literal::Value> {
    //         // trace!("Evaluating Node {self:?}");
    //         match self {
    //             Self::Grouping(a) => a.eval(),
    //             Self::Literal(v) => {
    //                 // debug!("Evaluated node to literal {v:?}");
    //                 Some(v.clone())
    //             }
    //             Self::Unary { operand, operator } => {
    //                 debug!("Evaluating unary-node {self:?}");
    //                 match operand.as_ref() {
    //                     Node::Literal(v) => operator.identity()(v.clone()),
    //                     other => other.eval().and_then(operator.identity()),
    //                 }
    //                 .and_then(|value| match value {
    //                     ops::ControlFlow::Continue(c) => c.evaluate(),
    //                     ops::ControlFlow::Break(b) => Some(b),
    //                 })
    //             }
    //             Self::Binary {
    //                 operand_a,
    //                 operand_b,
    //                 operator,
    //             } => {
    //                 debug!("Evaluating binary-node {self:?}");
    //                 match operand_a.as_ref() {
    //                     Node::Literal(a) => operand_b.as_ref().eval().zip(Some(a.clone())),
    //                     other => other
    //                         .eval()
    //                         .and_then(|a| operand_b.as_ref().eval().map(|b| (a, b))),
    //                 }
    //                 .and_then(|(a, b)| operator.identity()(a, b))
    //                 .and_then(|value| match value {
    //                     ops::ControlFlow::Continue(c) => c.evaluate(),
    //                     ops::ControlFlow::Break(b) => Some(b),
    //                 })
    //             }
    //         }
    //         // .and_then(|node| match node {
    //         //     Node::Literal(v) => {
    //         //         info!("Result is {v:?}");
    //         //         Some(v)
    //         //     }
    //         //     other => {
    //         //         debug!("Recursing {other:?}");
    //         //         other.eval()
    //         //     }
    //         // })
    //     }
}
impl Node {
    pub fn binary(
        b: impl Into<BinaryExpression<Node, Node, ops::ControlFlow<literal::Value, Node>>>,
    ) -> Self {
        let b: BinaryExpression<Node, Node, ops::ControlFlow<literal::Value, Node>> = b.into();
        b.into()
        // Self::Binary(Box::new(b.into()))
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
    pub fn added_to_expression(self, rhs_expr: impl Into<Node>) -> Self {
        ArithmeticOperator::Plus
            .express(self, rhs_expr.into())
            .into()
    }
}

#[cfg(test)]
mod expr_node_tests {
    use super::*;
    mod add_tests {
        use super::*;
        #[test]
        fn numbers() {
            let e = Node::plus(Node::literal_value(1.0), Node::number(2.0)).evaluate();
            assert_eq!(e, Some(3.0.into()));
        }
        #[test]
        fn exprs() {
            let e =
                Node::plus(Node::negation(Node::plus(2.0, 3.0)), Node::plus(2.0, 3.0)).evaluate();
            assert_eq!(e, Some(0.0.into()))
        }
        #[test]
        fn adding_nodes() {
            let a: Node = 1.0.into();
            let b: Node = 2.0.into();
            let c = a + b;
            debug!("Node C: {c:?}");
            assert_eq!(c.evaluate(), Some(3.0.into()))
        }
    }
    mod sub_tests {}
}
