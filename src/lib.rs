mod generic;
mod parsers;

use std::fmt;
use std::marker::PhantomData;

use anyhow::{bail, Result};
use generic::{Func, Product, Tuple};
use parsers::{And, Parser, Space};
use regex_syntax::hir::{self, Hir};

pub trait Command {
    type Context;
    type Output;

    fn exec(&self, context: Self::Context, args: &[&str]) -> Result<Self::Output>;
}

pub struct CommandSpec<P, C, Ctx> {
    parser: P,
    callback: C,
    context: PhantomData<Ctx>,
}

impl<P, C, Ctx> CommandSpec<P, C, Ctx>
where
    P: Parser,
{
    pub fn regex(&self) -> Hir {
        let mut hirs = Vec::new();
        hirs.push(Hir::anchor(hir::Anchor::StartText));
        self.parser.regex(&mut hirs);
        hirs.push(Hir::anchor(hir::Anchor::EndText));
        Hir::concat(hirs)
    }
}

impl<P, C, Ctx> Command for CommandSpec<P, C, Ctx>
where
    P: Parser,
    <P as Parser>::Extract: Tuple,
    C: Func<Product<Ctx, <P::Extract as Tuple>::HList>, Output = ()>,
{
    type Context = Ctx;
    type Output = C::Output;

    fn exec(&self, context: Self::Context, args: &[&str]) -> Result<Self::Output> {
        let args = self.parser.parse(&mut &args[..])?;
        Ok(self.callback.call(Product(context, args.hlist())))
    }
}

impl<P, C, Ctx> fmt::Display for CommandSpec<P, C, Ctx>
where
    P: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.parser.fmt(f)?;
        Ok(())
    }
}

pub trait CommandBuilder: Sized + Parser {
    fn and<B>(self, other: B) -> And<Self, B> {
        And { a: self, b: other }
    }

    fn space<B>(self, other: B) -> And<Self, And<Space, B>> {
        self.and(Space.and(other))
    }

    fn build<T, Ctx>(self, callback: T) -> CommandSpec<Self, T, Ctx>
    where
        T: Func<Product<Ctx, <Self::Extract as Tuple>::HList>>,
    {
        CommandSpec {
            parser: self,
            callback,
            context: Default::default(),
        }
    }
}

impl<T> CommandBuilder for T where T: Parser + Sized {}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use anyhow::bail;
    use regex_syntax::hir::Hir;

    use crate::{Command, parsers::{Space, argument, literal, repeat}};
    use crate::{Result, CommandBuilder, CommandSpec, Parser};

    #[test]
    fn hello_world() {
        let hello = literal("hello");
        let world = literal("world");

        let hello_world = hello.space(world);

        println!("{:?}", hello_world.parse(&mut &["hello", "world"][..]));

        // assert!(hello_world.parse(&mut "hello world").is_ok());
        // assert!(hello_world.parse(&mut "hello foo").is_err());

        let command = hello_world.build(|_: ()| println!("hello world!"));

        let caps = Regex::new(&format!("{}", command.regex())).unwrap().captures("hello world").unwrap();

        let mut args = vec![];
        for i in 1..caps.len() {
            args.push(&caps[i]);
        }

        println!("{:?}", args);

        println!("{:?}", command.exec((), &args[..]));

        assert!(command.exec((), &args[..]).is_ok());
    }

    #[test]
    fn foo() {
        let foo: CommandSpec<_, _, _> = literal("foo")
            .space(argument())
            .build(|_: (), x: i32| println!("foo {}", x));

        println!("{}", foo.regex());

        // // println!("{}", foo);

        // println!("{:?}", foo.exec((), "foo 42"));
    }

    #[test]
    fn regex() {
        let hir = regex_syntax::Parser::new().parse("foo +(\\d+)").unwrap();
        println!("{}", hir);
    }

    #[test]
    fn bar() {
        // let bar = literal("bar")
        //     .space(argument().space(argument()))
        //     .build(|_: (), x: i32, y: i32| println!("foo {} {}", x, y));

        // println!("{:?}", bar.exec((), "foo 42 69"));
    }

    #[test]
    fn tp() {
        let tp = literal("tp")
            .space(argument())
            .space(argument())
            .space(argument())
            .build(|_: (), x: i32, y: i32, z: i32| println!("tp: {} {} {}", x, y, z));

        // passing single str into slice of str

        println!("{:?}", tp.exec((), &["10", "11", "12"]));

        assert!(tp.exec((), &["10", "11", "12"]).is_ok());
    }

    // enum Modifier {
    //     Align(i32),
    //     In(String, String),
    // }

    // impl Parser for Modifier {
    //     type Extract = (Modifier,);

    //     fn parse(&self, input: &mut &[&str]) -> Result<Self::Extract> {
    //         let modifier = input[0];
    //         match modifier {
    //             "align" => Ok((Modifier::Align(input[1].parse()?),)),
    //             "in" => Ok((Modifier::In(input[1].to_owned(), input[2].to_owned()))),
    //             _ => bail!("did not match any valid modifier"),
    //         }
    //     }

    //     fn regex(&self, syntax: &mut Vec<Hir>) {
    //         todo!()
    //     }
    // }

    #[test]
    fn execute() {
        // let execute = literal("execute")
        //     .space(repeat(argument()))
        //     .option(Space.and(argument()))
        //     .build(|_: (), modifiers: Vec<(Modifier,)>, run: Option<String>| println!("{:?} run {:?}", modifiers, run))
    }

    #[test]
    fn sum() {
        let sum = literal("sum")
            .and(repeat(Space.and(argument())))
            .build(|_: (), numbers: Vec<(i32,)>| println!("sum {:?}", numbers));

        println!("{}", CommandSpec::regex(&sum))
    }

    #[test]
    fn context() {
        // let age = literal("age").build(|age: &mut u32| println!("you are {} years old.", age));

        // let set_age = literal("set_age")
        //     .space(argument())
        //     .build(|age: &mut u32, new_age: u32| {
        //         println!("set age from {} to {}", age, new_age);
        //         *age = new_age;
        //     });

        // let _ = age.exec(&mut 32, "age");
        // let mut age = 32;
        // let _ = set_age.exec(&mut age, "set_age 22");
        // println!("age: {}", age);
    }
}
