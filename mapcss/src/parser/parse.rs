use cssparser::Parser;
use map_data::TagMapping;

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
    pub tag_mapping: TagMapping,
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
