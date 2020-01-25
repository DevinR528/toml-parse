use super::err::ParseTomlError;
use super::token::Muncher;
pub trait Parse {
    type Item;
    /// Parse a valid toml str into to a toml token.
    ///
    /// Item is the type returned and T is the input
    fn parse(muncher: &mut Muncher) -> Result<Self::Item, ParseTomlError>;
}
