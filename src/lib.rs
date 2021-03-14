mod argument;
pub mod command;
mod generic;
pub mod parser;
mod regex;



#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
mod tests {
    use std::default;

    use crate::{argument::U32Parser, command::builder::CommandBuilder, parser::{self, Literal}};
    // //use crate::command::CommandBuilder;
    // use crate::{
    //     //command::{literal, Command},
    //     AddToDispatcher, Dispatcher,
    // };

    #[test]
    fn simple() {
        let command = CommandBuilder::<Literal,u32,u32>::new().parser(U32Parser::default()).build(|x: u32| {println!("hello"); 0});
        command.call()

        // let command = literal("tp")
        //     .arg()
        //     .arg()
        //     .arg()
        //     .build(|x: u32, y: u32, z: u32| {
        //         move |_state: &mut u32| println!("Command call result: {} {} {}", x, y, z)
        //     });

        // (Command::call(&command, "tp 10 11 12").unwrap())(&mut 0);

        // let mut dispatcher = Dispatcher::default();
        // command.add_to_dispatcher(None, &mut dispatcher);
        // let command_id = dispatcher.find("tp 10 11 12");
        // assert!(command_id.is_some())
    }
    // #[test]
    // fn simple_opt() {
    //     let command = literal("tp")
    //         .opt_arg()
    //         .build(|x: Option<u32>| move |_state: &mut u32| println!("{:?}", x));

    //     (Command::call(&command, "tp 10").unwrap())(&mut 0);

    //     let mut dispatcher = Dispatcher::default();
    //     command.add_to_dispatcher(None, &mut dispatcher);

    //     let command_id = dispatcher.find("tp 10");
    //     assert!(command_id.is_some())
    // }

    // #[test]
    // fn simple_opt2() {
    //     let command = literal("tp")
    //         .opt_arg()
    //         .arg()
    //         .build(|x: Option<u32>, y: u32| move |_state: &mut u32| println!("{:?}, {:?}", x, y));

    //     (Command::call(&command, "tp 10 ").unwrap())(&mut 0);

    //     let mut dispatcher = Dispatcher::default();
    //     command.add_to_dispatcher(None, &mut dispatcher);

    //     let command_id = dispatcher.find("tp 10");
    //     assert!(command_id.is_some())
    // }
}

// #[macro_export]
// macro_rules! regex_validator {
//     ($ident:ty, $regex:literal) => {
//         impl Validator for $ident {
//             fn validate<'a, 'b>(&self, input: &'a str) -> (bool, &'b str)
//             where
//                 'a: 'b,
//             {
//                 use lazy_static::lazy_static;
//                 use regex::Regex;
//                 lazy_static! {
//                     static ref RE: Regex = Regex::new($regex).unwrap();
//                 };
//                 if let Some(m) = RE.find(input) {
//                     let input = &input[m.end()..];
//                     (true, input)
//                 } else {
//                     (false, input)
//                 }
//             }
//         }
//     };
// }
