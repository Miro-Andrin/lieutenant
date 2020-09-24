use super::{command, NodeDescriptor};
use crate::generic::{Combine, Func, HList, Tuple};
use crate::graph::{RootNode, Node, NodeKind};
use crate::values::FromValues;
use std::marker::PhantomData;

pub struct Literal<B> {
    builder: B,
    value: String,
    aliases: Vec<String>
}

impl<B> CommandBuilder for Literal<B>
where
    B: CommandBuilder,
{
    type Args = B::Args;

    fn nodes(self) -> Vec<NodeKind> {
        let mut nodes = self.builder.nodes();
        nodes.push(NodeKind::Literal(self.value));
        nodes
    }
}

impl<B> Literal<B>
where
    B: CommandBuilder
{
    /// Adds an additional alias for this literal
    pub fn alias(mut self, alias: String) -> Self {
        self.aliases.push(alias);
        self
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
    P: NodeDescriptor,
    <B::Args as Tuple>::HList: Combine<<(P,) as Tuple>::HList>,
{
    type Args = Combined<B, (P,)>;

    fn nodes(self) -> Vec<NodeKind> {
        let mut nodes = self.builder.nodes();
        nodes.push(P::NODE_KIND);
        nodes
    }
}

pub struct BlankBuilder;

impl CommandBuilder for BlankBuilder {
    type Args = ();

    fn nodes(self) -> Vec<NodeKind> {
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

    fn nodes(self) -> Vec<NodeKind>;

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
            .map(|kind| Node::new(kind))
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


    /// Adds an case insensitive literal ie. `tp` or `ban`.
    fn literal(self, lit: &str) -> Literal<Self>
    where
        Self: Sized,
    {
        Literal {
            builder: self,
            value: lit.to_owned(),
            aliases: vec![],
        }
    }

    /// Adds an paramter of type `T`
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
