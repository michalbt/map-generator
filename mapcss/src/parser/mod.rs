use cssparser::Parser;

use crate::error::MapCssParseError;

mod selector;
mod stylesheet;

pub(crate) trait Parse {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>>
    where
        Self: Sized;
}
