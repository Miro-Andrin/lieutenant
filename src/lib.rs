mod command;
mod generic;

use std::marker::PhantomData;
use std::{borrow::Borrow, fmt, todo};

use anyhow::{bail, Result};
use command::{And, Space};
use generic::{Func, Product, Tuple};

type NodeId = usize;
type CommandId = usize;

pub trait Validator {
    fn validate(&self, input: &mut &str) -> bool;
}

pub struct Node {
    validator: Box<dyn Validator>,
    children: Vec<NodeId>,
    command: Option<CommandId>,
}

impl Validator for Node {
    fn validate(&self, input: &mut &str) -> bool {
        self.validator.validate(input)
    }
}

impl Node {
    pub fn new<V: Validator + 'static>(validator: V) -> Self {
        Node {
            validator: Box::new(validator),
            children: vec![],
            command: None,
        }
    }

    pub fn command(mut self, command: CommandId) -> Self {
        self.command = Some(command);
        self
    }
}

#[derive(Default)]
pub struct Dispatcher {
    root: Vec<NodeId>,
    nodes: Vec<Node>,
}

impl Dispatcher {
    pub fn add(&mut self, parent: Option<NodeId>, node: Node) -> NodeId {
        let node_id = self.nodes.len();
        self.nodes.push(node);
        if let Some(parent_id) = parent {
            self.nodes[parent_id].children.push(node_id);
        }
        node_id
    }

    pub fn find(&self, input: &str) -> Option<CommandId> {
        let mut input = input;

        let mut stack = self.root.clone();
        while let Some(node_id) = stack.pop() {
            let node = &self.nodes[node_id];
            if node.validate(&mut input) {
                if input.is_empty() {
                    if let Some(command_id) = node.command {
                        return Some(command_id);
                    }
                } else {
                    stack.extend(node.children.iter());
                }
            }
        }
        None
    }
}

pub trait AddToDispatcher {
    fn add_to_dispatcher(&self, parent: Option<NodeId>, dispatcher: &mut Dispatcher) -> NodeId;
}

#[cfg(test)]
mod tests {
    use crate::{Dispatcher, AddToDispatcher, command::{Command, literal}};
    use crate::command::{CommandBuilder};

    #[test]
    fn simple() {
        let command = literal("tp")
            .arg()
            .arg()
            .arg()
            .build(|x: u32, y: u32, z: u32| move |_state: &mut u32| println!("{} {} {}", x, y, z));

        (Command::call(&command, "tp 10 11 12").unwrap())(&mut 0);

        let mut dispatcher = Dispatcher::default();
        command.add_to_dispatcher(None, &mut dispatcher);

        let commands = vec![("my_awesome_plugin", 0)];

        let command_id = dispatcher.find("tp 10 11 12");

        let (plugin_id, command_id) = commands[command_id.unwrap()];
    }
}
