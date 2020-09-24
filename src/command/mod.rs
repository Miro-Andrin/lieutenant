mod builder;

use crate::generic::{Func, Tuple};
use crate::values::{FromValues, Values};
use std::marker::PhantomData;
use crate::graph::{NodeKind, ParserKind, StringProperty};

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
    const NODE_KIND: NodeKind;
}

macro_rules! node_parser {
    ($ident:ident; $parser:expr) => {
        impl NodeDescriptor for $ident {
            const NODE_KIND: NodeKind = NodeKind::Argument { parser: $parser };
        }
    };
}

node_parser!(bool; ParserKind::Bool);
node_parser!(f64; ParserKind::Double(f64::MIN..=f64::MAX));
node_parser!(f32; ParserKind::Float(f32::MIN..=f32::MAX));
node_parser!(i32; ParserKind::Integer(i32::MIN..=i32::MAX));
node_parser!(String; ParserKind::String(StringProperty::SingleWord));


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