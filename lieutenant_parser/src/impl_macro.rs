/*
When parsing a command that uses Optionals, the parser might have 
to do some backtracking. This reqires some complexity that 
99% of users don't need to think about. Therefor 
we have the trait Parser, that is simpler then IterParser, but requires the
use of this macro to be converted into a IterParser.
*/
// #[macro_export]
// macro_rules! impl_parser {
//     ($type_name:ident) => {

//         impl<World> crate::IterParser<World> for $type_name<World> where $type_name<World> : crate::Parser<World=World>{
            
//             type State = ();
//             type Extract = <Self as Parser>::Extract;
            
//             fn iter_parse<'p>(
//                 &self,
//                 world: &World,
//                 state: Self::State,
//                 input: &'p str,
//             ) -> (
//                 Result<(Self::Extract, &'p str), crate::ParseError<'p>>,
//                 Option<Self::State>,
//             ) {
//                 (self.parse(world,input), None)
//             }

//             fn regex(&self) -> String {
//                 <Self as Parser>::regex(self)
//             }
//         }

//     };
// }




