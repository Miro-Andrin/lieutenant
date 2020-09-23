mod builder;

use crate::generic::{Combine, Func, HList, Tuple};
use crate::values::{FromValues, Values};
use std::marker::PhantomData;
use crate::graph::{NodeKind, ParserKind};

pub use self::builder::{CommandBuilder, BlankBuilder};

pub trait Command {
    type Ctx;
    type Output;

    fn invoke(&self, ctx: &mut Self::Ctx, values: &mut Values) -> Option<Self::Output>;
}

pub struct CommandMapping<Ctx, Args, F> {
    ctx: PhantomData<Ctx>,
    args: PhantomData<Args>,
    callback: F,
}

impl<Ctx, Args, F> Command for CommandMapping<Ctx, Args, F>
where
    Args: FromValues<Ctx> + Tuple,
    F: Func<Args>,
{
    type Ctx = Ctx;
    type Output = F::Output;

    fn invoke(&self, ctx: &mut Ctx, values: &mut Values) -> Option<Self::Output> {
        let args = Args::from_values(ctx, values)?;
        Some(self.callback.call(args))
    }
}

pub fn command<Ctx, Args, F>(callback: F) -> CommandMapping<Ctx, Args, F>
where
    Args: FromValues<Ctx>,
    F: Func<Args>,
{
    CommandMapping {
        ctx: Default::default(),
        args: Default::default(),
        callback,
    }
}

pub trait NodeDescriptor {
    const NodeKind: NodeKind;
}

impl NodeDescriptor for i32 {
    const NodeKind: NodeKind = NodeKind::Argument { parser: ParserKind::IntRange };
}

#[cfg(test)]
mod tests {
    use super::builder::*;
    use crate::graph::GraphMerge;

    #[test]
    fn test() {
        let mut a = BlankBuilder::new()
            .literal("a")
            .param()
            .build::<(), _>(|_x: i32| { println!("hello world!"); Ok(()) });

        let b = BlankBuilder::new()
            .literal("b")
            .param()
            .build(teleport);

        let c = BlankBuilder::new()
            .literal("c")
            .param()
            .build(teleport);
        
        a.merge(b);
        a.merge(c);
    }

    fn teleport(x: i32) -> crate::error::Result<()>{
        println!("it works {}", x);
        Ok(())
    }
}