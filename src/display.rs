use std::fmt::{self, Debug, Formatter};

/// How to write a value to logs. Defaults to using the Debug impl, but can be overridden.
pub trait LogDisplay: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl<'a> LogDisplay for &'a str {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}
