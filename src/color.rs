use clap::builder::TypedValueParser;
use clap::builder::ValueParserFactory;
use clap::error::ErrorKind;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;

#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone)]
pub struct ColorParser;

impl TypedValueParser for ColorParser {
    type Value = Color;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        TypedValueParser::parse(self, cmd, arg, value.to_owned())
    }

    fn parse(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: OsString,
    ) -> Result<Self::Value, clap::Error> {
        let value = value
            .into_string()
            .map_err(|_| clap::Error::new(ErrorKind::InvalidUtf8))?;

        let value = value.as_bytes();

        fn incorrect_color() -> clap::Error {
            clap::Error::raw(ErrorKind::InvalidValue, "Incorrect color value\n")
        }

        let len @ (3 | 6) = value.len() else {
            return Err(incorrect_color());
        };

        fn from_hex(val: &[u8; 2]) -> Option<u8> {
            fn _inner_from_hex(v: u8) -> Option<u8> {
                Some(match v {
                    x @ b'0'..=b'9' => x - b'0',
                    x @ b'a'..=b'f' => x - b'a' + 0x0a,
                    x @ b'A'..=b'F' => x - b'A' + 0x0a,
                    _ => return None,
                })
            }

            let h = _inner_from_hex(val[0])?;
            let l = _inner_from_hex(val[1])?;

            Some((h << 4) | l)
        }

        let (r, g, b) = match len {
            3 => {
                let r = [value[0], value[0]];
                let g = [value[1], value[1]];
                let b = [value[2], value[2]];
                (r, g, b)
            }
            6 => {
                let r = [value[0], value[1]];
                let g = [value[2], value[3]];
                let b = [value[4], value[5]];
                (r, g, b)
            }
            _ => unreachable!(),
        };

        let r = from_hex(&r).ok_or_else(incorrect_color)?;
        let g = from_hex(&g).ok_or_else(incorrect_color)?;
        let b = from_hex(&b).ok_or_else(incorrect_color)?;

        Ok(Color { r, g, b })
    }
}

impl ValueParserFactory for Color {
    type Parser = ColorParser;

    fn value_parser() -> Self::Parser {
        ColorParser
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:02x}{:02x}{:02x}", self.r, self.g, self.b))
    }
}
