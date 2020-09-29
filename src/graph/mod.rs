mod parser_kind;

use crate::{command::Command};
use crate::error::Result;
pub use parser_kind::*;
use slab::Slab;
use std::ops::{Index, IndexMut};
use crate::automaton::{NFA};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl<Ctx> Index<NodeId> for Slab<Node<Ctx>> {
    type Output = Node<Ctx>;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self[index.0]
    }
}

impl<Ctx> IndexMut<NodeId> for Slab<Node<Ctx>> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self[index.0]
    }
}

pub struct Node<Ctx> {
    /// Whether this node is a `literal` or `argument` node.
    pub kind: NodeKind,

    /// Whether this node is "executable."
    ///
    /// When the end of input is reached, the last node
    /// visited will have its execute function invoked.
    pub execute: Option<Box<dyn Command<Ctx = Ctx, Output = Result<()>>>>,

    /// Child nodes. After this node is consumed,
    /// parsing will move on to the children if there is remaining
    /// input.
    // TODO: private
    pub(crate) children: Vec<NodeId>,
}

impl<Ctx> Node<Ctx> {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            execute: None,
            children: vec![]
        }
    }
}

impl<Ctx> fmt::Debug for Node<Ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self.kind)?;
        writeln!(f, "{:?}", self.children)?;
        writeln!(f, "Executable: {}", self.execute.is_some())?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum NodeKind {
    Argument {
        /// Descriptor for the parser for this
        /// node. Used to build the Declare Commands packet.
        parser: ParserKind,
    },
    Literal(String),
}

pub struct RootNode<Ctx> {
    // TODO private
    pub(crate) nodes: Slab<Node<Ctx>>,
    // TODO, should be private
    pub(crate) children: Vec<NodeId>,
    pub(crate) execute: Option<Box<dyn Command<Ctx = Ctx, Output = Result<()>>>>,
}

impl<Ctx> fmt::Debug for RootNode<Ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self.nodes)?;
        writeln!(f, "{:?}", self.children)?;
        writeln!(f, "Executable: {}", self.execute.is_some())?;
        Ok(())
    }
}

impl<Ctx> Index<NodeId> for RootNode<Ctx> {
    type Output = Node<Ctx>;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl<Ctx> IndexMut<NodeId> for RootNode<Ctx> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}

impl<Ctx> Default for RootNode<Ctx> {
    fn default() -> Self {
        Self {
            nodes: Slab::new(),
            children: vec![],
            execute: None,
        }
    }
}

impl<Ctx> RootNode<Ctx> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_node(&mut self, parent: Option<NodeId>, node: Node<Ctx>) -> NodeId {
        let id = NodeId(self.nodes.insert(node));
        if let Some(parent) = parent.and_then(|id| self.nodes.get_mut(id.0)) {
            parent.children.push(id.clone());
        } else {
            self.children.push(id.clone());
        }
        id
    }

    pub fn add_nodes<I>(&mut self, parent: Option<NodeId>, nodes: I) -> NodeId
    where
        I: IntoIterator<Item = Node<Ctx>>,
    {
        let id = NodeId(self.nodes.len());
        let mut parent = parent;
        for node in nodes {
            parent = Some(self.add_node(parent, node));
        }
        id
    }

    pub fn remove(&mut self, node_id: NodeId) -> Node<Ctx> {
        self.nodes.remove(node_id.0)
    }
}

pub trait GraphMerge<Other = Self> {
    fn merge(&mut self, other: Other);
}

impl<Ctx> GraphMerge for RootNode<Ctx> {
    fn merge(&mut self, mut other: Self) {
        let mut other_children = vec![(None, other.children)];
        while let Some((parent, other_childrens)) = other_children.pop() {
            for other_child_id in other_childrens {
                let mut other_child = other.nodes.remove(other_child_id.0);
                let other_child_children = std::mem::replace(&mut other_child.children, vec![]);
                other.children = vec![];
                let parent = self.add_node(parent.clone(), other_child);
                other_children.push((Some(parent), other_child_children));
            }
        }
    }
}

impl From<&NodeKind> for NFA {
    fn from(kind: &NodeKind) -> Self {
        match kind {
            
         

            NodeKind::Literal(lit) => {

                NFA::literal(lit)

            }
            NodeKind::Argument { parser, .. } => NFA::from(parser),
        }
    }
}