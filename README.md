# Whats Lieutenant?
Leutenant started out as a rewrite of mojangs brigdier command parsing/dispatching system. Now it works slighly differently, such that it better suports feathers plugin system.To describe how it works i am giving you a somewhat simplified guided tour of the code. 


# Parsers
Leutenants parsing is in the style of [parser combinators](https://en.wikipedia.org/wiki/Parser_combinator). 
```rust
/src/parser/parser.rs

pub trait IterParser {
    /// This assosiated type says what the return value is for the parser. If you have a parser that 
    /// returns a i32, then set it to Extract = (i32,), or if you dont want it returning anythin 
    /// use Extract = ()
    type Extract: Tuple;

    /// You probably just want to set it to (), and always return None in its place for 
    ///'fn parse(&self ...'. Internaly we use this for the And, Or and the Optional parsers.
    /// This makes implementors of the IterParser trait generators of potential parsing results. 
    type ParserState: Default;

    fn parse<'p>(
        &self,
        state: Self::ParserState,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str)>,
        Option<Self::ParserState>,
    );

    /// This method should return a regex that recognises a language that is a superset
    /// of what the parser recognises. Another way to put it. If the parser sucsessfully 
    /// parses some input, then the regex should have matched the part
    /// that it consumed, but it does not have to be the other way arround. Theoretically we could 
    /// therefor always use '.*?' as the regex, but then its not usefull.
    /// We use the regex as a heuristic to determine if a parser can parse some input.
    /// So if for example you expect a parser to be able to parse 
    /// json then a suitable regex could be "\{.*?\}". 
    fn regex(&self) -> String;
}
```

In the "/src/parser" module you find all the parser combinator like And, Opt, Literal, and Space. 

# Regex
The regex module contains a custom regex engine. The reason we want a custom regex solution is that we get two customised features.

## 1) Assosiated values
When we create the dfa representation of our regex we can assosiate every state/node with an assosiated value, that in our case is a command_id: u32. When the regex matches something it also gives the commands that the end state is assosiated with. 

## 2) Early termination
The idea is that we dont use the regular expression for exact matching, but more of a fuzzy match, witch gives us some optimisation options. The simplification step is to observe that if we at some state S in the dfa only could get there by matching one spesiffic command, then we can just say we matched that command. We mark S as a special end state, and break all of its outgoing connections. 
Dfa's are quite memmory intensive, and by doing this we drastically reduces their size.


# Argumets
Arguments are spesialisation of parsers. If you have a type like u32, and you want to say that there is one way of parsing it, then you implement the argument trait for it.

```rust
/argument/mod.rs

pub trait Argument {
    type Parser: IterParser<Extract = (Self,), ParserState = Self::ParserState> + Sized + Default;
    type ParserState: Default;
}
```

# Command

## Builder
In theory you could specify a combination of 3 parsers A,B,C as A followed by B followeb by C as
```rust
And {
    a : A,
    b : And {
        a : B,
        b : C,
    }
}
```
This is not too bad if one does not need spaces. But betweene every parser in Feather, we expect spaces. We therefor need to write :
```rust
And {
    a : A,
    b: And {
        a: Spaces,
        b: And {
            a : B,
            b : And {
                a : Spaces,
                b : C,
            }
        }
    }
}
```
But this is somewhat verbose, so command/builder.rs contains a builder pattern for parsers. 
A,B and C are all parsers. 
```rust
A
 .space()
 .followed_by(B)
 .space()
 .followed_by(C);
```


## Command
A command can be though of as the unification of a parsers and a mapping/function/FnMut on the parsers output.  
```rust
/command/command.rs

pub trait Command {
    type GameState;
    type CommandResult;
    fn call(&self, gamestate: Self::GameState, input: &str) -> anyhow::Result<Self::CommandResult>;
    fn regex(&self) -> String;
}
```

