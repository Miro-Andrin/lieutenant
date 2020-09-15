use crate::command::Command;
use crate::graph::{GraphMerge, NodeKind, RootNode};
use crate::values::{Value, Values};
use crate::{Error, Result, SyntaxError};
use std::ops::{Index, IndexMut, RangeInclusive};

/// A command dispatcher.
///
/// This dispatcher maintains a directed graph where
/// each node represents an argument to a command.
///
/// # Algorithm
/// The dispatcher internally keeps a data structure very
/// similar to the one found at https://wiki.vg/Command_Data#Parsers.
/// It keeps a graph of nodes, each of which has some parameters defining
/// how it should be parsed.
///
/// The first step in parsing command is called _resolving_ it. In this
/// step, the dispatcher does a traversal of the command graph while parsing
/// the input, until the input is empty and it has reached an executable node.
/// At this point, it invokes the `execute` function at the final node, passing
/// it the full command input.
///
/// Critically, the above step does't actually parse command arguments into
/// their final types: instead, it only knows about three types of argumentsCommandspatcher
/// to be able to determine which nodes to follow.
///
/// The _actual_ parsing is handled by the command's `execute()` function, which takes
/// the whole command input as a raw string. It's free to do whatever it likes to parse
/// the command, and then it uses that parsed data to do whatever it needs to do.
///
/// The benefit of the above process is this: the dispatcher doesn't have to worry about
/// parameter types and such; all it needs is some C-like raw data. Thereby we avoid
/// using generics and fancy dynamic dispatch, both of which would hinder the development
/// of a WASM command API.
///
/// Note that the `Node` API isn't intended to be used directly by users. Instead, the `command`
/// proc macro (unimplemented) and the `warp`-like builder API (unimplemented, TODO - Defman)
/// provide a convenient, elegant abstraction over raw command nodes.
pub struct CommandDispatcher<Ctx> {
    graph: RootNode<Ctx>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputConsumer {
    /// Consume until we reach a space or the end of input.
    SingleWord,
    /// If the input begins with a quote (single or double),
    /// then parse a string until the end quote. Otherwise,
    /// consume input as if this were `SingleWord`.
    Quoted,
    /// Consume all remaining input.
    Greedy,
}

/// The function used to execute a command.
pub type CommandFn<Ctx> = fn(ctx: &mut Ctx, input: &mut Values) -> Result<()>;

impl<Ctx> Default for CommandDispatcher<Ctx> {
    fn default() -> Self {
        Self::new()
    }
}

impl InputConsumer {
    /// Consumes the provided string according to the rules
    /// defined by this `InputConsumer`. After this call returns `Ok`,
    /// `input` will point to the remaining input and the returned
    /// string is the consumed input.
    pub fn consume<'a>(self, input: &mut &'a str) -> Result<&'a str> {
        match self {
            InputConsumer::SingleWord => {
                let space = find_space_position(input);
                let consumed = &input[..space];
                *input = &input[space..].trim_start();
                Ok(consumed)
            }
            InputConsumer::Quoted => {
                let mut chars = input.char_indices();
                while let Some((i, c)) = chars.next() {
                    todo!()
                }
                todo!()
            }
            InputConsumer::Greedy => {
                let consumed = *input;
                *input = &input[input.len()..];
                Ok(consumed)
            }
        }
    }
}

fn find_space_position(input: &str) -> usize {
    let space = input.char_indices().skip_while(|(_, c)| *c != ' ').next();

    if let Some((index, _)) = space {
        index
    } else {
        // all remaining input
        input.len()
    }
}

impl<Ctx> CommandDispatcher<Ctx> {
    /// Creates a new `CommandDispatcher` with no nodes.
    pub fn new() -> Self {
        Self {
            graph: Default::default(),
        }
    }

    /// Parses and executes a command.
    // TODO
    pub fn execute(&self, full_command: &str, ctx: &mut Ctx) -> Result<()> {
        let command = full_command;

        let mut node_stack = vec![(command, self.graph.children.clone())];
        let mut values = vec![];

        // Depth-first search to determine which node to execute.
        while let Some((mut command, mut node_ids)) = node_stack.pop() {
            while let Some(node_id) = node_ids.pop() {
                let mut command = command.clone();
                let node = &self.graph[node_id];
                let node_input = node.consumer.consume(&mut command)?;

                match &node.kind {
                    NodeKind::Literal(lit) => {
                        if node_input != lit {
                            continue;
                        }
                    }
                    NodeKind::Argument { parser } => match parser.parse(node_input) {
                        Ok(value) => values.push(value),
                        Err(_) => todo!(),
                    },
                }

                // If there's no remaining input, then we execute this node.
                if command.is_empty() {
                    match &node.execute {
                        Some(execute) => {
                            return execute.invoke(ctx, &mut (&values).into()).unwrap()
                        }
                        None => return Err(Error::Syntax(SyntaxError::MissingArgument)),
                    }
                }

                // Push the node's children to the stack.
                node_stack.push((command, node.children.clone()));
            }
        }

        Err(Error::Syntax(SyntaxError::MissingArgument))
    }
}

impl<Ctx> GraphMerge for CommandDispatcher<Ctx> {
    fn merge(&mut self, other: Self) {
        self.graph.merge(other.graph)
    }
}

impl<Ctx> GraphMerge<RootNode<Ctx>> for CommandDispatcher<Ctx> {
    fn merge(&mut self, other: RootNode<Ctx>) {
        self.graph.merge(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_single_word() {
        let mut s = &mut "input after space";
        assert_eq!(InputConsumer::SingleWord.consume(&mut s).unwrap(), "input");
        assert_eq!(*s, " after space");
    }

    #[test]
    fn consume_single_word_without_trailing_space() {
        let mut s = &mut "input";
        assert_eq!(InputConsumer::SingleWord.consume(&mut s).unwrap(), "input");
        assert!(s.is_empty());
    }

    #[test]
    fn consume_quoted() {
        let mut s = &mut "\"in quotes\" not in quotes";
        assert_eq!(InputConsumer::Quoted.consume(&mut s).unwrap(), "in quotes");
        assert_eq!(*s, " not in quotes");
    }

    #[test]
    fn consume_quoted_without_quotes() {
        let mut s = &mut "not in quotes";
        assert_eq!(InputConsumer::Quoted.consume(&mut s).unwrap(), "not");
        assert_eq!(*s, " in quotes");
    }

    #[test]
    fn undelimited_quote() {
        let mut s = &mut "\"not delimited";
        let err = InputConsumer::Quoted.consume(&mut s).unwrap_err();
        assert!(matches!(
            err,
            Error::Syntax(SyntaxError::UnterminatedString)
        ));
    }

    #[test]
    fn command_test() {
        // let s = &mut "tp 4";
        // let mut dispatcher = CommandDispatcher::new();
        // let tp = Node {
        //     consumer: InputConsumer::SingleWord,
        //     kind: NodeKind::Literal("tp".to_owned()),
        //     execute: None,
        //     children: vec![],
        // };

        // let x = Node {
        //     consumer: InputConsumer::SingleWord,
        //     kind: NodeKind::Argument {
        //         parser: ParserKind::IntRange,
        //     },
        //     execute: Some(teleport),
        //     children: vec![],
        // };

        // let y = Node {
        //     consumer: InputConsumer::SingleWord,
        //     kind: NodeKind::Argument {
        //         parser: ParserKind::IntRange,
        //     },
        //     execute: Some(teleport),
        //     children: vec![],
        // };

        // let z = Node {
        //     consumer: InputConsumer::SingleWord,
        //     kind: NodeKind::Argument {
        //         parser: ParserKind::IntRange,
        //     },
        //     execute: Some(teleport),
        //     children: vec![],
        // };

        // let id = dispatcher.add_node(None, tp);
        // let id = dispatcher.add_node(Some(id), x);
        // let id = dispatcher.add_node(Some(id), y);
        // let _ = dispatcher.add_node(Some(id), z);

        // let result = dispatcher.execute(s, &mut ());
        // dbg!(&result);
    }

    fn teleport<Ctx>(ctx: &mut Ctx, values: &mut Values) -> Result<()> {
        use crate::command;
        // let tp = command(|n: i32| println!("n * n = {}", n * n));
        // tp.invoke(ctx, values);
        Ok(())
    }
}
