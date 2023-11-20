use std::{
    fmt::{self, Write},
    str,
};

#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct CxxStr(pub Vec<u8>);

impl fmt::Display for CxxStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf8(&self.0, f, str::chars)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for CxxStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct CxxWStr(pub Vec<u16>);

impl fmt::Display for CxxWStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf16(&self.0, f, core::iter::once)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for CxxWStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

fn display_utf16<Transformer: Fn(char) -> O, O: Iterator<Item = char>>(
    input: &[u16],
    f: &mut fmt::Formatter<'_>,
    t: Transformer,
) -> fmt::Result {
    char::decode_utf16(input.iter().copied())
        .flat_map(|r| t(r.unwrap_or(char::REPLACEMENT_CHARACTER)))
        .try_for_each(|c| f.write_char(c))
}

fn display_utf8<'a, Transformer: Fn(&'a str) -> O, O: Iterator<Item = char> + 'a>(
    mut input: &'a [u8],
    f: &mut fmt::Formatter<'_>,
    t: Transformer,
) -> fmt::Result {
    // Adapted from <https://doc.rust-lang.org/std/str/struct.Utf8Error.html>
    loop {
        match str::from_utf8(input) {
            Ok(valid) => {
                t(valid).try_for_each(|c| f.write_char(c))?;
                break;
            }
            Err(error) => {
                let (valid, after_valid) = input.split_at(error.valid_up_to());

                t(unsafe { str::from_utf8(valid).unwrap_unchecked() })
                    .try_for_each(|c| f.write_char(c))?;
                f.write_char(char::REPLACEMENT_CHARACTER)?;

                if let Some(invalid_sequence_length) = error.error_len() {
                    input = unsafe { after_valid.get_unchecked(invalid_sequence_length..) };
                } else {
                    break;
                }
            }
        }
    }
    Ok(())
}
