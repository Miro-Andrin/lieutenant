// use crate::parser::Parser;

// type NodeId = usize;
// type CommandId = usize;

// pub struct Node {
//     regex: String,

//     children: Vec<NodeId>,
//     /// Is some if command can terminate on this node
//     command: Option<CommandId>
// }

// impl<P: Parser> Node {

//     fn new(parser: P, command: Option<CommandId>) -> Self {
//         Self{
//             regex: String,
//             children: Vec::with_capacity(2),
//             command: command,
//         }
//     }
// }

// struct Tree {

//     nodes: Vec<Node>,
// }

// // regex -> Vec<(CommandId, Parser)>
