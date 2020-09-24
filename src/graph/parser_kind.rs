
use std::ops::RangeInclusive;
use crate::error::Result;
use crate::values::Value;
use crate::automaton::{Pattern};
use crate::pattern;

#[derive(Debug, PartialEq)]
pub enum StringProperty {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

/// Describes a parser from [this list](https://wiki.vg/Command_Data#Parsers).
/// Doesn't actually provide a means to parse functions—this is only
/// used to build the Declare Commands packet.
#[derive(Debug, PartialEq)]
pub enum ParserKind {
    Bool,
    Double(RangeInclusive<f64>),
    Float(RangeInclusive<f32>),
    Integer(RangeInclusive<i32>),
    String(StringProperty),
    Entity {
        /// Whether only one entity is allowed.
        only_one: bool,
        /// Whether only players will be included.
        player_required: bool,
    },
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    Color,
    ChatComponent,
    Message,
    JsonNbt,
    NbtPath,
    Objective,
    ObjectiveCritera,
    Operation,
    Particle,
    Rotation,
    ScoreboardSlot,
    ScoreHolder {
        /// Whether more than one entity will be allowed.
        multiple_allowed: bool,
    },
    Swizzle,
    Team,
    ItemSlot,
    ResourceLocation,
    MobEffect,
    Function,
    EntityAnchor,
    Range {
        decimals_allowed: bool,
    },
    IntRange,
    FloatRange,
    ItemEnchantment,
    EntitySummon,
    Dimension,
    Uuid,
    NbtTag,
    NbtCompoundTag,
    Time,
}

impl ParserKind {
    pub fn parse(&self, input: &str) -> Result<Value> {
        use ParserKind::*;
        match self {
            IntRange => {
                Ok(Value::I32(input.parse::<i32>().unwrap()))
            }
            _ => unimplemented!(),
        }
    }
}

impl<'a> From<&'a ParserKind> for Pattern<'a> {
    fn from(parser: &'a ParserKind) -> Self {
        let optional = Pattern::optional;
        let concat = Pattern::concat;

        match parser {
            ParserKind::Bool => Pattern::Alt(&[pattern!("true"), pattern!("false")]),
            ParserKind::String(StringProperty::SingleWord) => Pattern::WORD,
            ParserKind::Integer(_) => pattern!(["0123456789"]["0123456789"]*),
            // ParserKind::Double(_) => concat(&[optional(&pattern!("-"))]),
            // ParserKind::String(StringProperty::GreedyPhrase) => Pattern::concat(&[Pattern::WORD, Pattern::SPACE]).repeat(),
            _ => todo!()
        }
    }
}
