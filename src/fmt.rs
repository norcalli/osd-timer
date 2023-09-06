use std::fmt::{self, Display};

pub struct FmtFn<F>(pub F)
where
    F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result;

impl<F> std::fmt::Display for FmtFn<F>
where
    F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0)(f)
    }
}

impl<F> From<F> for FmtFn<F>
where
    F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result,
{
    fn from(v: F) -> Self {
        FmtFn(v)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FixedStr<const N: usize>(pub [u8; N]);

impl<const N: usize> FixedStr<N> {
    #[inline]
    pub const fn as_str(&self) -> &str {
        match self.try_as_str() {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }
    #[inline]
    pub const fn try_as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.0.as_slice())
    }
}

impl<const N: usize> std::fmt::Display for FixedStr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.try_as_str() {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "[{e}]"),
        }
    }
}

pub struct OrDisplay<T: Display, U>(Option<T>, U);

impl<T: Display, U: Display> Display for OrDisplay<T, U> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Some(t) => t.fmt(fmt),
            None => self.1.fmt(fmt),
        }
    }
}

pub trait OrDisplayExt<T: Display> {
    fn or_display<U: Display>(&self, u: U) -> OrDisplay<&T, U>;

    fn into_or_display<U: Display>(self, u: U) -> OrDisplay<T, U>;
}

impl<T: Display> OrDisplayExt<T> for Option<T> {
    fn or_display<U: Display>(&self, u: U) -> OrDisplay<&T, U> {
        OrDisplay(self.as_ref(), u)
    }

    fn into_or_display<U: Display>(self, u: U) -> OrDisplay<T, U> {
        OrDisplay(self, u)
    }
}
