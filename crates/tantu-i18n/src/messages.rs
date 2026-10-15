//! [`Translations`], [`Args`] and [`ResourceError`]: Fluent resources per locale. Spec:
//! `docs/specs/i18n/localization.md` (I18N-MSG-01..05).

use crate::Locale;

/// A problem in a Fluent resource (syntax error or duplicate id), with its byte offset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceError {
    /// What went wrong.
    pub message: String,
    /// Where, as a byte offset into the resource.
    pub offset: usize,
}

/// Fluent resources for a set of locales.
pub struct Translations {
    #[allow(dead_code)] // Used once lookup is implemented.
    pub(crate) fallback: Locale,
}

impl Translations {
    /// No resources yet; lookups end at `fallback`.
    pub fn new(fallback: Locale) -> Self {
        Translations { fallback }
    }

    /// Adds a resource (Fluent syntax) for `locale`; a later resource's messages replace
    /// earlier ones with the same id. Errors are returned and the valid messages kept.
    pub fn add(&mut self, locale: Locale, source: &str) -> Result<(), Vec<ResourceError>> {
        let _ = (locale, source);
        todo!("I18N-MSG-01")
    }

    /// The locales with resources, the fallback included.
    pub fn locales(&self) -> Vec<Locale> {
        todo!("I18N-MSG-05")
    }

    /// The best available locale for `requested` (in preference order), else the fallback.
    pub fn negotiate(&self, requested: &[Locale]) -> Locale {
        let _ = requested;
        todo!("I18N-MSG-05")
    }
}

/// A string or number argument.
#[derive(Clone, Debug, PartialEq)]
pub enum ArgValue {
    /// Inserted as is.
    Text(String),
    /// Selects plural variants and is written in the locale's number format.
    Number(f64),
}

impl From<&str> for ArgValue {
    fn from(value: &str) -> Self {
        ArgValue::Text(value.to_owned())
    }
}

impl From<String> for ArgValue {
    fn from(value: String) -> Self {
        ArgValue::Text(value)
    }
}

macro_rules! number_args {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for ArgValue {
                fn from(value: $ty) -> Self {
                    ArgValue::Number(value as f64)
                }
            }
        )*
    };
}

number_args!(i32, i64, u32, u64, usize, f32, f64);

/// Message arguments.
#[derive(Clone, Debug, Default)]
pub struct Args {
    pub(crate) values: Vec<(String, ArgValue)>,
}

impl Args {
    /// No arguments.
    pub fn new() -> Self {
        Args::default()
    }

    /// Sets the argument `name` (replacing an earlier value).
    pub fn set(mut self, name: &str, value: impl Into<ArgValue>) -> Self {
        let value = value.into();
        self.values.retain(|(n, _)| n != name);
        self.values.push((name.to_owned(), value));
        self
    }
}
