#[derive(Debug)]
pub enum Value<'a> {
    U32(u32),
    I32(i32),
    // Non gready value... do we need to differentiate?
    Word(&'a str),
    // Greedy value
    String(&'a str),
}

pub struct Values<'i> {
    values: &'i Vec<Value<'i>>,
    pos: usize,
}

impl<'i> Clone for Values<'i> {
    fn clone(&self) -> Self {
        Self {
            values: self.values,
            pos: self.pos,
        }
    }
}

impl<'i> From<&'i Vec<Value<'i>>> for Values<'i> {
    fn from(values: &'i Vec<Value<'i>>) -> Self {
        Self {
            values,
            pos: 0,
        }
    }
}

impl<'i> Values<'i> {
    pub fn next(&mut self) -> Option<&Value<'i>> {
        let value = self.values.get(self.pos);
        self.pos += 1;
        value
    }
}

pub trait FromValue<Ctx>: Sized {
    fn from_value(ctx: &mut Ctx, value: &Value) -> Option<Self>;
}

pub trait FromValues<Ctx>: Sized {
    fn from_values(ctx: &mut Ctx, values: &mut Values) -> Option<Self>;
}


impl<Ctx> FromValue<Ctx> for i32 {
    fn from_value(_ctx: &mut Ctx, value: &Value) -> Option<Self> {
        match value {
            Value::I32(n) => Some(*n),
            _ => None,
        }
    }
}

impl<Ctx> FromValues<Ctx> for () {
    fn from_values(_ctx: &mut Ctx, _values: &mut Values) -> Option<Self> {
        Some(())
    }
}

impl<Ctx, T1> FromValues<Ctx> for (T1,)
where
    T1: FromValue<Ctx>,
{
    fn from_values(ctx: &mut Ctx, values: &mut Values) -> Option<Self> {
        Some((T1::from_value(ctx, values.next()?)?,))
    }
}