use anyhow::{Error, bail};

use crate::parser::IterParser;



struct I32Parser {}

impl IterParser for I32Parser {
    type Extract = (i32,);

    type ParserState = ();

    fn parse<'p>(
        &self,
        state: Self::ParserState,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str)>,
        Option<Self::ParserState>,
    ) {
        let number = 0;
        // Consume digit from head of input
        

        let mut iter = input.char_indices();
        let mut index = 0;
        if let Some((i,c)) = iter.next() {
            if c == '+' || c == '-'  {
                index = i;
            }
        } else {
            return (Err(anyhow::anyhow!("Empty input")), None)
        }

        for (i, c) in iter {
            if !c.is_digit(10) {
                break
            }
            index = i
        }

        match input[0..=index].parse::<i32>() {
            Ok(number) => {
                return (Ok(((number,),input)), None)
            }
            Err(_) => {
                return (Err(anyhow::anyhow!("Not a number")), None)
            }
        };
        

    }

    fn regex(&self) -> String {
        "(+|-)?\\d+".into()
    }
}


