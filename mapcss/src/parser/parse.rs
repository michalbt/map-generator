use cssparser::Parser;
use map_data::{KeyedMapping, RelationRoleKey, TagKey};

use crate::error::MapCssParseError;

pub(crate) trait Parse {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>>
    where
        Self: Sized;
}

#[derive(Debug, Default)]
pub(crate) struct ParseContext {
    pub tag_mapping: KeyedMapping<TagKey>,
    pub relation_role_mapping: KeyedMapping<RelationRoleKey>,
}

impl ParseContext {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse<'i, T>(
        &mut self,
    ) -> impl FnMut(&mut Parser<'i, '_>) -> Result<T, MapCssParseError<'i>>
    where
        T: Parse,
    {
        |inp| T::parse(inp, self)
    }
}
