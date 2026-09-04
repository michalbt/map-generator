use cssparser::Parser;

use crate::error::MapCssParseError;

mod selector;
mod stylesheet;
#[cfg(test)]
mod test_helpers;

pub(crate) trait Parse {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>>
    where
        Self: Sized;
}
