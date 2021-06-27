# Lieutenant-Parser
This crate is meant to be used together with lieutenant-dispatcher, a command parsing and dispatching
system created to be used by the feather minecraft server. The dispatcher works by using regular expressions
to determine if some user input like '/msg killerbunny You stole my name' is a valid command, and returns a
id of what plugin and parser is meant to handle that command. 


This crate provides a system for building parsers that can tell what regex encompases when they can trigger.
If you have a look at 'parser/src/builder' you can see the builder pattern you can use for command building.

```rust
let cmd: CommandSpec<_,_,_,_,_> = literal("/echo").on_call(|| move |_x: usize, _y: &mut usize| {
        10 as usize
}); 

let x = cmd.call((6, &mut 7), "/echo");
let regex = cmd.regex();
```












