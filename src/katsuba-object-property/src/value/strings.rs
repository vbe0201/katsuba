use std::{
    fmt::{self, Write},
    str,
};

/// A string of bytes, not necessarily valid UTF-8.
#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct CxxStr(pub Vec<u8>);

impl fmt::Display for CxxStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf8(&self.0, f)
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

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CxxStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CxxStrVisitor;

        impl serde::de::Visitor<'_> for CxxStrVisitor {
            type Value = CxxStr;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a string, optionally containing \\xHH byte escapes")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                decode_escaped_bytes(v).map(CxxStr).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(CxxStrVisitor)
    }
}

/// A wide string of code points, not necessarily valid UTF-16.
#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct CxxWStr(pub Vec<u16>);

impl fmt::Display for CxxWStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf16(&self.0, f)
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

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CxxWStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CxxWStrVisitor;

        impl serde::de::Visitor<'_> for CxxWStrVisitor {
            type Value = CxxWStr;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a string, optionally containing \\wHHHH unit escapes")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                decode_escaped_units(v).map(CxxWStr).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(CxxWStrVisitor)
    }
}

fn write_char_escaped(c: char, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if c == '\\' {
        f.write_str("\\\\")
    } else {
        f.write_char(c)
    }
}

fn display_utf16(input: &[u16], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for r in char::decode_utf16(input.iter().copied()) {
        match r {
            Ok(c) => write_char_escaped(c, f)?,
            Err(e) => write!(f, "\\w{:04x}", e.unpaired_surrogate())?,
        }
    }
    Ok(())
}

fn display_utf8(mut input: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // Adapted from <https://doc.rust-lang.org/std/str/struct.Utf8Error.html>
    loop {
        match str::from_utf8(input) {
            Ok(valid) => {
                valid.chars().try_for_each(|c| write_char_escaped(c, f))?;
                break;
            }
            Err(error) => {
                let (valid, rest) = input.split_at(error.valid_up_to());
                str::from_utf8(valid)
                    .unwrap()
                    .chars()
                    .try_for_each(|c| write_char_escaped(c, f))?;

                let invalid_len = error.error_len().unwrap_or(rest.len());
                let (invalid, remainder) = rest.split_at(invalid_len);
                invalid.iter().try_for_each(|&b| write!(f, "\\x{b:02x}"))?;

                if error.error_len().is_none() {
                    break;
                }

                input = remainder;
            }
        }
    }
    Ok(())
}

#[cfg(feature = "serde")]
fn decode_escaped_bytes(s: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            continue;
        }

        match chars.next() {
            Some('\\') => out.push(b'\\'),
            Some('x') => out.push(read_hex_digits::<2>(&mut chars)? as u8),
            Some(c) => return Err(format!("unknown escape sequence '\\{c}'")),
            None => return Err("truncated escape sequence at end of string".to_string()),
        }
    }

    Ok(out)
}

#[cfg(feature = "serde")]
fn decode_escaped_units(s: &str) -> Result<Vec<u16>, String> {
    let mut out = Vec::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            let mut buf = [0u16; 2];
            out.extend_from_slice(c.encode_utf16(&mut buf));
            continue;
        }

        match chars.next() {
            Some('\\') => out.push(b'\\' as u16),
            Some('w') => out.push(read_hex_digits::<4>(&mut chars)?),
            Some(c) => return Err(format!("unknown escape sequence '\\{c}'")),
            None => return Err("truncated escape sequence at end of string".to_string()),
        }
    }

    Ok(out)
}

#[cfg(feature = "serde")]
fn read_hex_digits<const N: usize>(chars: &mut str::Chars<'_>) -> Result<u16, String> {
    const { assert!(N <= 4) };

    let mut value: u16 = 0;
    for _ in 0..N {
        let digit = chars
            .next()
            .and_then(|c| c.to_digit(16))
            .ok_or_else(|| format!("expected {N} valid hex digits"))?;

        value = (value << 4) | digit as u16;
    }

    Ok(value)
}
