// use crate::{generic::Func, parser::Parser};

// pub struct Command<P, F> {
//     pub(crate) parser: P,
//     pub(crate) callback: F,
// }

// impl<P, F> Command<P, F>
// where
//     P: Parser,
//     F: Func<P::Extract>,
// {
//     pub fn call(&self, mut input: &str) -> anyhow::Result<F::Output> {
//         let (args, _output) = self.parser.parse(&mut input);
//         Ok(self.callback.call(args))
//     }
// }
