use super::{command, IntoNode};
use crate::dispatcher::InputConsumer;
use crate::generic::{Combine, Func, HList, Tuple};
use crate::graph::{RootNode, Node, NodeKind};
use crate::values::FromValues;
use std::marker::PhantomData;

pub struct Literal<B> {
    builder: B,
    value: String,
}

impl<B> CommandBuilder for Literal<B>
where
    B: CommandBuilder,
{
    type Args = B::Args;

    fn nodes(self) -> Vec<(InputConsumer, NodeKind)> {
        let mut nodes = self.builder.nodes();
        nodes.push((InputConsumer::SingleWord, NodeKind::Literal(self.value)));
        nodes
    }
}

pub struct Parameter<B, P> {
    builder: B,
    parameter: PhantomData<P>,
}

type Combined<T, U> = <<<<T as CommandBuilder>::Args as Tuple>::HList as Combine<
    <U as Tuple>::HList,
>>::Output as HList>::Tuple;

impl<B, P> CommandBuilder for Parameter<B, (P,)>
where
    B: CommandBuilder,
    P: IntoNode,
    <B::Args as Tuple>::HList: Combine<<(P,) as Tuple>::HList>,
{
    type Args = Combined<B, (P,)>;

    fn nodes(self) -> Vec<(InputConsumer, NodeKind)> {
        let mut nodes = self.builder.nodes();
        nodes.push((P::Consumer, P::NodeKind));
        nodes
    }
}

pub struct BlankBuilder;

impl CommandBuilder for BlankBuilder {
    type Args = ();

    fn nodes(self) -> Vec<(InputConsumer, NodeKind)> {
        vec![]
    }
}

impl BlankBuilder {
    pub fn new() -> Self {
        Self
    }
}

pub trait CommandBuilder {
    type Args: Tuple;

    fn nodes(self) -> Vec<(InputConsumer, NodeKind)>;

    fn build<Ctx, F>(self, callback: F) -> RootNode<Ctx>
    where
        Self: Sized,
        Ctx: 'static,
        Self::Args: FromValues<Ctx> + 'static,
        F: Func<Self::Args, Output = crate::error::Result<()>> + 'static,
    {
        let mut root = RootNode::default();
        let mut nodes = self
            .nodes()
            .into_iter()
            .map(|(consumer, kind)| Node::new(consumer, kind))
            .collect::<Vec<_>>();
        
        let command = command(callback);

        if let Some(node) = nodes.last_mut() {
            node.execute = Some(Box::new(command));
        } else {
            root.execute = Some(Box::new(command));
        }

        root.add_nodes(None, nodes);

        root
    }

    fn literal(self, lit: &str) -> Literal<Self>
    where
        Self: Sized,
    {
        Literal {
            builder: self,
            value: lit.to_owned(),
        }
    }

    fn param<T>(self) -> Parameter<Self, (T,)>
    where
        Self: Sized,
    {
        Parameter {
            builder: self,
            parameter: Default::default(),
        }
    }
}
