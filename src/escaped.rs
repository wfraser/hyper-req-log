use std::fmt::{Display, Formatter, Write};

pub struct Escaped<'a> {
    bytes: &'a [u8],
}

impl<'a, T: AsRef<[u8]> + ?Sized> From<&'a T> for Escaped<'a> {
    fn from(value: &'a T) -> Self {
        Self {
            bytes: value.as_ref(),
        }
    }
}

impl<'a, T: AsRef<[u8]>> From<Option<&'a T>> for Escaped<'a> {
    fn from(value: Option<&'a T>) -> Self {
        Self {
            bytes: value.map(AsRef::as_ref).unwrap_or(&[]),
        }
    }
}

impl<'a> Display for Escaped<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.bytes.is_empty() {
            return f.write_str("\"\"");
        }
        let mut range = 0..self.bytes.len();
        while range.start != self.bytes.len() {
            match std::str::from_utf8(&self.bytes[range.clone()]) {
                Ok(s) => {
                    if range == (0..self.bytes.len()) {
                        if s.chars().all(|c| c.is_ascii_graphic() && c != '\\') {
                            return f.write_str(s);
                        } else {
                            f.write_char('"')?;
                        }
                    }
                    for c in s.chars() {
                        if c == '\\' {
                            f.write_str("\\\\")?;
                        } else if !c.is_ascii_graphic() {
                            write!(f, "{}", c.escape_debug())?;
                        } else {
                            f.write_char(c)?;
                        }
                    }
                }
                Err(e) => {
                    if range == (0..self.bytes.len()) {
                        f.write_char('"')?;
                    }
                    if e.valid_up_to() == 0 {
                        if let Some(len) = e.error_len() {
                            range.end = range.start + len + 1;
                        }
                        // We've isolated the garbage, print it escaped
                        for byte in &self.bytes[range.clone()] {
                            write!(f, "\\x{byte:02x}")?;
                        }
                    } else {
                        range.end = range.start + e.valid_up_to();
                        continue;
                    }
                }
            }
            range = range.end..self.bytes.len();
        }
        f.write_char('"')
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(Escaped::from("").to_string(), "\"\"");
        assert_eq!(Escaped::from("hello").to_string(), "hello");
        assert_eq!(Escaped::from("hello world").to_string(), "\"hello world\"");
        assert_eq!(Escaped::from("non-åsçïï").to_string(), "\"non-åsçïï\"");
        assert_eq!(Escaped::from("emoji 👍").to_string(), "\"emoji 👍\"");
        assert_eq!(
            Escaped::from("back\\slash").to_string(),
            "\"back\\\\slash\""
        );
        assert_eq!(
            Escaped::from(b"bad utf8 \xc3\x28!").to_string(),
            "\"bad utf8 \\xc3\\x28!\""
        );
        assert_eq!(
            Escaped::from(b"bad utf8 \xc3\x28").to_string(),
            "\"bad utf8 \\xc3\\x28\""
        );
        assert_eq!(
            Escaped::from(b"\xc3\x28 bad utf8").to_string(),
            "\"\\xc3\\x28 bad utf8\""
        );
    }
}
